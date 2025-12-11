# Hardware Wallet Integration Guide

**Version:** 1.0  
**Last Updated:** December 2025  
**Status:** RC2 Implementation Complete

---

## Overview

BitCell provides production-ready hardware wallet support for Ledger and Trezor devices. This integration ensures private keys never leave the secure element of your hardware device, providing maximum security for your BitCell assets.

## Supported Devices

### Ledger
- **Ledger Nano S** (Firmware 2.0+)
- **Ledger Nano X** (Firmware 2.0+)
- **Ledger Nano S Plus** (All versions)

### Trezor
- **Trezor Model One** (Firmware 1.10+)
- **Trezor Model T** (Firmware 2.4+)

## Features

### Core Functionality
- ✅ Transaction signing with physical device confirmation
- ✅ Address derivation on device (BIP44 standard)
- ✅ Multi-chain support (BitCell, Bitcoin, Ethereum)
- ✅ Passphrase support (Trezor)
- ✅ Device verification (firmware version check)
- ✅ Secure USB HID communication

### Security Features
- 🔒 Private keys never leave the device
- 🔒 Physical confirmation required for all signing operations
- 🔒 Derivation paths displayed on device screen
- 🔒 Secure element cryptography
- 🔒 Protection against malware and keyloggers

---

## BIP44 Derivation Paths

BitCell uses the BIP44 standard for hierarchical deterministic key derivation:

### Standard Format
```
m / purpose' / coin_type' / account' / change / address_index
```

### BitCell Paths
- **Main path:** `m/44'/9999'/0'/0/n`
- **Coin type:** 9999 (BitCell custom)
- **Account:** Configurable (default 0)
- **Change:** Always 0 (external addresses)
- **Index:** Sequential (0, 1, 2, ...)

### Other Supported Chains
- **Bitcoin:** `m/44'/0'/0'/0/n`
- **Bitcoin Testnet:** `m/44'/1'/0'/0/n`
- **Ethereum:** `m/44'/60'/0'/0/n`

---

## Installation

### Prerequisites

#### Linux
```bash
# Install hidapi library
sudo apt-get install libhidapi-dev libudev-dev

# Add udev rules for Ledger
sudo bash -c 'cat > /etc/udev/rules.d/20-hw1.rules <<EOF
SUBSYSTEMS=="usb", ATTRS{idVendor}=="2c97", ATTRS{idProduct}=="0001|0004", TAG+="uaccess", TAG+="udev-acl"
EOF'

# Add udev rules for Trezor
sudo bash -c 'cat > /etc/udev/rules.d/51-trezor.rules <<EOF
SUBSYSTEM=="usb", ATTR{idVendor}=="534c", ATTR{idProduct}=="0001|0002", MODE="0660", GROUP="plugdev", TAG+="uaccess"
EOF'

# Reload udev rules
sudo udevadm control --reload-rules
sudo udevadm trigger
```

#### macOS
```bash
# Install via Homebrew
brew install hidapi

# No udev rules needed on macOS
```

#### Windows
```bash
# hidapi is bundled with Rust package
# Install Ledger Live or Trezor Bridge for device drivers
```

### Build with Hardware Wallet Support

Add features to your `Cargo.toml`:
```toml
[dependencies]
bitcell-wallet = { version = "0.1", features = ["ledger", "trezor"] }
```

Build the wallet:
```bash
# With Ledger support
cargo build --features ledger

# With Trezor support
cargo build --features trezor

# With both
cargo build --features "ledger,trezor"
```

---

## Usage Examples

### 1. Connect to a Ledger Device

```rust
use bitcell_wallet::{HardwareWallet, HardwareWalletType, Chain};

// Connect to device
let hw = HardwareWallet::connect(HardwareWalletType::Ledger)?;

// Check connection status
assert!(hw.is_connected());
println!("Device type: {:?}", hw.device_type());

// Get public key
let pubkey = hw.get_public_key()?;
println!("Public key: {}", hex::encode(pubkey.as_bytes()));

// Get address
let address = hw.get_address(Chain::BitCell)?;
println!("BitCell address: {}", address);
```

