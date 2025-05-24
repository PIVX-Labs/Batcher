use std::{
    env::home_dir,
    fs::{self, OpenOptions},
    io::{self, prelude::*},
};

mod coins;
use coins::{CoinParams, get_supported_coins};

use pivx_rpc_rs::{self, BitcoinRpcClient};

use base58::ToBase58;
use bitcoin_hashes::{sha256, sha256d, Hash};
use ripemd::{Digest, Ripemd160};
use secp256k1::{rand, rand::Rng, PublicKey, Secp256k1, SecretKey};

/// A struct representing an optimized promotional keypair.
///
/// This struct contains a private key of type `SecretKey`, a public key represented as a string,
/// a promotional code represented as a string, and the value, if applicable.
///
pub struct OptimisedPromoKeypair {
    private: SecretKey,
    public: String,
    code: String,
    value: f64,
}

/// A struct representing a promo batch request.
///
/// This struct contains the Value and the Quantity of the batch it represents.
///
pub struct PromoBatch {
    /// The value of the batch
    value: f64,
    /// The quantity of the batch
    qty: u64,
}

/// Iterations required for a PIVX Promo to be derived.
///
/// This constant is an array of `u64` values, representing the iterations required for a PIVX
/// promotional key to be derived. Currently, only one value is present in the array.
///
pub const PROMO_TARGETS: [u64; 1] = [12_500_000];

/// The default coin to use if none is selected
pub const DEFAULT_COIN_TICKER: &str = "PIV";

fn main() {
    // Select which coin to create promo codes for
    let coin_params = select_coin();
    println!("Selected coin: {} ({})", coin_params.name, coin_params.ticker);

    // Parse the coin's config
    let coin_config = parse_coin_conf(&coin_params);

    // Setup the RPC
    let rpc = BitcoinRpcClient::new(
        String::from("http://localhost:") + &coin_config.rpc_port.to_string(),
        Some(coin_config.rpc_user.to_owned()),
        Some(coin_config.rpc_pass.to_owned()),
        4,
        10,
        1000,
    );

    let should_save: bool;
    let mut promo_prefix = String::new();
    let mut filename = String::from("codes");
    let mut batches: Vec<PromoBatch> = Vec::new();

    // If Promo Interactive mode is on: let's ask and figure out ALL the settings beforehand for a fine-tuned experience
    should_save = ask_bool("Would you like to save your batch as a CSV file?", true);
    if should_save {
        filename = ask_string("What would you like to name it?", &filename)
    }
    println!("Perfect, now, let's start planning your batch!");
    println!("----------------------------------------------");
    loop {
        let qty = ask_float(
            format!("Batch {}: how many codes do you want?", batches.len() + 1).as_str(),
            5.0,
        ) as u64;
        let value = ask_float(
            format!(
                "Batch {}: how much {} should each of your {} codes be worth?",
                batches.len() + 1,
                coin_params.ticker,
                qty
            )
            .as_str(),
            1.0,
        );
        batches.push(PromoBatch { value, qty });

        // Clear the screen and log the batches
        clear_terminal_screen();
        println!("----------------------------------------------");
        let mut count = 1;
        let mut total_value = 0.0;
        let mut total_codes: u64 = 0;
        for batch in batches.as_slice() {
            println!(
                " - Batch {}: {} codes of {} {}",
                count, batch.qty, batch.value, coin_params.ticker
            );
            count += 1;
            total_value += batch.value * batch.qty as f64;
            total_codes += batch.qty;
        }
        println!(
            "... for a total of {} codes worth {} {}",
            total_codes, total_value, coin_params.ticker
        );
        println!("----------------------------------------------");

        // Ask if they wanna add more batches, or they're ready to start generating
        let continue_batching = ask_bool("Would you like to add another batch?", false);

        // If it's a no... break the batch creation loop and move on
        if !continue_batching {
            break;
        }
    }

    // Check if they want a prefix used
    promo_prefix = ask_string(
        format!(
            "What prefix would you like to use, if any? For example: promo-{}, or, if omitted: {}",
            get_alpha_numeric_rand(5),
            get_alpha_numeric_rand(6)
        )
        .as_str(),
        &promo_prefix,
    );

    // Create CSV file and write header if saving is enabled
    let csv_filename = if should_save {
        let mut filename_with_ext = filename.clone() + ".csv";
        
        // Check if file already exists
        if std::path::Path::new(&filename_with_ext).exists() {
            println!("Warning: File '{}' already exists!", filename_with_ext);
            println!("If you choose 'No', a new file with a timestamp will be created instead.");
            let overwrite = ask_bool("Do you want to overwrite it?", false);
            
            if !overwrite {
                // Generate a unique filename with timestamp
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                filename_with_ext = format!("{}_{}.csv", filename, timestamp);
                println!("Creating new file: {}", filename_with_ext);
            }
        }
        
        let mut file = fs::File::create(&filename_with_ext).unwrap();
        writeln!(file, "coin,value,code,").unwrap();
        Some(filename_with_ext)
    } else {
        None
    };

    // Start generating!
    println!("Time to begin! Please do NOT cancel or interfere with the generation process!");
    println!("Generating...");
    let mut codes: Vec<OptimisedPromoKeypair> = Vec::new();

    // We'll loop each batch and decrement it's quantity as each code is generated
    let mut batch_count = 1;
    for mut batch in batches {
        let mut code_count = 1;
        // Loop each code within the batch
        while batch.qty >= 1 {
            let mut promo = create_promo_key(&promo_prefix, &coin_params);
            let wif = secret_to_wif(promo.private, coin_params.priv_key_byte);
            println!(
                "Code {code_count} of batch {batch_count}: Promo: '{}' - Address: {} - WIF: {wif}",
                promo.code, promo.public
            );

            // If these codes have value, fill 'em!
            if batch.value > 0.0 {
                println!(" - Filling with {} {}...", batch.value, coin_params.ticker);

                // Attempt filling the code's address
                loop {
                    match rpc.sendtoaddress(
                        &promo.public,
                        batch.value + coin_params.promo_fee,
                        Some(&format!("{} Promos pre-fill", coin_params.name)),
                        Some(""),
                        Some(false),
                    ) {
                        Ok(tx_id) => {
                            println!(" - TX: {}", tx_id);
                            promo.value = batch.value;
                            break;
                        }
                        Err(e) => {
                            eprintln!(
                                " - TX failed with error: \"{}\". Retrying in 10 seconds...",
                                e
                            );
                            std::thread::sleep(std::time::Duration::from_secs(10));
                        }
                    }
                }
            }

            // Append to CSV file immediately if saving is enabled
            if let Some(ref csv_file) = csv_filename {
                let mut file = OpenOptions::new()
                    .append(true)
                    .open(csv_file)
                    .unwrap();
                writeln!(file, "{},{},{}", coin_params.ticker.to_lowercase(), promo.value, promo.code).unwrap();
            }

            // Push this promo
            codes.push(promo);

            // Decrement batch quantity
            batch.qty -= 1;
            code_count += 1;
        }
        batch_count += 1;
    }

    // CSV already saved during generation if enabled
    if should_save {
        if let Some(ref csv_file) = csv_filename {
            println!("Saved batch as \"{}\"!", csv_file);
        }
    }

    println!("Finished! - Quitting...");
}

