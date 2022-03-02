/// # JWT (json web tokens)
///
/// ## Configurable JWT Environment Variables
///
/// ### Header key for the token
///
/// ```bash
/// export TOKEN_HEADER="Bearer"
/// ```
///
/// ### Token Org (embedded in the jwt)
///
/// ```bash
/// export TOKEN_ORG="Org Name";
/// ```
///
/// ### Token Lifetime Duration
///
/// ```bash
/// # 30 days
/// export TOKEN_EXPIRATION_SECONDS_INTO_FUTURE=2592000;
/// # 7 days
/// export TOKEN_EXPIRATION_SECONDS_INTO_FUTURE=604800;
/// # 1 day
/// export TOKEN_EXPIRATION_SECONDS_INTO_FUTURE=86400;
/// ```
///
/// ### JWT Signing Keys
///
/// ```bash
/// export TOKEN_ALGO_KEY_DIR="./jwt"
/// export TOKEN_ALGO_PRIVATE_KEY_ORG="${TOKEN_ALGO_KEY_DIR}/private-key.pem"
/// export TOKEN_ALGO_PRIVATE_KEY="${TOKEN_ALGO_KEY_DIR}/private-key-pkcs8.pem"
/// export TOKEN_ALGO_PUBLIC_KEY="${TOKEN_ALGO_KEY_DIR}/public-key.pem"
/// ```
///
/// generate your own jwt keys with these commands (bash)
///
/// ```bash
/// openssl ecparam -name prime256v1 -genkey -out "${TOKEN_ALGO_PRIVATE_KEY_ORG}"
/// openssl pkcs8 -topk8 -nocrypt -in private-key.pem -out "${TOKEN_ALGO_PRIVATE_KEY}"
/// openssl ec -in "${TOKEN_ALGO_PRIVATE_KEY_ORG}" -pubout -out "${TOKEN_ALGO_PUBLIC_KEY}"
/// ```

use serde::Deserialize;
use serde::Serialize;

use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::decode;
use jsonwebtoken::encode;
use jsonwebtoken::Algorithm;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::EncodingKey;
use jsonwebtoken::Header;
use jsonwebtoken::TokenData;
use jsonwebtoken::Validation;

/// TokenClaim
///
/// custom claim contained in the signed jwt
///
/// example:
/// <https://github.com/Keats/jsonwebtoken/blob/master/examples/validation.rs#L6-L11>
///
/// # Arguments
///
/// * `sub` - String - custom, unique identifier
/// * `org` - String - custom, unique org identifier
/// * `exp` - usize - epoch time when the token expires
///
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TokenClaim {
    pub sub: String,
    pub org: String,
    pub exp: usize,
}

/// validate_token
///
/// validate a decoded jwt token
///
/// 1. create a token validator object
/// 2. decode the token by decrypting it using the
///    `decoding_key_bytes` and validate the contents
///
/// # Returns
///
/// ## validate_token on Success Returns
///
/// A valid user token will return:
///
/// Ok([`TokenData`](jsonwebtoken::TokenData))
///
/// # Arguments
///
/// * `tracking_label` - `&str` - custom, unique identifier
/// * `token` - `&str` - custom, unique org identifier
/// * `uid` - `&str` - epoch time when the token expires
/// * `decoding_key_bytes` - `&[u8]` - jwt key
///   contents in bytes
///
/// # Errors
///
/// ## validate_token on Failure Returns
///
/// `String` error messages can be returned for many reasons
/// (connectivity, aws credentials, mfa timeouts, etc.)
///
/// Err(err_msg: `String`)
///
/// If it is not a valid user token it will return:
///
/// Err(err_msg: `String`)
///
pub async fn validate_token(
    tracking_label: &str,
    token: &str,
    uid: &str,
    decoding_key_bytes: &[u8])
