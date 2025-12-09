# Mobile Wallet SDK Integration Guide

This guide provides step-by-step instructions for integrating the BitCell Mobile Wallet SDK into your iOS or Android application.

## Quick Start

### iOS (Swift)

```swift
import BitCellWalletMobile

// 1. Generate a mnemonic
let mnemonic = try generateMnemonic(wordCount: .words12)

// 2. Configure secure storage
let config = SecureStorageConfig(
    useBiometric: true,
    useKeychain: true,
    appIdentifier: "com.yourapp.bitcell"
)

// 3. Create wallet
let wallet = try createWallet(
    mnemonicPhrase: mnemonic,
    storageConfig: config
)

// 4. Unlock wallet
try wallet.unlock(password: "user-password")

// 5. Get address
let address = try wallet.getAddress()
print("Wallet address: \(address)")

// 6. Sign transaction
let txDetails = TransactionDetails(
    fromAddress: address,
    toAddress: "recipient-address",
    amount: "1000000",
    fee: "1000",
    nonce: 0,
    data: nil
)

let signedTx = try wallet.signTransaction(txDetails: txDetails)
print("Transaction hash: \(signedTx.txHash)")
```

### Android (Kotlin)

```kotlin
import com.bitcell.wallet.mobile.*

// 1. Generate a mnemonic
val mnemonic = generateMnemonic(MnemonicWordCount.WORDS12)

// 2. Configure secure storage
val config = SecureStorageConfig(
    useBiometric = true,
    useKeychain = true,
    appIdentifier = "com.yourapp.bitcell"
)

// 3. Create wallet
val wallet = createWallet(
    mnemonicPhrase = mnemonic,
    storageConfig = config
)

// 4. Unlock wallet
wallet.unlock(password = "user-password")

// 5. Get address
val address = wallet.getAddress()
println("Wallet address: $address")

// 6. Sign transaction
val txDetails = TransactionDetails(
    fromAddress = address,
    toAddress = "recipient-address",
    amount = "1000000",
    fee = "1000",
    nonce = 0,
    data = null
)

val signedTx = wallet.signTransaction(txDetails)
println("Transaction hash: ${signedTx.txHash}")
```

## Common Use Cases

### Creating a New Wallet

```swift
// iOS
func createNewWallet() -> MobileWallet? {
    do {
        // Generate mnemonic
        let mnemonic = try generateMnemonic(wordCount: .words12)
        
        // Save mnemonic securely (show to user first!)
        // User should write it down before proceeding
        
        // Create wallet
        let config = SecureStorageConfig(
            useBiometric: true,
            useKeychain: true,
            appIdentifier: Bundle.main.bundleIdentifier ?? "com.bitcell.wallet"
        )
        
        let wallet = try createWallet(
            mnemonicPhrase: mnemonic,
            storageConfig: config
        )
        
        return wallet
    } catch {
        print("Failed to create wallet: \(error)")
        return nil
    }
}
```

```kotlin
// Android
fun createNewWallet(): MobileWallet? {
    return try {
        // Generate mnemonic
        val mnemonic = generateMnemonic(MnemonicWordCount.WORDS12)
        
        // Save mnemonic securely (show to user first!)
        // User should write it down before proceeding
        
        // Create wallet
        val config = SecureStorageConfig(
            useBiometric = true,
            useKeychain = true,
            appIdentifier = context.packageName
        )
        
        createWallet(
            mnemonicPhrase = mnemonic,
            storageConfig = config
        )
    } catch (e: Exception) {
        Log.e("Wallet", "Failed to create wallet", e)
        null
    }
}
```

### Restoring from Backup

```swift
// iOS
func restoreWallet(mnemonic: String) -> MobileWallet? {
    do {
        // Validate mnemonic first
        guard validateMnemonic(mnemonicPhrase: mnemonic) else {
            throw NSError(domain: "Invalid mnemonic", code: -1)
        }
        
        let config = SecureStorageConfig(
            useBiometric: true,
            useKeychain: true,
            appIdentifier: Bundle.main.bundleIdentifier ?? "com.bitcell.wallet"
        )
        
        let wallet = try restoreWallet(
            mnemonicPhrase: mnemonic,
            storageConfig: config
        )
        
        return wallet
    } catch {
        print("Failed to restore wallet: \(error)")
        return nil
    }
}
```

