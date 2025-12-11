# BitCell Mobile Wallet SDK

Cross-platform mobile wallet SDK for iOS and Android with secure key management, biometric authentication, and backup/restore functionality.

## Overview

The BitCell Mobile Wallet SDK provides a secure, user-friendly wallet solution for mobile applications. It leverages platform-specific secure storage (iOS Keychain, Android Keystore) and biometric authentication to protect user funds.

## Features

- ✅ **BIP39 Mnemonic Support**: Generate and restore wallets from 12/18/24-word seed phrases
- ✅ **Secure Key Storage**: Platform-specific secure storage (iOS Keychain / Android Keystore)
- ✅ **Biometric Authentication**: Face ID, Touch ID on iOS; BiometricPrompt on Android
- ✅ **Transaction Signing**: Sign transactions securely on-device
- ✅ **Backup & Restore**: Encrypted wallet backups with password protection
- ✅ **Cross-Platform**: Single Rust codebase with FFI bindings for iOS (Swift) and Android (Kotlin)

## Architecture

```
┌─────────────────────────────────────────┐
│  Mobile App (Swift/Kotlin)              │
├─────────────────────────────────────────┤
│  Generated FFI Bindings (UniFFI)        │
├─────────────────────────────────────────┤
│  bitcell-wallet-mobile (Rust)           │
│  ├─ Core Wallet Logic                   │
│  ├─ Secure Storage Abstraction          │
│  ├─ Biometric Authentication            │
│  └─ Backup/Restore                      │
├─────────────────────────────────────────┤
│  bitcell-wallet (Core)                  │
├─────────────────────────────────────────┤
│  Platform Native APIs                   │
│  ├─ iOS: Keychain, LocalAuthentication  │
│  └─ Android: Keystore, BiometricPrompt  │
└─────────────────────────────────────────┘
```

## Building

### Prerequisites

- Rust 1.82+
- iOS: Xcode 15+ with command line tools
- Android: Android SDK 24+ (NDK 25+)

### Build the Core Library

```bash
cd crates/bitcell-wallet-mobile
cargo build --release
```

### Generate FFI Bindings

#### iOS (Swift)

```bash
# Install cargo-swift if not already installed
cargo install cargo-swift

# Generate Swift bindings
cargo run --features uniffi/cli --bin uniffi-bindgen generate \
  src/bitcell_wallet_mobile.udl \
  --language swift \
  --out-dir ios/
```

This generates:
- `bitcell_wallet_mobileFFI.h` - C header file
- `bitcell_wallet_mobile.swift` - Swift wrapper
- `libbitcell_wallet_mobile.a` - Static library

#### Android (Kotlin)

```bash
# Generate Kotlin bindings
cargo run --features uniffi/cli --bin uniffi-bindgen generate \
  src/bitcell_wallet_mobile.udl \
  --language kotlin \
  --out-dir android/

# Build for Android targets
cargo install cargo-ndk
cargo ndk -t armeabi-v7a -t arm64-v8a -t x86 -t x86_64 build --release
```

This generates:
- `bitcell_wallet_mobile.kt` - Kotlin wrapper
- `libbitcell_wallet_mobile.so` - Native library (per architecture)

## Platform Integration

### iOS Integration

1. **Add the SDK to your Xcode project**:
   - Drag `libbitcell_wallet_mobile.a` to your project
   - Add `bitcell_wallet_mobileFFI.h` to bridging header
   - Add `bitcell_wallet_mobile.swift` to your project

2. **Configure Info.plist**:
   ```xml
   <key>NSFaceIDUsageDescription</key>
   <string>Unlock your BitCell wallet with Face ID</string>
   ```

3. **Implement Keychain Storage**:

