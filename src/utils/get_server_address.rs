/// get_server_address
///
/// wrapper for getting the server address from
/// the env var:
/// `API_ENDPOINT`
///
/// change the server address with: 
///
/// ```bash
/// export API_ENDPOINT="0.0.0.0:3000"
/// ```
///
/// # Examples
///
/// ```rust
/// use restapi::utils::get_server_address::get_server_address;
/// assert_eq!(
///     get_server_address("api"),
///     std::env::var("API_ENDPOINT")
///         .unwrap_or(String::from("0.0.0.0:3000")));
/// ```
///
pub fn get_server_address(
    server_name: &str)
-> String
{
    let endpont_name =
        std::env::var(
            format!("SERVER_NAME_{}", server_name).to_uppercase())
        .unwrap_or(String::from("api"));
    let api_address =
        std::env::var(
            format!("{endpont_name}_ENDPOINT").to_uppercase())
        .unwrap_or(String::from("0.0.0.0:3000"));
    return api_address; 
}