### Biometric Authentication Flow

```swift
// iOS
func unlockWithBiometric() {
    do {
        try wallet.unlockWithBiometric()
        // Wallet unlocked successfully
        showMainScreen()
    } catch {
        // Biometric failed, ask for password
        showPasswordPrompt()
    }
}

func showPasswordPrompt() {
    // Show password input UI
    let password = getUserPassword()
    
    do {
        try wallet.unlock(password: password)
        showMainScreen()
    } catch {
        showError("Invalid password")
    }
}
```

```kotlin
// Android
fun unlockWithBiometric() {
    try {
        wallet.unlockWithBiometric()
        // Wallet unlocked successfully
        showMainScreen()
    } catch (e: Exception) {
        // Biometric failed, ask for password
        showPasswordPrompt()
    }
}

fun showPasswordPrompt() {
    // Show password input UI
    val password = getUserPassword()
    
    try {
        wallet.unlock(password)
        showMainScreen()
    } catch (e: Exception) {
        showError("Invalid password")
    }
}
```

### Creating and Restoring Backups

```swift
// iOS
func createBackup(password: String) -> String? {
    do {
        let backup = try wallet.createBackup(password: password)
        let json = try backup.toJson()
        
        // Save JSON to file or cloud storage
        return json
    } catch {
        print("Failed to create backup: \(error)")
        return nil
    }
}

func restoreFromBackup(json: String, password: String) -> Bool {
    do {
        let backup = try WalletBackup.fromJson(json: json)
        try wallet.restoreFromBackup(backup: backup, password: password)
        return true
    } catch {
        print("Failed to restore backup: \(error)")
        return false
    }
}
```

### Transaction Signing

```swift
// iOS
func sendTransaction(to recipient: String, amount: UInt64) -> String? {
    do {
        // Ensure wallet is unlocked
        guard !wallet.isLocked() else {
            throw NSError(domain: "Wallet is locked", code: -1)
        }
        
        // Get account info for nonce
        let accountInfo = try wallet.getAccountInfo()
        let address = try wallet.getAddress()
        
        // Build transaction details
        let txDetails = TransactionDetails(
            fromAddress: address,
            toAddress: recipient,
            amount: String(amount),
            fee: "1000", // 1000 units
            nonce: accountInfo.nonce,
            data: nil
        )
        
        // Sign transaction
        let signedTx = try wallet.signTransaction(txDetails: txDetails)
        
        // Broadcast signed transaction to network
        broadcastTransaction(signedTx.rawTransaction)
        
        return signedTx.txHash
    } catch {
        print("Failed to sign transaction: \(error)")
        return nil
    }
}
```

## Error Handling

Always handle errors appropriately:

```swift
// iOS
do {
    let wallet = try createWallet(mnemonicPhrase: mnemonic, storageConfig: config)
    // Success
} catch let error as MobileWalletError {
    switch error {
    case .invalidMnemonic:
        print("The mnemonic phrase is invalid")
    case .walletLocked:
        print("Wallet is locked. Please unlock first.")
    case .insufficientBalance:
        print("Insufficient balance for transaction")
    case .biometricError:
        print("Biometric authentication failed")
    default:
        print("An error occurred: \(error)")
    }
}
```

```kotlin
// Android
try {
    val wallet = createWallet(mnemonic, config)
    // Success
} catch (e: MobileWalletException) {
    when (e) {
        is MobileWalletException.InvalidMnemonic -> 
            println("The mnemonic phrase is invalid")
        is MobileWalletException.WalletLocked -> 
            println("Wallet is locked. Please unlock first.")
        is MobileWalletException.InsufficientBalance -> 
            println("Insufficient balance for transaction")
        is MobileWalletException.BiometricError -> 
            println("Biometric authentication failed")
        else -> 
            println("An error occurred: ${e.message}")
    }
}
```