```swift
import Security
import LocalAuthentication

class IosSecureStorage: SecureKeyStorage {
    func storeKey(keyId: String, keyData: [UInt8]) throws {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrAccount as String: keyId,
            kSecValueData as String: Data(keyData),
            kSecAttrAccessible as String: kSecAttrAccessibleWhenUnlockedThisDeviceOnly
        ]
        
        SecItemDelete(query as CFDictionary)
        let status = SecItemAdd(query as CFDictionary, nil)
        
        if status != errSecSuccess {
            throw MobileWalletError.StorageError
        }
    }
    
    func retrieveKey(keyId: String) throws -> [UInt8] {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrAccount as String: keyId,
            kSecReturnData as String: true
        ]
        
        var result: AnyObject?
        let status = SecItemCopyMatching(query as CFDictionary, &result)
        
        guard status == errSecSuccess,
              let data = result as? Data else {
            throw MobileWalletError.StorageError
        }
        
        return [UInt8](data)
    }
    
    // Implement remaining methods...
}
```

4. **Implement Biometric Authentication**:

```swift
import LocalAuthentication

class IosBiometricProvider: BiometricAuthProvider {
    func authenticate(promptMessage: String) -> BiometricResult {
        let context = LAContext()
        var error: NSError?
        
        guard context.canEvaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, error: &error) else {
            return .notAvailable
        }
        
        var result: BiometricResult = .failed
        let semaphore = DispatchSemaphore(value: 0)
        
        context.evaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, 
                              localizedReason: promptMessage) { success, error in
            if success {
                result = .success
            } else if let error = error as? LAError {
                result = error.code == .userCancel ? .cancelled : .failed
            }
            semaphore.signal()
        }
        
        semaphore.wait()
        return result
    }
    
    func isAvailable() -> Bool {
        LAContext().canEvaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, error: nil)
    }
    
    func isEnrolled() -> Bool {
        isAvailable()
    }
}
```

5. **Create a Wallet**:

```swift
let config = SecureStorageConfig(
    useBiometric: true,
    useKeychain: true,
    appIdentifier: "com.yourapp.bitcell"
)

do {
    let mnemonic = try generateMnemonic(wordCount: .words12)
    let wallet = try createWallet(mnemonicPhrase: mnemonic, storageConfig: config)
    
    // Wallet is created and locked
    print("Wallet created: \(try wallet.getAddress())")
} catch {
    print("Error: \(error)")
}
```

### Android Integration

1. **Add the SDK to your Android project**:

In `app/build.gradle`:
```gradle
android {
    sourceSets {
        main {
            jniLibs.srcDirs = ['src/main/jniLibs']
        }
    }
}

dependencies {
    implementation files('libs/bitcell-wallet-mobile.jar')
}
```

2. **Place native libraries**:
   - `src/main/jniLibs/armeabi-v7a/libbitcell_wallet_mobile.so`
   - `src/main/jniLibs/arm64-v8a/libbitcell_wallet_mobile.so`
   - `src/main/jniLibs/x86/libbitcell_wallet_mobile.so`
   - `src/main/jniLibs/x86_64/libbitcell_wallet_mobile.so`

3. **Add permissions in AndroidManifest.xml**:
```xml
<uses-permission android:name="android.permission.USE_BIOMETRIC" />
<uses-permission android:name="android.permission.INTERNET" />
```

4. **Implement Keystore Storage**:

```kotlin
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey

class AndroidSecureStorage(private val context: Context) : SecureKeyStorage {
    private val keyStore = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
    
    override fun storeKey(keyId: String, keyData: List<UByte>) {
        val prefs = context.getSharedPreferences("bitcell_wallet", Context.MODE_PRIVATE)
        
        // In production: Encrypt keyData with hardware-backed key
        val encrypted = encryptData(keyId, keyData.toByteArray())
        
        prefs.edit()
            .putString(keyId, Base64.encodeToString(encrypted, Base64.DEFAULT))
            .apply()
    }
    
    override fun retrieveKey(keyId: String): List<UByte> {
        val prefs = context.getSharedPreferences("bitcell_wallet", Context.MODE_PRIVATE)
        val encrypted = prefs.getString(keyId, null)
            ?: throw Exception("Key not found")
        
        val decrypted = decryptData(keyId, Base64.decode(encrypted, Base64.DEFAULT))
        return decrypted.toList().map { it.toUByte() }
    }
    
    private fun getOrCreateKey(keyId: String): SecretKey {
        if (!keyStore.containsAlias(keyId)) {
            val keyGen = KeyGenerator.getInstance(
                KeyProperties.KEY_ALGORITHM_AES,
                "AndroidKeyStore"
            )
            
            val spec = KeyGenParameterSpec.Builder(
                keyId,
                KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT
            )
                .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
                .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
                .setUserAuthenticationRequired(false)
                .build()
            
            keyGen.init(spec)
            return keyGen.generateKey()
        }
        
        return keyStore.getKey(keyId, null) as SecretKey
    }
    
    // Implement encryption/decryption methods...
}
```

