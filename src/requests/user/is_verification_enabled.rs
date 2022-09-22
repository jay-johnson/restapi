//! Module for checking the environment variable:
//! ``USER_EMAIL_VERIFICATION_ENABLED`` to detect
//! if user email verification is enabled
//!

/// is_verification_enabled
///
/// Helper function to determine if
/// user email verification is enabled
///
/// ## Roadmap
///
/// This should move into the
/// [`CoreConfig`](crate::core::core_config::CoreConfig)
/// server statics.
///
/// # Returns
///
/// `bool` where `true` - email verfication is enabled,
/// `false - email verification is disabled
///
/// # Examples
///
/// ```bash
/// # default - email verification enabled
/// export USER_EMAIL_VERIFICATION_ENABLED=1
/// ```
///
/// ```rust
/// use restapi::requests::user::is_verification_enabled::is_verification_enabled;
/// return is_verification_enabled();
/// ```
///
pub fn is_verification_enabled() -> bool {
    std::env::var("USER_EMAIL_VERIFICATION_ENABLED")
        .unwrap_or_else(|_| "1".to_string())
        == *"1"
}
