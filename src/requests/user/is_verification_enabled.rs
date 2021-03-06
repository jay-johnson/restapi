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
pub fn is_verification_enabled()
-> bool
{
    return std::env::var("USER_EMAIL_VERIFICATION_ENABLED")
        .unwrap_or(String::from("1")) == String::from("1");
}
