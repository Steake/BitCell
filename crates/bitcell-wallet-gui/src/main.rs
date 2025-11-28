//! BitCell Wallet GUI
//!
//! Cross-platform native GUI for the BitCell wallet using Slint.
//! Targets: macOS, Linux, Windows
//! Features: 60fps smooth interactions, accessibility support, no WebView

use bitcell_wallet::{Chain, Mnemonic, Wallet, WalletConfig};
use std::cell::RefCell;
use std::rc::Rc;

slint::include_modules!();

/// Wallet application state
struct AppState {
    wallet: Option<Wallet>,
    mnemonic: Option<Mnemonic>,
}

impl AppState {
    fn new() -> Self {
        Self {
            wallet: None,
            mnemonic: None,
        }
    }
}

/// Convert chain string to Chain enum
fn parse_chain(chain: &str) -> Chain {
    match chain.to_lowercase().as_str() {
        "bitcoin" | "btc" => Chain::Bitcoin,
        "ethereum" | "eth" => Chain::Ethereum,
        _ => Chain::BitCell,
    }
}

/// Format chain for display
fn chain_display_name(chain: Chain) -> &'static str {
    match chain {
        Chain::BitCell => "BitCell",
        Chain::Bitcoin => "Bitcoin",
        Chain::BitcoinTestnet => "Bitcoin Testnet",
        Chain::Ethereum => "Ethereum",
        Chain::EthereumSepolia => "Ethereum Sepolia",
        Chain::Custom(_) => "Custom",
    }
}

fn main() -> Result<(), slint::PlatformError> {
    // Create the main window
    let main_window = MainWindow::new()?;
    
    // Create shared application state
    let state = Rc::new(RefCell::new(AppState::new()));
    
    // Get global wallet state handle
    let wallet_state = main_window.global::<WalletState>();
    
    // Setup callback handlers
    setup_callbacks(&main_window, state.clone());
    
    // Initialize with welcome view
    wallet_state.set_current_tab(0);
    wallet_state.set_wallet_exists(false);
    wallet_state.set_wallet_locked(true);
    
    // Run the event loop
    main_window.run()
}

