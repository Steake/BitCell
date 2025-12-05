//! BitCell Wallet GUI
//!
//! Cross-platform native GUI for the BitCell wallet using Slint.
//! Targets: macOS, Linux, Windows
//! Features: 60fps smooth interactions, accessibility support, no WebView

use bitcell_wallet::{Chain, Mnemonic, Wallet, WalletConfig};
use std::cell::RefCell;
use std::rc::Rc;

slint::include_modules!();

mod rpc_client;
use rpc_client::RpcClient;

mod qrcode;
mod game_viz;

/// Wallet application state
struct AppState {
    wallet: Option<Wallet>,
    mnemonic: Option<Mnemonic>,
    rpc_client: Option<RpcClient>,
}

impl AppState {
    fn new() -> Self {
        Self {
            wallet: None,
            mnemonic: None,
            rpc_client: Some(RpcClient::new("127.0.0.1".to_string(), 30334)),
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
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
    
    // Create RPC client for polling
    let rpc_client = state.borrow().rpc_client.clone().unwrap();
    let main_window_weak = main_window.as_weak();
    
    // Start polling timer for RPC connection status
    let timer = slint::Timer::default();
    timer.start(slint::TimerMode::Repeated, std::time::Duration::from_secs(2), move || {
        let client = rpc_client.clone();
        let window_weak = main_window_weak.clone();
        
        tokio::spawn(async move {
            match client.get_node_info().await {
                Ok(_) => {
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(window) = window_weak.upgrade() {
                            window.global::<WalletState>().set_rpc_connected(true);
                        }
                    });
                }
                Err(e) => {
                    tracing::debug!("RPC connection check failed: {}", e);
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(window) = window_weak.upgrade() {
                            window.global::<WalletState>().set_rpc_connected(false);
                        }
                    });
                }
            }
        });
    });
    
    // Start polling timer for tournament state
    let rpc_client_tournament = state.borrow().rpc_client.clone().unwrap();
    let tournament_window_weak = main_window.as_weak();
    
    let tournament_timer = slint::Timer::default();
    tournament_timer.start(slint::TimerMode::Repeated, std::time::Duration::from_secs(2), move || {
        let client = rpc_client_tournament.clone();
        let window_weak = tournament_window_weak.clone();
        
        tokio::spawn(async move {
            if let Ok(tournament_state) = client.get_tournament_state().await {
                // Parse tournament state JSON
                let phase = tournament_state
                    .get("phase")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string();
                
                let round = tournament_state
                    .get("current_round")
                    .and_then(|v| v.as_u64())
                    .map(|r| r.to_string())
                    .unwrap_or_else(|| "N/A".to_string());
                
                let winner = tournament_state
                    .get("last_winner")
                    .and_then(|v| v.as_str())
                    .unwrap_or("None")
                    .to_string();
                
                // Fetch battle replay if we have a winner
                let current_block = tournament_state
                    .get("current_round")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                
                let mut grid_data = Vec::new();
                let mut width = 0;
                let mut height = 0;
                let mut has_grid = false;
                
                if current_block > 0 {
                    if let Ok(replay) = client.get_battle_replay(current_block).await {
                        if let Some(grids) = replay.get("grid_states").and_then(|v| v.as_array()) {
                            // Take the last frame for now
                            if let Some(last_frame) = grids.last() {
                                if let Some(rows) = last_frame.as_array() {
                                    height = rows.len() as u32;
                                    if height > 0 {
                                        width = rows[0].as_array().map(|r| r.len()).unwrap_or(0) as u32;
                                        
                                        for row in rows {
                                            if let Some(cells) = row.as_array() {
                                                let row_vec: Vec<u8> = cells.iter()
                                                    .map(|c| c.as_u64().unwrap_or(0) as u8)
                                                    .collect();
                                                grid_data.push(row_vec);
                                            }
                                        }
                                        has_grid = true;
                                    }
                                }
                            }
                        }
                    }
                }
                
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(window) = window_weak.upgrade() {
                        let ws = window.global::<WalletState>();
                        ws.set_tournament_phase(phase.into());
                        ws.set_tournament_round(round.into());
                        ws.set_last_winner(winner.into());
                        
                        if has_grid {
                            let grid_image = crate::game_viz::render_grid(&grid_data, width, height);
                            ws.set_game_grid(grid_image);
                        }
                    }
                });
            }
        });
    });
    
    // Run the event loop
    main_window.run()?;
    Ok(())
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
                    Ok(addr) => {
                        let addr_str = addr.to_string_formatted();
                        wallet_state.set_status_message(
                            format!("New {} address generated", chain_display_name(chain)).into()
                        );
                        
                        // Generate QR code
                        let qr_image = qrcode::generate_qr_code(&addr_str);
                        wallet_state.set_qr_code(qr_image);
                        
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
        let state = state.clone();
        let window_weak = window.as_weak();
        
        wallet_state.on_send_transaction(move |to_address, amount_str, chain_str| {
            let window = window_weak.unwrap();
            let wallet_state = window.global::<WalletState>();
            
            // Parse amount (convert from human-readable to smallest units)
            let amount: f64 = amount_str.parse().unwrap_or(0.0);
            if amount <= 0.0 {
                wallet_state.set_status_message("Invalid amount".into());
                return;
            }
            
            if to_address.is_empty() {
                wallet_state.set_status_message("Invalid recipient address".into());
                return;
            }
            
            let chain = parse_chain(&chain_str);
            
            // Convert to smallest units (1 CELL = 100_000_000 units)
            let amount_units = (amount * 100_000_000.0) as u64;
            
            // Get wallet and RPC client
            let app_state = state.borrow();
            
            let (from_address, rpc_client) = {
                let wallet = match &app_state.wallet {
                    Some(w) => w,
                    None => {
                        wallet_state.set_status_message("No wallet loaded".into());
                        return;
                    }
                };
                
                // Get the first address as sender
                let addresses = wallet.all_addresses();
                let from_addr = match addresses.iter().find(|a| a.chain() == chain) {
                    Some(a) => a.to_string_formatted(),
                    None => {
                        wallet_state.set_status_message(format!("No {} address available", chain_display_name(chain)).into());
                        return;
                    }
                };
                
                let rpc = match &app_state.rpc_client {
                    Some(c) => c.clone(),
                    None => {
                        wallet_state.set_status_message("RPC client not initialized".into());
                        return;
                    }
                };
                
                (from_addr, rpc)
            };
            
            // Drop app_state borrow before the async operation
            drop(app_state);
            
            // Set loading state
            wallet_state.set_is_loading(true);
            wallet_state.set_status_message("Preparing transaction...".into());
            
            let window_weak = window.as_weak();
            let to_address = to_address.to_string();
            
            // Async nonce fetch and transaction preparation
            tokio::spawn(async move {
                // Get nonce from node
                let nonce = match rpc_client.get_transaction_count(&from_address).await {
                    Ok(n) => n,
                    Err(e) => {
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(window) = window_weak.upgrade() {
                                let ws = window.global::<WalletState>();
                                ws.set_is_loading(false);
                                ws.set_status_message(format!("Failed to get nonce: {}", e).into());
                            }
                        });
                        return;
                    }
                };
                
                // Get gas price
                let gas_price = match rpc_client.get_gas_price().await {
                    Ok(p) => p,
                    Err(_) => 1000, // Default fee if gas price unavailable
                };
                
                // Calculate fee (simple estimate)
                let fee = gas_price.saturating_mul(21000);
                
                // For now, display transaction details and inform user signing requires wallet unlock
                // In production, this would integrate with hardware wallet or secure key management
                let tx_info = format!(
                    "Transaction prepared:\n\
                     From: {}\n\
                     To: {}\n\
                     Amount: {} units\n\
                     Fee: {} units\n\
                     Nonce: {}\n\n\
                     Hardware wallet signing coming soon. \
                     Use the CLI or Admin console with HSM for secure signing.",
                    from_address, to_address, amount_units, fee, nonce
                );
                
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(window) = window_weak.upgrade() {
                        let ws = window.global::<WalletState>();
                        ws.set_is_loading(false);
                        ws.set_status_message(tx_info.into());
                    }
                });
            });
        });
    }
    
    // Refresh balances callback
    {
        let state = state.clone();
        let window_weak = window.as_weak();
        
        wallet_state.on_refresh_balances(move || {
            let window = window_weak.unwrap();
            let wallet_state = window.global::<WalletState>();
            
            wallet_state.set_is_loading(true);
            
            let app_state = state.borrow();
            if let Some(rpc_client) = &app_state.rpc_client {
                let client = rpc_client.clone();
                let window_weak = window.as_weak();
                
                // Get addresses to refresh
                let addresses: Vec<String> = if let Some(ref wallet) = app_state.wallet {
                    wallet.all_addresses().iter().map(|a| a.to_string_formatted()).collect()
                } else {
                    vec![]
                };
                
                tokio::spawn(async move {
                    // Fetch balances
                    let mut updates = Vec::new();
                    for addr in addresses {
                        if let Ok(balance) = client.get_balance(&addr).await {
                            updates.push((addr, balance));
                        }
                    }
                    
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(window) = window_weak.upgrade() {
                            let wallet_state = window.global::<WalletState>();
                            wallet_state.set_is_loading(false);
                            wallet_state.set_status_message(format!("Updated {} balances", updates.len()).into());
                            // Note: Updating the actual model requires more complex logic to map back to the wallet
                            // For now we just verify connectivity and data fetching works
                        }
                    });
                });
            } else {
                wallet_state.set_is_loading(false);
                wallet_state.set_status_message("RPC client not initialized".into());
            }
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