pub fn ask_float(question: &str, default: f64) -> f64 {
    println!("{question} (default: \"{default}\")");

    // We run this in a loop; incase the user enters a weird non-number; we'll catch it, tell them to stop being stupid, and ask again
    let mut float_answer = default;
    loop {
        print!("{default}: ");
        io::stdout().flush().unwrap_or_default();

        // Wait for input
        let mut answer = String::new();
        let stdin = std::io::stdin();
        stdin.read_line(&mut answer).unwrap_or_default();
        answer = answer.trim().to_string();

        // If it's empty: use the default
        if answer.is_empty() {
            break;
        }

        // Attempt to parse the float
        float_answer = match answer.parse() {
            Ok(number) => number,
            Err(_) => 0.0,
        };

        // If it's a good answer, we break the loop
        if float_answer >= 0.0 {
            break;
        } else {
            eprintln!("Weird answer... try again!");
        }
    }

    // Add some natural spacing
    println!("");

    // Return our glorious float
    float_answer
}

pub fn ask_string(question: &str, default: &str) -> String {
    println!("{question} (default: \"{default}\")");
    print!("{default}: ");
    io::stdout().flush().unwrap_or_default();

    // Wait for input
    let mut answer = String::new();
    let stdin = std::io::stdin();
    stdin.read_line(&mut answer).unwrap_or_default();
    answer = answer.trim().to_string();

    // Add some natural spacing
    println!("");

    // If it's empty: use the default
    if answer.is_empty() {
        default.to_string()
    } else {
        answer
    }
}

