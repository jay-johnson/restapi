use std::collections::HashMap;

use url::Url;

/// get_query_params_from_url
///
/// parse a url's query parameters and return as a
/// [`HashMap`](std::collections::HashMap)
/// where: key as a `String` => value as a `String`
///
/// please be warned that duplicate query parameter `keys` are
/// not supported correctly. this function will only work
/// with unique query_parameters at this time
///
/// credit source to:
///
/// <https://stackoverflow.com/questions/43272935/how-do-i-decode-a-url-and-get-the-query-string-as-a-hashmap>
///
/// # Arguments
///
/// * `tracking_label` - `&str` - logging label for caller
/// * `url` - `&str` - url to process
///
/// # Errors
///
/// `String` - error message
///
/// # Examples
///
/// ```
/// use restapi::utils::get_query_params_from_url::get_query_params_from_url;
/// let hash_map = tokio_test::block_on(
///     get_query_params_from_url(
///         "test-url",
///         "https://mydomain.io/user?key1=value1&key2=value2")
///     ).unwrap();
/// println!("{:?}", hash_map);
/// ```
pub async fn get_query_params_from_url(
    tracking_label: &str,
    url: &str)
-> Result<HashMap<String, String>, String>
{
    let url_len = url.len();
    if url_len == 0 {
        return Err(format!("get_query_params_from_url - no url set"));
    }
    else if url_len > 512 {
        return Err(format!("get_query_params_from_url - url is too long"));
    }
    let parsed_url = match Url::parse(url) {
        Ok(parsed_url) => parsed_url,
        Err(e) => {
            let err_msg = format!("{e}");
            error!("\
                {tracking_label} - \
                get_query_params_from_url failed to parse \
                {url} with err='{err_msg}'");
            return Err(err_msg);
        },
    };
    let query_params_hash_map: HashMap<String, String> =
        parsed_url.query_pairs().into_owned().collect();
    /*
    info!("\
        {tracking_label} - \
        found hash map - hash_map {url}");
    */
    return Ok(query_params_hash_map);
}
