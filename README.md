# Batcher

Batcher is a tool that helps users generate batches of on-chain promotional codes, powered by [PIVX Promos](https://github.com/PIVX-Labs/PIVX-Promos), with the optional ability of automatically filling them with coins by communicating with a local node.

This was built for creating physical and/or virtual promotional coins that can be redeemed via a redeeming application (typically a frontend lightwallet system).

## Features

- [x] Multi-coin support (PIVX, Bitcoin, and more)
- [x] Create batches of promotional Keypairs
- [x] Automatically fill code addresses with coins
- [x] Export Codes to CSV
- [x] Coin-specific configuration handling

## How It Works

Batcher generates promo codes like `promo-abc123` which are deterministically converted to private keys using a secure hashing algorithm. From these private keys, public addresses are derived according to each cryptocurrency's specifications.

## Supported Coins

Batcher now supports multiple cryptocurrencies, including:

- PIVX (PIV)
- Metrix (MRX)
- DogeCoin (DOGE)
- ... and more!

Adding new coins is simple - just add their parameters to the `get_supported_coins()` function in `src/coins.rs`:

```rust
CoinParams {
    name: "YourCoin".to_string(),
    ticker: "YCN".to_string(),
    conf_dir_name: "YourCoin".to_string(),
    conf_file_name: "yourcoin.conf".to_string(),
    default_rpc_port: 12345,
    pub_key_byte: 30, // Replace with your coin's version byte
    priv_key_byte: 128, // Replace with your coin's WIF byte
    promo_fee: 0.00010000, // Network fee for transactions
}
```

## Platform-Specific Configuration

Batcher automatically handles platform-specific configurations:

- On Windows, config files are located in `AppData\Roaming\{CoinName}`
- On macOS, config files are located in `Library/Application Support/{CoinName}/`
- On Linux, config files are located in `~/.{coincasename}` (lowercase with a dot prefix)

## Usage

When you run Batcher, it will:

1. Prompt you to select which cryptocurrency you want to work with
2. Read the coin's configuration from the appropriate directory
3. Guide you through creating batches of promotional codes
4. Optionally fill the generated addresses with the selected cryptocurrency
5. Save the results to a CSV file

## Building from Source

```bash
cargo build --release
```

## Running Batcher

```bash
./target/release/batcher
```

Follow the interactive prompts to generate your promotional codes.