pub fn ask_bool(question: &str, default: bool) -> bool {
    let default_answer_string = match default {
        true => "Y/n",
        false => "y/N",
    };
    println!("{question}");
    print!("{default_answer_string}: ");
    io::stdout().flush().unwrap_or_default();

    // Wait for input
    let mut answer = String::new();
    let stdin = std::io::stdin();
    stdin.read_line(&mut answer).unwrap_or_default();
    // Trim and lowercase it for simplicity
    answer = answer.trim().to_string().to_ascii_lowercase();

    // Add some natural spacing
    println!("");

    // Check if Yes/No - a non-matching answer will use default
    match answer.as_str() {
        "y" => true,
        "n" => false,
        _ => default,
    }
}

/// Clear (wipe) the terminal screen
pub fn clear_terminal_screen() {
    print!("{esc}c", esc = 27 as char);
}

/// Converts a secret key into Wallet Import Format (WIF).
///
/// # Arguments
///
/// * `privkey` - The secret key to be converted.
/// * `version_byte` - The version byte for the WIF format (coin-specific).
///
/// # Returns
///
/// The secret key in WIF format as a string.
///
pub fn secret_to_wif(privkey: SecretKey, version_byte: u8) -> String {
    // Convert into byte format
    let privkey_bytes = privkey.secret_bytes();

    // Format the byte payload into WIF format
    let mut wif_bytes = vec![version_byte];
    wif_bytes.extend_from_slice(&privkey_bytes);
    wif_bytes.extend_from_slice(&[1]);

    // Concat the WIF bytes with it's SHA256d checksum.
    let sha256d_wif = sha256d::Hash::hash(&wif_bytes).into_inner();
    wif_bytes.extend_from_slice(&sha256d_wif[..4]);

    // Return the WIF String
    wif_bytes.to_base58()
}

/// Converts a public key into a coin address.
///
/// # Arguments
///
/// * `pubkey` - The public key to be converted.
/// * `version_byte` - The version byte for the address format (coin-specific).
///
/// # Returns
///
/// The coin address as a string.
///
pub fn pubkey_to_address(pubkey: PublicKey, version_byte: u8) -> String {
    // Convert into byte format
    let pubkey_bytes = pubkey.serialize();

    // First sha256 round of the compressed pubkey
    let pre_ripemd = sha256::Hash::hash(&pubkey_bytes).into_inner();

    // Then a ripemd160 round
    let mut ripemd_factory = Ripemd160::new();
    ripemd_factory.update(&pre_ripemd);
    let public_key_hash = ripemd_factory.finalize();

    // Create the double-SHA256 Checksum for the network public key hash
    let mut address_bytes = vec![version_byte];
    address_bytes.extend_from_slice(&public_key_hash);
    let sha256d = sha256d::Hash::hash(&address_bytes).into_inner();

    // Concat the address bytes with it's checksum.
    address_bytes.extend_from_slice(&sha256d[..4]);

    // Return the Base58 address
    address_bytes.to_base58()
}

/// A string representing the base58 charset for generating alphanumeric random values.
///
const MAP_ALPHANUMERIC: &str = "abcdefghijkmnopqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ123456789";

/// Returns a vector of random bytes of the specified size.
///
/// # Arguments
///
/// * `n_size` - The number of random bytes to generate.
///
/// # Returns
///
/// A vector of random bytes.
///
pub fn get_safe_rand(n_size: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut random_values = vec![0; n_size];
    rng.fill(&mut random_values[..]);
    random_values
}

/// Returns a randomly generated alphanumeric string of the specified size.
///
/// # Arguments
///
/// * `n_size` - The desired length of the generated string.
///
/// # Returns
///
/// A randomly generated alphanumeric string.
///
pub fn get_alpha_numeric_rand(n_size: usize) -> String {
    let mut result = String::new();
    let rand_values = get_safe_rand(n_size);
    for byte in rand_values {
        let index = (byte % MAP_ALPHANUMERIC.len() as u8) as usize;
        result.push(MAP_ALPHANUMERIC.chars().nth(index).unwrap());
    }
    result
}