5. **Implement Biometric Authentication**:

```kotlin
import androidx.biometric.BiometricPrompt
import androidx.biometric.BiometricManager
import androidx.fragment.app.FragmentActivity

class AndroidBiometricProvider(
    private val activity: FragmentActivity
) : BiometricAuthProvider {
    
    override fun authenticate(promptMessage: String): BiometricResult {
        val biometricManager = BiometricManager.from(activity)
        
        when (biometricManager.canAuthenticate(BiometricManager.Authenticators.BIOMETRIC_STRONG)) {
            BiometricManager.BIOMETRIC_SUCCESS -> {
                var result = BiometricResult.FAILED
                val latch = CountDownLatch(1)
                
                val promptInfo = BiometricPrompt.PromptInfo.Builder()
                    .setTitle("BitCell Wallet")
                    .setSubtitle(promptMessage)
                    .setNegativeButtonText("Cancel")
                    .build()
                
                val callback = object : BiometricPrompt.AuthenticationCallback() {
                    override fun onAuthenticationSucceeded(
                        result: BiometricPrompt.AuthenticationResult
                    ) {
                        result = BiometricResult.SUCCESS
                        latch.countDown()
                    }
                    
                    override fun onAuthenticationFailed() {
                        result = BiometricResult.FAILED
                        latch.countDown()
                    }
                    
                    override fun onAuthenticationError(errorCode: Int, errString: CharSequence) {
                        result = if (errorCode == BiometricPrompt.ERROR_USER_CANCELED) {
                            BiometricResult.CANCELLED
                        } else {
                            BiometricResult.FAILED
                        }
                        latch.countDown()
                    }
                }
                
                val prompt = BiometricPrompt(activity, callback)
                prompt.authenticate(promptInfo)
                
                latch.await()
                return result
            }
            BiometricManager.BIOMETRIC_ERROR_NONE_ENROLLED -> return BiometricResult.NOT_ENROLLED
            else -> return BiometricResult.NOT_AVAILABLE
        }
    }
    
    override fun isAvailable(): Boolean {
        val manager = BiometricManager.from(activity)
        return manager.canAuthenticate(BiometricManager.Authenticators.BIOMETRIC_STRONG) == 
               BiometricManager.BIOMETRIC_SUCCESS
    }
    
    override fun isEnrolled(): Boolean = isAvailable()
}
```

6. **Create a Wallet**:

```kotlin
val config = SecureStorageConfig(
    useBiometric = true,
    useKeychain = true,
    appIdentifier = "com.yourapp.bitcell"
)

try {
    val mnemonic = generateMnemonic(MnemonicWordCount.WORDS12)
    val wallet = createWallet(mnemonic, config)
    
    // Wallet is created and locked
    println("Wallet created: ${wallet.getAddress()}")
} catch (e: MobileWalletException) {
    println("Error: ${e.message}")
}
```

## API Reference

### Core Functions

#### `generateMnemonic(wordCount: MnemonicWordCount) -> String`
Generate a new BIP39 mnemonic phrase.

**Parameters:**
- `wordCount`: Number of words (12, 18, or 24)

**Returns:** Mnemonic phrase as a string

#### `validateMnemonic(mnemonicPhrase: String) -> Boolean`
Validate a mnemonic phrase.

#### `createWallet(mnemonicPhrase: String, storageConfig: SecureStorageConfig) -> MobileWallet`
Create a new wallet from a mnemonic.

#### `restoreWallet(mnemonicPhrase: String, storageConfig: SecureStorageConfig) -> MobileWallet`
Restore a wallet from a mnemonic.

### MobileWallet Class

#### Wallet Management

- `lock()`: Lock the wallet
- `unlock(password: String)`: Unlock with password
- `unlockWithBiometric()`: Unlock with biometric authentication
- `getLockState() -> WalletLockState`: Get current lock state
- `isLocked() -> Boolean`: Check if wallet is locked