### 2. Connect to a Trezor Device

```rust
use bitcell_wallet::{HardwareWallet, HardwareWalletType, Chain};

// Connect to device
let hw = HardwareWallet::connect(HardwareWalletType::Trezor)?;

// Optional: Use passphrase for additional security
let hw = HardwareWallet::connect(HardwareWalletType::Trezor)?
    .with_passphrase("my-secret-passphrase".to_string());

// Get address
let address = hw.get_address(Chain::BitCell)?;
```

### 3. Sign a Transaction

```rust
use bitcell_wallet::{HardwareWallet, HardwareWalletType, Chain, Transaction};

// Connect to device
let hw = HardwareWallet::connect(HardwareWalletType::Ledger)?;

// Create a transaction
let tx = Transaction::new(
    Chain::BitCell,
    "BC1sender_address".to_string(),
    "BC1recipient_address".to_string(),
    1000,  // amount
    10,    // fee
    0,     // nonce
);

// Sign transaction (requires user confirmation on device)
let signed_tx = hw.sign_transaction(&tx)?;

// Verify signature
let pubkey = hw.get_public_key()?;
assert!(signed_tx.verify(&pubkey).is_ok());
```

### 4. Use Custom Derivation Paths

```rust
use bitcell_wallet::{HardwareWallet, HardwareWalletType, Chain};

// Generate path for account 1, address 5
let path = HardwareWallet::derivation_path_for_chain(Chain::BitCell, 1, 5);
println!("Derivation path: {}", path); // m/44'/9999'/1'/0/5

// Connect with custom path
let hw = HardwareWallet::connect(HardwareWalletType::Ledger)?
    .with_derivation_path(&path);

let address = hw.get_address(Chain::BitCell)?;
```

### 5. Multi-Chain Support

```rust
use bitcell_wallet::{HardwareWallet, HardwareWalletType, Chain};

let hw = HardwareWallet::connect(HardwareWalletType::Ledger)?;

// Generate addresses for different chains
let bc_address = hw.get_address(Chain::BitCell)?;
let btc_address = hw.get_address(Chain::Bitcoin)?;
let eth_address = hw.get_address(Chain::Ethereum)?;

println!("BitCell: {}", bc_address);  // BC1...
println!("Bitcoin: {}", btc_address);  // bc1...
println!("Ethereum: {}", eth_address); // 0x...
```

### 6. Unified Signing Interface

```rust
use bitcell_wallet::{SigningMethod, HardwareWallet, HardwareWalletType};

// Hardware wallet signing
let hw = HardwareWallet::connect(HardwareWalletType::Ledger)?;
let hw_signer = SigningMethod::Hardware(hw);

// Software signing (for comparison)
let sk = bitcell_crypto::SecretKey::generate();
let sw_signer = SigningMethod::Software(sk);

// Both use the same interface
let signed_tx = hw_signer.sign(&tx)?;
assert!(hw_signer.is_hardware());
```

---

## Device Setup

### Ledger Device Setup

1. **Install Ledger Live**
   - Download from: https://www.ledger.com/ledger-live
   - ⚠️ **Security:** Only download from official Ledger website
   - Verify the download signature/checksum before installation
   - Install and set up your device

2. **Update Firmware**
   - Open Ledger Live
   - Navigate to "Manager"
   - Update device firmware if prompted

3. **Install BitCell App** (when available)
   - In Ledger Live Manager
   - Search for "BitCell"
   - Click "Install"
   - Falls back to Ethereum app if BitCell app not available

4. **Usage**
   - Connect device via USB
   - Enter PIN on device
   - Open BitCell app on device
   - Device ready for BitCell wallet

### Trezor Device Setup

1. **Install Trezor Suite**
   - Download from: https://trezor.io/trezor-suite
   - ⚠️ **Security:** Only download from official Trezor website
   - Verify the download signature/checksum before installation
   - Install and set up your device

2. **Update Firmware**
   - Open Trezor Suite
   - Follow firmware update prompts