## Best Practices

### 1. Always Show Mnemonic to User

```swift
func onWalletCreated(mnemonic: String) {
    // Show mnemonic in a secure, user-friendly way
    showMnemonicScreen(mnemonic: mnemonic)
    
    // Require user confirmation they've saved it
    requireUserConfirmation {
        // Only proceed after confirmation
        proceedToWallet()
    }
}
```

### 2. Lock Wallet on Background

```swift
// iOS - AppDelegate or SceneDelegate
func sceneDidEnterBackground(_ scene: UIScene) {
    // Lock wallet when app goes to background
    try? wallet?.lock()
}
```

```kotlin
// Android - Activity
override fun onPause() {
    super.onPause()
    // Lock wallet when activity pauses
    wallet?.lock()
}
```

### 3. Validate Before Sending

```swift
func validateTransaction(amount: UInt64, fee: UInt64) -> Bool {
    guard let accountInfo = try? wallet.getAccountInfo() else {
        return false
    }
    
    let balance = UInt64(accountInfo.balance) ?? 0
    let total = amount + fee
    
    return balance >= total
}
```

### 4. Secure Logging

```swift
// Never log sensitive data!
// ❌ Bad
print("Mnemonic: \(mnemonic)")
print("Private key: \(privateKey)")

// ✅ Good
print("Wallet created successfully")
print("Address: \(address)")
```

### 5. Handle Network Errors

```swift
func broadcastTransaction(_ rawTx: String) async throws {
    do {
        let response = try await networkClient.broadcast(rawTx)
        // Success
    } catch {
        // Handle network errors gracefully
        if error is NetworkError {
            throw TransactionError.networkUnavailable
        }
        throw error
    }
}
```

## Platform-Specific Notes

### iOS

1. **Simulator Limitations**:
   - Face ID/Touch ID simulation is limited
   - Test on real devices for biometric features
   - Keychain works in simulator but has fewer protections

2. **Background Execution**:
   - iOS may terminate background tasks
   - Save state before backgrounding
   - Reload on foreground

3. **Memory Management**:
   - Use `deinit` to cleanup resources
   - Be mindful of reference cycles

### Android

1. **API Level Compatibility**:
   - BiometricPrompt requires API 28+
   - Provide fallback for older devices
   - Test on various Android versions

2. **Process Death**:
   - Android may kill app process
   - Persist wallet state appropriately
   - Use ViewModel for state management

3. **Hardware Variations**:
   - Test on devices with/without biometric hardware
   - Test on devices with/without StrongBox
   - Handle hardware variations gracefully

## Testing Checklist

- [ ] Create new wallet
- [ ] Restore from mnemonic
- [ ] Unlock with password
- [ ] Unlock with biometric (if available)
- [ ] Sign transaction
- [ ] Create backup
- [ ] Restore from backup
- [ ] Lock/unlock cycle
- [ ] App backgrounding
- [ ] Network errors
- [ ] Invalid inputs
- [ ] Edge cases (empty balance, max values, etc.)

## Troubleshooting

### Wallet Won't Unlock

1. Check if mnemonic is valid: `validateMnemonic()`
2. Verify storage configuration
3. Check device keychain/keystore status
4. Try deleting and recreating wallet (on test devices only!)

### Biometric Not Working

1. Check if biometric is available: `biometric.isAvailable()`
2. Verify user has enrolled biometric
3. Check permissions (iOS: Info.plist, Android: manifest)
4. Test on real device, not simulator

### Transaction Signing Fails

1. Ensure wallet is unlocked
2. Verify transaction details are valid
3. Check balance is sufficient
4. Verify nonce is correct

## Additional Resources

- [Full API Documentation](./README.md)
- [Security Best Practices](./SECURITY.md)
- [Example Apps](../examples/)
- [Contributing Guide](../../CONTRIBUTING.md)
