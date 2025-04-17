/// Struct that represents a cryptocurrency's parameters
#[derive(Clone, Debug)]
pub struct CoinParams {
    /// Name of the coin (e.g., "PIVX")
    pub name: String,
    /// Ticker symbol (e.g., "PIV")
    pub ticker: String,
    /// Configuration directory name (e.g., "PIVX" for macOS/Windows, becomes ".pivx" on Linux)
    pub conf_dir_name: String,
    /// Configuration file name (e.g., "pivx.conf")
    pub conf_file_name: String,
    /// Default RPC port
    pub default_rpc_port: u16,
    /// Public key version byte for address generation
    pub pub_key_byte: u8,
    /// Private key version byte for WIF format
    pub priv_key_byte: u8,
    /// The network fee paid for the redeemer client
    pub promo_fee: f64,
}

/// Get a list of all supported coins
pub fn get_supported_coins() -> Vec<CoinParams> {
    vec![
        CoinParams {
            name: "PIVX".to_string(),
            ticker: "PIV".to_string(),
            conf_dir_name: "PIVX".to_string(),
            conf_file_name: "pivx.conf".to_string(),
            default_rpc_port: 51473,
            pub_key_byte: 30,
            priv_key_byte: 212,
            promo_fee: 0.00010000,
        },
        CoinParams {
            name: "DogeCoin".to_string(),
            ticker: "DOGE".to_string(),
            conf_dir_name: "Dogecoin".to_string(),
            conf_file_name: "dogecoin.conf".to_string(),
            default_rpc_port: 22555,
            pub_key_byte: 30,
            priv_key_byte: 158,
            promo_fee: 0.01000000,
        },
        CoinParams {
            name: "Metrix".to_string(),
            ticker: "MRX".to_string(),
            conf_dir_name: "MetrixCoin".to_string(),
            conf_file_name: "metrixcoin.conf".to_string(),
            default_rpc_port: 33831,
            pub_key_byte: 50,
            priv_key_byte: 85,
            promo_fee: 0.01000000,
        },
        CoinParams {
            name: "PepeCoin".to_string(),
            ticker: "PEPE".to_string(),
            conf_dir_name: "Pepecoin".to_string(),
            conf_file_name: "pepecoin.conf".to_string(),
            default_rpc_port: 33873,
            pub_key_byte: 56,
            priv_key_byte: 158,
            promo_fee: 0.00010000,
        },
        CoinParams {
            name: "StakeCubeCoin".to_string(),
            ticker: "SCC".to_string(),
            conf_dir_name: "StakeCubeCoin".to_string(),
            conf_file_name: "stakecubecoin.conf".to_string(),
            default_rpc_port: 39999,
            pub_key_byte: 125,
            priv_key_byte: 253,
            promo_fee: 0.00010000,
        },
        CoinParams {
            name: "NewMNSCoin".to_string(),
            ticker: "NMNSC".to_string(),
            conf_dir_name: "nMNSC".to_string(),
            conf_file_name: "nmnsc.conf".to_string(),
            default_rpc_port: 14259,
            pub_key_byte: 53,
            priv_key_byte: 82,
            promo_fee: 0.00010000,
        },
    ]
}