/// Setup all callback handlers for the UI
fn setup_callbacks(window: &MainWindow, state: Rc<RefCell<AppState>>) {
    let wallet_state = window.global::<WalletState>();
    
    // Create wallet callback
    {
        let state = state.clone();
        let window_weak = window.as_weak();
        
        wallet_state.on_create_wallet(move || {
            let window = window_weak.unwrap();
            let wallet_state = window.global::<WalletState>();
            
            // Create new wallet
            let config = WalletConfig {
                name: "BitCell Wallet".to_string(),
                ..WalletConfig::default()
            };
            
            let (wallet, mnemonic) = Wallet::create_new(config);
            
            // Format mnemonic for display
            let mnemonic_words: Vec<&str> = mnemonic.words();
            let mnemonic_display = mnemonic_words
                .chunks(4)
                .enumerate()
                .map(|(row, chunk)| {
                    chunk
                        .iter()
                        .enumerate()
                        .map(|(col, word)| format!("{}. {}", row * 4 + col + 1, word))
                        .collect::<Vec<_>>()
                        .join("   ")
                })
                .collect::<Vec<_>>()
                .join("\n");
            
            // Update state
            {
                let mut app_state = state.borrow_mut();
                app_state.wallet = Some(wallet);
                app_state.mnemonic = Some(mnemonic);
            }
            
            // Update UI
            wallet_state.set_mnemonic_display(mnemonic_display.into());
            wallet_state.set_show_mnemonic(true);
            wallet_state.set_current_tab(2);
            wallet_state.set_wallet_exists(true);
            wallet_state.set_wallet_locked(false);
            wallet_state.set_status_message("Wallet created successfully!".into());
            
            // Update addresses
            update_addresses(&wallet_state, &state);
        });
    }
    
    // Restore wallet callback
    {
        let state = state.clone();
        let window_weak = window.as_weak();
        
        wallet_state.on_restore_wallet(move |mnemonic_str, passphrase| {
            let window = window_weak.unwrap();
            let wallet_state = window.global::<WalletState>();
            
            match Mnemonic::from_phrase(&mnemonic_str) {
                Ok(mnemonic) => {
                    let wallet = Wallet::from_mnemonic(
                        &mnemonic,
                        &passphrase,
                        WalletConfig::default(),
                    );
                    
                    // Update state
                    {
                        let mut app_state = state.borrow_mut();
                        app_state.wallet = Some(wallet);
                        app_state.mnemonic = Some(mnemonic);
                    }
                    
                    // Update UI
                    wallet_state.set_wallet_exists(true);
                    wallet_state.set_wallet_locked(false);
                    wallet_state.set_current_tab(3);
                    wallet_state.set_status_message("Wallet restored successfully!".into());
                    
                    // Update addresses
                    update_addresses(&wallet_state, &state);
                }
                Err(e) => {
                    wallet_state.set_status_message(format!("Error: {}", e).into());
                }
            }
        });
    }
    
    // Lock wallet callback
    {
        let state = state.clone();
        let window_weak = window.as_weak();
        
        wallet_state.on_lock_wallet(move || {
            let window = window_weak.unwrap();
            let wallet_state = window.global::<WalletState>();
            
            if let Some(ref mut wallet) = state.borrow_mut().wallet {
                wallet.lock();
            }
            
            wallet_state.set_wallet_locked(true);
            wallet_state.set_status_message("Wallet locked".into());
        });
    }
    
    // Unlock wallet callback
    {
        let state = state.clone();
        let window_weak = window.as_weak();
        
        wallet_state.on_unlock_wallet(move |passphrase| {
            let window = window_weak.unwrap();
            let wallet_state = window.global::<WalletState>();
            
            let mut app_state = state.borrow_mut();
            
            // Clone mnemonic to avoid borrowing issues
            let mnemonic_clone = app_state.mnemonic.clone();
            
            if let (Some(ref mut wallet), Some(ref mnemonic)) = 
                (&mut app_state.wallet, &mnemonic_clone) 
            {
                match wallet.unlock(mnemonic, &passphrase) {
                    Ok(()) => {
                        wallet_state.set_wallet_locked(false);
                        wallet_state.set_current_tab(3);
                        wallet_state.set_status_message("Wallet unlocked".into());
                    }
                    Err(e) => {
                        wallet_state.set_status_message(format!("Error: {}", e).into());
                    }
                }
            }
        });
    }
    
    // Generate address callback
    {
        let state = state.clone();
        let window_weak = window.as_weak();
        
        wallet_state.on_generate_address(move |chain_str| {
            let window = window_weak.unwrap();
            let wallet_state = window.global::<WalletState>();
            
            let chain = parse_chain(&chain_str);
            
            if let Some(ref mut wallet) = state.borrow_mut().wallet {
                match wallet.next_address(chain) {
                    Ok(_addr) => {
                        wallet_state.set_status_message(
                            format!("New {} address generated", chain_display_name(chain)).into()
                        );
                        update_addresses(&wallet_state, &state);
                    }
                    Err(e) => {
                        wallet_state.set_status_message(format!("Error: {}", e).into());
                    }
                }
            }
        });
    }
    
    // Send transaction callback
    {
        let _state = state.clone();
        let window_weak = window.as_weak();
        
        wallet_state.on_send_transaction(move |to_address, amount, chain_str| {
            let window = window_weak.unwrap();
            let wallet_state = window.global::<WalletState>();
            
            // Parse amount
            let amount: f64 = amount.parse().unwrap_or(0.0);
            if amount <= 0.0 {
                wallet_state.set_status_message("Invalid amount".into());
                return;
            }
            
            if to_address.is_empty() {
                wallet_state.set_status_message("Invalid recipient address".into());
                return;
            }
            
            // TODO: Implement actual transaction sending
            // This is a placeholder that will be implemented when network integration is complete
            // For now, show a message indicating the feature is not yet available
            
            wallet_state.set_status_message(
                format!("Transaction prepared (offline): {} {} to {} - Connect to node to broadcast", 
                    amount, chain_str, 
                    if to_address.len() > 16 {
                        format!("{}...", &to_address[..16])
                    } else {
                        to_address.to_string()
                    }
                ).into()
            );
            wallet_state.set_current_tab(3);
        });
    }
    
    // Refresh balances callback
    {
        let window_weak = window.as_weak();
        
        wallet_state.on_refresh_balances(move || {
            let window = window_weak.unwrap();
            let wallet_state = window.global::<WalletState>();
            
            // In a real implementation, this would fetch balances from nodes
            wallet_state.set_status_message("Balances refreshed".into());
        });
    }
    
    // Copy to clipboard callback
    {
        let window_weak = window.as_weak();
        
        wallet_state.on_copy_to_clipboard(move |text| {
            let window = window_weak.unwrap();
            let wallet_state = window.global::<WalletState>();
            
            // Platform-specific clipboard handling
            #[cfg(target_os = "linux")]
            {
                if let Ok(mut child) = std::process::Command::new("xclip")
                    .args(["-selection", "clipboard"])
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                {
                    use std::io::Write;
                    if let Some(ref mut stdin) = child.stdin {
                        let _ = stdin.write_all(text.as_bytes());
                    }
                }
            }
            
            #[cfg(target_os = "macos")]
            {
                if let Ok(mut child) = std::process::Command::new("pbcopy")
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                {
                    use std::io::Write;
                    if let Some(ref mut stdin) = child.stdin {
                        let _ = stdin.write_all(text.as_bytes());
                    }
                }
            }
            
            #[cfg(target_os = "windows")]
            {
                // Windows clipboard via PowerShell using stdin to avoid injection
                if let Ok(mut child) = std::process::Command::new("powershell")
                    .args(["-Command", "Set-Clipboard -Value $input"])
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                {
                    use std::io::Write;
                    if let Some(ref mut stdin) = child.stdin {
                        let _ = stdin.write_all(text.as_bytes());
                    }
                }
            }
            
            wallet_state.set_status_message("Copied to clipboard".into());
        });
    }
}

/// Update addresses in the UI from wallet state
fn update_addresses(wallet_state: &WalletState, state: &Rc<RefCell<AppState>>) {
    let app_state = state.borrow();
    
    if let Some(ref wallet) = app_state.wallet {
        let addresses: Vec<WalletAddress> = wallet
            .all_addresses()
            .iter()
            .map(|addr| {
                let balance = wallet.get_balance(addr);
                WalletAddress {
                    chain: chain_display_name(addr.chain()).into(),
                    address: addr.to_string_formatted().into(),
                    balance: format!("{:.8}", balance.amount() as f64 / 100_000_000.0).into(),
                }
            })
            .collect();
        
        let model = std::rc::Rc::new(slint::VecModel::from(addresses));
        wallet_state.set_addresses(model.into());
    }
}