/// Creates a crypto Promos keypair based on a given prefix and coin parameters.
///
/// # Arguments
///
/// * `prefix` - A reference to a String representing the prefix of the promotional code.
/// * `coin_params` - A reference to the CoinParams for the selected coin.
///
/// # Returns
///
/// An `OptimisedPromoKeypair` struct containing the generated private and public keys, along with the promo code.
///
pub fn create_promo_key(prefix: &String, coin_params: &CoinParams) -> OptimisedPromoKeypair {
    // Precompute a Secp256k1 context
    let secp = Secp256k1::new();

    // Select the latest Target
    let target = PROMO_TARGETS.last().unwrap();

    // Generate entropy and append it to the promo code
    // Omitted prefixes add an extra character for higher entropy - with prefix, we deduct a character.
    let promo_code = if prefix.is_empty() {
        get_alpha_numeric_rand(6)
    } else {
        prefix.to_owned() + "-" + &get_alpha_numeric_rand(5)
    };

    // Convert the Promo Code to it's first SHA256 hash
    let mut promo_key = sha256::Hash::hash(promo_code.as_bytes()).into_inner();

    // Recursively hash until we hit the target (minus one, as promo_key hashes it once)
    let mut iterations: u64 = 1;
    while &iterations < target {
        promo_key = sha256::Hash::hash(&promo_key).into_inner();
        iterations += 1;
    }

    // Generate the final keys
    let private = SecretKey::from_slice(&promo_key).unwrap();
    let public = pubkey_to_address(
        PublicKey::from_secret_key(&secp, &private),
        coin_params.pub_key_byte
    );

    OptimisedPromoKeypair {
        private,
        public,
        code: promo_code,
        value: 0.0,
    }
}

pub struct RpcConfig {
    pub rpc_user: String,
    pub rpc_pass: String,
    pub rpc_port: u16,
}

/// Selects a coin from the list of supported coins
pub fn select_coin() -> CoinParams {
    let supported_coins = get_supported_coins();
    
    println!("Which coin are you creating Promo Codes for?");
    
    for (i, coin) in supported_coins.iter().enumerate() {
        println!("{}. {} ({})", i + 1, coin.name, coin.ticker);
    }
    
    let default_idx = supported_coins
        .iter()
        .position(|c| c.ticker == DEFAULT_COIN_TICKER)
        .unwrap_or(0) + 1;
    
    let selection = ask_float(
        format!("Enter a number (1-{}) to select a coin", supported_coins.len()).as_str(),
        default_idx as f64,
    ) as usize;
    
    // Make sure the selection is valid
    if selection < 1 || selection > supported_coins.len() {
        println!("Invalid selection, using default: {} ({})", 
                 supported_coins[default_idx - 1].name, 
                 supported_coins[default_idx - 1].ticker);
        return supported_coins[default_idx - 1].clone();
    }
    
    supported_coins[selection - 1].clone()
}

pub fn parse_coin_conf(coin_params: &CoinParams) -> RpcConfig {
    let mut conf_dir = home_dir().unwrap_or_default();
    if cfg!(target_os = "windows") {
        conf_dir.push(format!("AppData\\Roaming\\{}", coin_params.name));
    } else if cfg!(target_os = "macos") {
        conf_dir.push(format!("Library/Application Support/{}/", coin_params.name));
    } else {
        // On Linux, use lowercase name with dot prefix
        let linux_dir_name = format!(".{}", coin_params.conf_dir_name.to_lowercase());
        conf_dir.push(linux_dir_name);
    }
    let conf_file = conf_dir.join(&coin_params.conf_file_name);

    let mut defaults = RpcConfig {
        rpc_user: String::from("user"),
        rpc_pass: String::from("pass"),
        rpc_port: coin_params.default_rpc_port,
    };

    let contents = match fs::read_to_string(conf_file) {
        Ok(c) => c,
        Err(_) => return defaults,
    };

    for line in contents.lines() {
        let parts: Vec<_> = line.splitn(2, '=').collect();
        match parts[..] {
            ["rpcuser", user] => defaults.rpc_user = user.to_owned(),
            ["rpcpassword", pass] => defaults.rpc_pass = pass.to_owned(),
            ["rpcport", port] => defaults.rpc_port = port.parse().unwrap_or(defaults.rpc_port),
            _ => {}
        }
    }

    defaults
}

pub fn compile_to_csv(promos: Vec<OptimisedPromoKeypair>, coin_ticker: &str) -> String {
    let mut csv = String::from("coin,value,code,\n");

    for promo in promos {
        // Store the selected coin ticker in the CSV
        csv.push_str(&format!("{},{},{}\n", coin_ticker.to_lowercase(), promo.value, promo.code));
    }
    csv
}
