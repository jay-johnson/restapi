/// get_uuid
///
/// wrapper for building a unique id
///
/// # Returns
///
/// `String` with a reasonaly-unique identifier
///
/// # Examples
///
/// ```rust
/// use restapi::utils::get_uuid::get_uuid;
/// assert_ne!(get_uuid(), format!("not-a-uuid"));
/// ```
///
pub fn get_uuid()
-> String
{
    return String::from(uuid::Uuid::new_v4().to_string().replace("-", ""));
}
