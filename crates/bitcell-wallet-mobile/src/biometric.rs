//! Biometric authentication support
//!
//! This module provides a platform-agnostic interface for biometric authentication.
//! On iOS, this should be implemented using LocalAuthentication framework (Face ID/Touch ID).
//! On Android, this should be implemented using BiometricPrompt API.

use crate::error::{MobileWalletError, Result};

/// Biometric authentication result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BiometricResult {
    /// Authentication succeeded
    Success,
    /// Authentication failed (wrong biometric)
    Failed,
    /// User cancelled the authentication
    Cancelled,
    /// Biometric hardware not available
    NotAvailable,
    /// User has not enrolled biometrics
    NotEnrolled,
}

/// Biometric authentication provider trait
///
/// Platform-specific implementations should:
/// - iOS: Use LAContext from LocalAuthentication framework
/// - Android: Use BiometricPrompt from androidx.biometric
///
/// # Platform Implementation Guide
///
/// ## iOS (Swift)
/// ```swift
/// import LocalAuthentication
///
/// class IosBiometricProvider: BiometricAuthProvider {
///     func authenticate(promptMessage: String) -> BiometricResult {
///         let context = LAContext()
///         var error: NSError?
///         
///         guard context.canEvaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, error: &error) else {
///             return error?.code == LAError.biometryNotEnrolled.rawValue ? .notEnrolled : .notAvailable
///         }
///         
///         // Synchronous for simplicity - should be async in production
///         var result: BiometricResult = .failed
///         context.evaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, 
///                               localizedReason: promptMessage) { success, error in
///             if success {
///                 result = .success
///             } else if let error = error as? LAError {
///                 result = error.code == .userCancel ? .cancelled : .failed
///             }
///         }
///         return result
///     }
///     
///     func isAvailable() -> Bool {
///         LAContext().canEvaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, error: nil)
///     }
///     
///     func isEnrolled() -> Bool {
///         let context = LAContext()
///         return context.canEvaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, error: nil)
///     }
/// }
/// ```
///
/// ## Android (Kotlin)
/// ```kotlin
/// import androidx.biometric.BiometricPrompt
/// import androidx.biometric.BiometricManager
///
/// class AndroidBiometricProvider(private val activity: FragmentActivity) : BiometricAuthProvider {
///     override fun authenticate(promptMessage: String): BiometricResult {
///         val biometricManager = BiometricManager.from(activity)
///         
///         return when (biometricManager.canAuthenticate(BiometricManager.Authenticators.BIOMETRIC_STRONG)) {
///             BiometricManager.BIOMETRIC_SUCCESS -> {
///                 // Show biometric prompt
///                 val promptInfo = BiometricPrompt.PromptInfo.Builder()
///                     .setTitle("BitCell Wallet")
///                     .setSubtitle(promptMessage)
///                     .setNegativeButtonText("Cancel")
///                     .build()
///                     
///                 // Handle authentication callback
///                 BiometricResult.SUCCESS // Simplified - should handle async callback
///             }
///             BiometricManager.BIOMETRIC_ERROR_NONE_ENROLLED -> BiometricResult.NOT_ENROLLED
///             else -> BiometricResult.NOT_AVAILABLE
///         }
///     }
///     
///     override fun isAvailable(): Boolean {
///         val manager = BiometricManager.from(activity)
///         return manager.canAuthenticate(BiometricManager.Authenticators.BIOMETRIC_STRONG) == 
///                BiometricManager.BIOMETRIC_SUCCESS
///     }
///     
///     override fun isEnrolled(): Boolean = isAvailable()
/// }
/// ```
pub trait BiometricAuthProvider: Send + Sync {
    /// Authenticate using biometric
    ///
    /// Shows platform-specific biometric prompt and returns result
    fn authenticate(&self, prompt_message: String) -> BiometricResult;
    
    /// Check if biometric authentication is available on device
    fn is_available(&self) -> bool;
    
    /// Check if user has enrolled biometrics
    fn is_enrolled(&self) -> bool;
}

/// Mock biometric provider for testing
pub struct MockBiometricProvider {
    available: bool,
    enrolled: bool,
    should_succeed: bool,
}

impl MockBiometricProvider {
    pub fn new() -> Self {
        Self {
            available: true,
            enrolled: true,
            should_succeed: true,
        }
    }
    
    pub fn with_availability(mut self, available: bool) -> Self {
        self.available = available;
        self
    }
    
    pub fn with_enrollment(mut self, enrolled: bool) -> Self {
        self.enrolled = enrolled;
        self
    }
    
    pub fn with_success(mut self, should_succeed: bool) -> Self {
        self.should_succeed = should_succeed;
        self
    }
}

impl Default for MockBiometricProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl BiometricAuthProvider for MockBiometricProvider {
    fn authenticate(&self, _prompt_message: String) -> BiometricResult {
        if !self.available {
            return BiometricResult::NotAvailable;
        }
        if !self.enrolled {
            return BiometricResult::NotEnrolled;
        }
        if self.should_succeed {
            BiometricResult::Success
        } else {
            BiometricResult::Failed
        }
    }
    
    fn is_available(&self) -> bool {
        self.available
    }
    
    fn is_enrolled(&self) -> bool {
        self.enrolled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_biometric_success() {
        let provider = MockBiometricProvider::new();
        let result = provider.authenticate("Test authentication".to_string());
        assert_eq!(result, BiometricResult::Success);
    }

    #[test]
    fn test_mock_biometric_not_available() {
        let provider = MockBiometricProvider::new().with_availability(false);
        let result = provider.authenticate("Test".to_string());
        assert_eq!(result, BiometricResult::NotAvailable);
        assert!(!provider.is_available());
    }

    #[test]
    fn test_mock_biometric_not_enrolled() {
        let provider = MockBiometricProvider::new().with_enrollment(false);
        let result = provider.authenticate("Test".to_string());
        assert_eq!(result, BiometricResult::NotEnrolled);
        assert!(!provider.is_enrolled());
    }

    #[test]
    fn test_mock_biometric_failed() {
        let provider = MockBiometricProvider::new().with_success(false);
        let result = provider.authenticate("Test".to_string());
        assert_eq!(result, BiometricResult::Failed);
    }
}