-> Result<TokenData::<TokenClaim>, String>
{
    let verbose = false;
    let label = format!("{tracking_label}");

    // 1. prep to validate the token
    let token_to_validate = Validation {
        sub: Some(uid.to_string()),
        ..Validation::new(Algorithm::ES256)
    };

    if verbose {
        trace!("\
            {label} - \
            token={token}");
    }
    let token_data = match decode::<TokenClaim>(
        &token,
        &DecodingKey::from_ec_pem(&decoding_key_bytes).unwrap(),
        &token_to_validate) {
            Ok(c) => {
                c
            },
            Err(err) => match *err.kind() {
                ErrorKind::InvalidToken => {
                    return Err(format!("\
                        {label} - token was invalid"));
                },
                ErrorKind::InvalidAlgorithm => {
                    return Err(format!("\
                        {label} - token algorithm is invalid"));
                },
                ErrorKind::InvalidIssuer => {
                    return Err(format!("\
                        {label} - token issuer is invalid"));
                },
                ErrorKind::ExpiredSignature => {
                    return Err(format!("\
                        {label} - token expired - need to refresh"));
                },
                _ => {
                    return Err(format!("\
                        {label} - hit an unexpected err='{:?}'",
                        err));
                },
            },
    };
    return Ok(token_data);
}

/// get_current_timestamp
///
/// get the current unix epoch time as a `usize`
///
pub fn get_current_timestamp()
-> usize
{
    let start = SystemTime::now();
    return start.duration_since(UNIX_EPOCH)
        .expect("Time went backwards").as_secs() as usize;
}

/// get_expiration_epoch_time
///
/// determine when the jwt should expire in the future.
/// and return it as a `usize`
///
pub fn get_expiration_epoch_time(
    seconds_in_future: usize)
-> usize
{
    let token_expiration: usize = get_current_timestamp() + seconds_in_future;
    return token_expiration;
}

/// get_token_org
///
/// wrapper for returning an env var `TOKEN_ORG`
/// that can change the signed jwt contents for a
/// custom organization name
///
/// v2 this should move into the server statics:
/// [`CoreConfig`](crate::core::core_config::CoreConfig)
///
pub fn get_token_org()
-> String
{
    let token_org = std::env::var("TOKEN_ORG")
        .unwrap_or(String::from("Org Name"));
    return token_org;
}

/// get_token_expiration_in_seconds
///
/// wrapper for returning an env var
/// `TOKEN_EXPIRATION_SECONDS_INTO_FUTURE`
/// that can change the future expiration epoch time
/// for a new jwt
///
/// v2 this should move into the server statics:
/// [`CoreConfig`](crate::core::core_config::CoreConfig)
///
pub fn get_token_expiration_in_seconds()
-> usize
{
    let token_expiration_str = std::env::var("TOKEN_EXPIRATION_SECONDS_INTO_FUTURE")
        .unwrap_or(String::from("2592000"));
    let token_expiration_usize = token_expiration_str.parse::<usize>().unwrap();
    return token_expiration_usize;
}

/// create_token
///
/// create a [`TokenClaim`](crate::jwt::api::TokenClaim), and
/// sign the it using the algorithm:
/// [`ES256`](jsonwebtoken::Algorithm)
///
/// # Arguments
///
/// * `tracking_label` - `&str` - logging label for the caller
/// * `uid` - `&str` - unique identifier for this application
/// * `encoding_key_bytes` - `&[u8]` - jwt key
///   contents in bytes
///
/// # Returns
///
/// Ok(token: `String`)
///
/// # Errors
///
/// ## create_token on Failure Returns
///
/// Err(err_msg: `String`)
///
pub async fn create_token(
    tracking_label: &str,
    uid: &str,
    encoding_key_bytes: &[u8])
-> Result<String, String>
{

    // env vars for these
    let token_org = get_token_org();
    let token_expiration = get_expiration_epoch_time(
        get_token_expiration_in_seconds());

    let access_claim = TokenClaim {
        sub: uid.to_string(),
        org: token_org.clone(),
        exp: token_expiration
    };

    let token = match encode(
            &Header::new(Algorithm::ES256),
            &access_claim,
            &EncodingKey::from_ec_pem(&encoding_key_bytes).unwrap()
        ) {
            Ok(t) => t,
            Err(e) => {
                let err_msg = format!("\
                    {tracking_label} - \
                    failed to encode token for uid={uid} with err='{}'",
                    e);
                error!("{err_msg}");
                return Err(err_msg);
            },
    };
    /*
    if verbose {
        info!("\
            {tracking_label} - \
            token that is stored in a db: {:?} - sleeping",
            token);
    }
    */

    return Ok(token);
}