#### Account Operations

- `getAccountInfo() -> AccountInfo`: Get account information
- `getAddress() -> String`: Get wallet address
- `getPublicKey() -> String`: Get public key (hex encoded)

#### Transactions

- `signTransaction(txDetails: TransactionDetails) -> SignedTransactionResult`: Sign a transaction
- `signMessage(message: String) -> String`: Sign an arbitrary message

#### Backup & Restore

- `createBackup(password: String) -> WalletBackup`: Create encrypted backup
- `restoreFromBackup(backup: WalletBackup, password: String)`: Restore from backup
- `exportMnemonic(password: String) -> String`: Export mnemonic (requires password)

#### Security

- `changePassword(oldPassword: String, newPassword: String)`: Change wallet password
- `enableBiometric(enable: Boolean)`: Enable/disable biometric authentication
- `isBiometricEnabled() -> Boolean`: Check if biometric is enabled

#### Utilities

- `getWalletVersion() -> String`: Get SDK version
- `clearSecureStorage()`: Clear all secure storage (dangerous!)

## Security Considerations

### Key Storage

1. **iOS Keychain**:
   - Uses `kSecAttrAccessibleWhenUnlockedThisDeviceOnly` for maximum security
   - Keys never leave the device (no iCloud sync)
   - Bound to device hardware

2. **Android Keystore**:
   - Uses hardware-backed keystore when available (StrongBox on supported devices)
   - Keys are generated and stored in secure hardware (TEE/SE)
   - Protected against extraction

### Biometric Authentication

- Uses platform-native biometric APIs
- Biometric data never leaves the device
- Falls back to passcode if biometric fails
- Supports Face ID, Touch ID (iOS) and fingerprint/face unlock (Android)

### Backup Encryption

> **⚠️ WARNING: Backup encryption is NOT yet implemented!**
>
> The current implementation does **NOT** encrypt backups. Backups are only hex-encoded and are **NOT SECURE**. 
> Do **NOT** use this feature for production or store sensitive data until proper encryption is implemented.

**Intended design (not yet implemented):**
- Backups will be encrypted with AES-256-GCM
- Password-based key derivation (PBKDF2, 100,000 iterations)
- Random salt and nonce per backup
- Backup format versioning for future compatibility

### Best Practices

1. **Never log sensitive data**: Mnemonics, private keys, passwords
2. **Always zeroize**: Clear sensitive data from memory after use
3. **Validate inputs**: Check all user inputs before processing
4. **Use secure channels**: Only transmit over HTTPS
5. **Regular security audits**: Review and test security regularly

## Testing

Run tests:
```bash
cargo test -p bitcell-wallet-mobile
```

Run with output:
```bash
cargo test -p bitcell-wallet-mobile -- --nocapture
```

## Troubleshooting

### iOS Build Issues

**Issue**: "Library not found"
- Ensure `libbitcell_wallet_mobile.a` is added to "Link Binary With Libraries" in Build Phases
- Check library search paths in Build Settings

**Issue**: Biometric not working
- Verify `NSFaceIDUsageDescription` in Info.plist
- Check device has Face ID/Touch ID enrolled
- Test on real device (simulator has limited biometric support)

### Android Build Issues

**Issue**: "UnsatisfiedLinkError"
- Verify native libraries are in correct `jniLibs` directories
- Check ABI compatibility (arm64-v8a for most modern devices)
- Ensure `System.loadLibrary()` is called before SDK use

**Issue**: Biometric not available
- Check device has biometric hardware
- Verify `USE_BIOMETRIC` permission in manifest
- Test on real device (emulator biometric support limited)

## Examples

See the `examples/` directory for complete sample apps:
- `ios-example/`: SwiftUI wallet app
- `android-example/`: Jetpack Compose wallet app

## Contributing

Contributions welcome! Please:
1. Follow Rust code style guidelines
2. Add tests for new features
3. Update documentation
4. Test on both iOS and Android

## License

MIT OR Apache-2.0

## Support

- GitHub Issues: https://github.com/Steake/BitCell/issues
- Documentation: https://docs.bitcell.network
- Discord: https://discord.gg/bitcell
