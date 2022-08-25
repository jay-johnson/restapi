/// is_verification_required
///
/// Helper for determining if user
/// email verification is required for
/// login and access to resources
///
/// ## Roadmap
///
/// This should move into the
/// [`CoreConfig`](crate::core::core_config::CoreConfig)
/// server statics.
///
/// # Returns
///
/// `bool` where `true` - email verfication is required to login,
/// `false - email verification is required to login
///
/// # Examples
///
/// ```bash
/// # default is disabled - login without verifying email
/// export USER_EMAIL_VERIFICATION_REQUIRED=1
/// ```
///
/// ```rust
/// use restapi::requests::user::is_verification_required::is_verification_required;
/// return is_verification_required();
/// ```
///
pub fn is_verification_required() -> bool {
    std::env::var("USER_EMAIL_VERIFICATION_REQUIRED")
        .unwrap_or_else(|_| "0".to_string())
        == *"1"
}