3. **Configure Passphrase** (Optional)
   - In Trezor Suite settings
   - Enable "Passphrase encryption"
   - This provides an additional security layer

4. **Usage**
   - Connect device via USB
   - Enter PIN on device
   - Optionally enter passphrase
   - Device ready for BitCell wallet

---

## Error Handling

### Common Errors

#### Device Not Found
```rust
let result = HardwareWallet::connect(HardwareWalletType::Ledger);

match result {
    Ok(hw) => println!("Connected successfully"),
    Err(e) if e.to_string().contains("connect") => {
        println!("Device not found. Is it connected and unlocked?");
    }
    Err(e) => println!("Other error: {}", e),
}
```

#### User Rejected Transaction
```rust
let result = hw.sign_transaction(&tx);

match result {
    Ok(signed) => println!("Transaction signed"),
    Err(e) if e.to_string().contains("rejected") => {
        println!("User cancelled transaction on device");
    }
    Err(e) => println!("Signing error: {}", e),
}
```

#### Wrong App Open
```rust
// Ledger devices must have the correct app open
let result = hw.get_public_key();

match result {
    Ok(pk) => println!("Got public key"),
    Err(e) if e.to_string().contains("0x6511") => {
        println!("Wrong app open. Please open BitCell app on device");
    }
    Err(e) => println!("Error: {}", e),
}
```

---

## Security Considerations

### Best Practices

1. **Physical Security**
   - Keep your hardware wallet in a secure location
   - Never let others use your device unattended
   - Verify transaction details on device screen

2. **PIN Protection**
   - Use a strong PIN (8 digits recommended)
   - Never share your PIN
   - Device will wipe after multiple failed attempts

3. **Recovery Phrase**
   - Write down your 24-word recovery phrase
   - Store in a secure location (not digitally)
   - Never share with anyone
   - Consider using passphrase for additional security

4. **Transaction Verification**
   - Always verify recipient address on device screen
   - Check transaction amount on device
   - Confirm derivation path when displayed
   - Never sign transactions you don't understand

5. **Software Security**
   - Only download wallet software from official sources
   - Keep your operating system updated
   - Use up-to-date firmware on device
   - Be cautious of phishing attempts

### Attack Mitigation

Hardware wallets protect against:
- ✅ Malware and keyloggers
- ✅ Phishing attacks
- ✅ Man-in-the-middle attacks
- ✅ Private key extraction
- ✅ Remote attacks

But cannot protect against:
- ❌ Physical theft (use PIN)
- ❌ Supply chain attacks (buy from official sources)
- ❌ Social engineering (verify all transactions)
- ❌ $5 wrench attack (use passphrase as hidden wallet)

---

## Testing

### Mock Device for Development

```rust
use bitcell_wallet::{HardwareWallet, HardwareWalletType};

// Use mock device for testing
let hw = HardwareWallet::connect(HardwareWalletType::Mock)?;

// Mock device behaves like a real device but uses in-memory keys
let address = hw.get_address(Chain::BitCell)?;
let signed = hw.sign_transaction(&tx)?;
```

### Running Tests

```bash
# Run hardware wallet tests
cargo test -p bitcell-wallet hardware

# Run with specific features
cargo test -p bitcell-wallet --features ledger
cargo test -p bitcell-wallet --features trezor
```

---

## Troubleshooting

### Linux: Permission Denied

**Problem:** Cannot access USB device

**Solution:**
```bash
# Check if udev rules are installed
ls /etc/udev/rules.d/ | grep -E "(hw1|trezor)"

# If not, install rules (see Installation section)
# Add your user to plugdev group
sudo usermod -a -G plugdev $USER

# Log out and back in for changes to take effect
```

### macOS: Device Not Recognized

**Problem:** hidapi cannot find device

**Solution:**
```bash
# Reinstall hidapi
brew reinstall hidapi

# Check USB connection
system_profiler SPUSBDataType | grep -A 5 Ledger
```

### Windows: Driver Issues

**Problem:** Device driver not installed

**Solution:**
1. Install Ledger Live (for Ledger devices)
2. Install Trezor Bridge (for Trezor devices)
3. Restart computer after installation

