//! BitCell Wallet CLI
//!
//! Command-line interface for the BitCell wallet.

use bitcell_wallet::{Chain, Mnemonic, Wallet, WalletConfig};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "bitcell-wallet")]
#[command(about = "BitCell blockchain wallet", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new wallet with a fresh mnemonic
    Create {
        /// Wallet name
        #[arg(short, long, default_value = "Default Wallet")]
        name: String,
    },
    /// Restore a wallet from a mnemonic phrase
    Restore {
        /// Mnemonic phrase (24 words)
        #[arg(short, long)]
        mnemonic: String,
        /// Optional passphrase
        #[arg(short, long, default_value = "")]
        passphrase: String,
    },
    /// Generate a new address
    Address {
        /// Chain to generate address for
        #[arg(short, long, default_value = "bitcell")]
        chain: String,
    },
    /// Show wallet balance
    Balance {
        /// Chain to show balance for
        #[arg(short, long)]
        chain: Option<String>,
    },
    /// Show version information
    Version,
}

fn parse_chain(chain: &str) -> Result<Chain, String> {
    match chain.to_lowercase().as_str() {
        "bitcell" | "cell" => Ok(Chain::BitCell),
        "bitcoin" | "btc" => Ok(Chain::Bitcoin),
        "bitcoin-testnet" | "btc-testnet" => Ok(Chain::BitcoinTestnet),
        "ethereum" | "eth" => Ok(Chain::Ethereum),
        "ethereum-sepolia" | "eth-sepolia" => Ok(Chain::EthereumSepolia),
        _ => Err(format!("Unknown chain: {}", chain)),
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create { name } => {
            println!("💰 BitCell Wallet");
            println!("=================");
            println!();

            let config = WalletConfig {
                name: name.clone(),
                ..WalletConfig::default()
            };

            let (wallet, mnemonic) = Wallet::create_new(config);

            println!("✅ Wallet '{}' created successfully!", name);
            println!();
            println!("⚠️  IMPORTANT: Write down your recovery phrase and store it safely!");
            println!("   Anyone with this phrase can access your funds.");
            println!();
            println!("Recovery Phrase ({} words):", mnemonic.word_count());
            println!("─────────────────────────────────────────────────────────");
            for (i, word) in mnemonic.words().iter().enumerate() {
                print!("{:2}. {:<12}", i + 1, word);
                if (i + 1) % 4 == 0 {
                    println!();
                }
            }
            println!("─────────────────────────────────────────────────────────");
            println!();

            // Show generated addresses
            println!("Generated Addresses:");
            for addr in wallet.all_addresses() {
                println!("  {:?}: {}", addr.chain(), addr.to_string_formatted());
            }
        }
        Commands::Restore {
            mnemonic,
            passphrase,
        } => {
            println!("💰 BitCell Wallet - Restore");
            println!("===========================");
            println!();

            match Mnemonic::from_phrase(&mnemonic) {
                Ok(mnemonic) => {
                    let wallet =
                        Wallet::from_mnemonic(&mnemonic, &passphrase, WalletConfig::default());

                    println!("✅ Wallet restored successfully!");
                    println!();
                    println!("Generated Addresses:");
                    for addr in wallet.all_addresses() {
                        println!("  {:?}: {}", addr.chain(), addr.to_string_formatted());
                    }
                }
                Err(e) => {
                    eprintln!("❌ Error: Invalid mnemonic phrase - {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Address { chain } => {
            match parse_chain(&chain) {
                Ok(chain) => {
                    // For demo purposes, create a temporary wallet
                    let (mut wallet, _) = Wallet::create_new(WalletConfig::default());
                    match wallet.next_address(chain) {
                        Ok(addr) => {
                            println!("New {:?} address: {}", chain, addr.to_string_formatted());
                        }
                        Err(e) => {
                            eprintln!("❌ Error generating address: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Balance { chain } => {
            println!("💰 BitCell Wallet - Balance");
            println!("===========================");
            println!();

            // For demo purposes, show zero balances
            let chains = if let Some(chain_str) = chain {
                match parse_chain(&chain_str) {
                    Ok(c) => vec![c],
                    Err(e) => {
                        eprintln!("❌ Error: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                vec![Chain::BitCell, Chain::Bitcoin, Chain::Ethereum]
            };

            for chain in chains {
                println!("{:?}: 0.00", chain);
            }
            println!();
            println!("Note: Connect to a node to fetch actual balances.");
        }
        Commands::Version => {
            println!("bitcell-wallet v{}", env!("CARGO_PKG_VERSION"));
            println!("BitCell blockchain wallet");
        }
    }
}