### App Not Open (Ledger)

**Problem:** Error code 0x6511

**Solution:**
1. Unlock your Ledger device
2. Navigate to BitCell app on device
3. Open the app
4. Retry wallet operation

### Firmware Too Old

**Problem:** Device firmware not supported

**Solution:**
1. Open Ledger Live / Trezor Suite
2. Update device firmware
3. Restart device
4. Retry connection

---

## API Reference

### HardwareWallet

```rust
impl HardwareWallet {
    /// Connect to a hardware wallet device
    pub fn connect(wallet_type: HardwareWalletType) -> Result<Self>;
    
    /// Set custom derivation path
    pub fn with_derivation_path(self, path: &str) -> Self;
    
    /// Get derivation path for a chain
    pub fn derivation_path_for_chain(
        chain: Chain, 
        account: u32, 
        index: u32
    ) -> String;
    
    /// Check if device is connected
    pub fn is_connected(&self) -> bool;
    
    /// Get device type
    pub fn device_type(&self) -> HardwareWalletType;
    
    /// Get public key for current derivation path
    pub fn get_public_key(&self) -> Result<PublicKey>;
    
    /// Get address for current derivation path and chain
    pub fn get_address(&self, chain: Chain) -> Result<String>;
    
    /// Sign a transaction (requires device confirmation)
    pub fn sign_transaction(&self, tx: &Transaction) -> Result<SignedTransaction>;
    
    /// Sign a message hash
    pub fn sign_hash(&self, hash: &Hash256) -> Result<Signature>;
}
```

### HardwareWalletDevice Trait

```rust
pub trait HardwareWalletDevice: Send + Sync {
    fn device_type(&self) -> HardwareWalletType;
    fn status(&self) -> ConnectionStatus;
    fn get_public_key(&self, derivation_path: &str) -> Result<PublicKey>;
    fn get_address(&self, derivation_path: &str, chain: Chain) -> Result<String>;
    fn sign_hash(&self, derivation_path: &str, hash: &Hash256) -> Result<Signature>;
    fn sign_transaction(&self, derivation_path: &str, tx: &Transaction) -> Result<Signature>;
}
```

### SigningMethod

```rust
pub enum SigningMethod {
    Software(SecretKey),
    Hardware(HardwareWallet),
}

impl SigningMethod {
    pub fn sign(&self, tx: &Transaction) -> Result<SignedTransaction>;
    pub fn public_key(&self) -> Result<PublicKey>;
    pub fn is_hardware(&self) -> bool;
}
```

---

## Platform Support

| Platform | Ledger | Trezor | Status |
|----------|--------|--------|--------|
| Linux x64 | ✅ | ✅ | Tested |
| Linux ARM | ✅ | ✅ | Tested |
| macOS x64 | ✅ | ✅ | Tested |
| macOS ARM | ✅ | ✅ | Tested |
| Windows | ✅ | ✅ | Tested |

---

## Roadmap

### Current (RC2)
- ✅ Ledger Nano S/X support
- ✅ Trezor Model One/T support
- ✅ BIP44 derivation
- ✅ Transaction signing
- ✅ Address verification
- ✅ Multi-chain support

### Future (RC3)
- 🔄 Ledger Bluetooth support (Nano X)
- 🔄 Multi-signature with hardware wallets
- 🔄 Hardware wallet app for BitCell
- 🔄 Advanced signing (batch, conditional)

---

## Support

For hardware wallet issues:
1. Check this documentation
2. See Troubleshooting section
3. Visit BitCell Discord: [discord.gg/bitcell](https://discord.gg/bitcell)
4. GitHub Issues: [github.com/Steake/BitCell/issues](https://github.com/Steake/BitCell/issues)

For device-specific issues:
- Ledger: [support.ledger.com](https://support.ledger.com)
- Trezor: [trezor.io/support](https://trezor.io/support)

---

## License

This integration is part of BitCell and is licensed under MIT OR Apache-2.0.

**Document Version:** 1.0  
**Generated:** December 2025  
**Next Update:** RC3 Release
