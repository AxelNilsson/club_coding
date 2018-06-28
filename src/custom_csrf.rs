use csrf::{AesGcmCsrfProtection, CsrfProtection};
use data_encoding::BASE64;
use rocket::fairing::AdHoc;
use rocket::http::Cookie;
use rocket::request::{self, FromRequest, Request};
use rocket::{Outcome, State};
use time::Duration;

/// Struct for CSRF secret key.
/// Used in endpoints that requires the
/// CSRF secret key. It is a Secret Key
/// that you can not show in public.
/// Only to be used in Rust code.
pub struct CSRFSecretToken(pub [u8; 32]);

/// Returns a AdHoc Fairing with the CSRF secret key.
/// Will panic if no CSRF Key is set in
/// Rocket.toml File
pub fn csrf_secret_key_fairing() -> rocket::fairing::AdHoc {
    AdHoc::on_attach(|rocket| {
        let config = rocket.config().clone();

        let csrf_secret_key = config
            .get_str("csrf_secret_key")
            .expect("csrf_secret_key key not specified");

        let csrf_secret_key = csrf_secret_key.as_bytes();
        assert_eq!(csrf_secret_key.len(), 32);

        let mut csrf_key = [0u8; 32];
        csrf_key.copy_from_slice(csrf_secret_key);

        Ok(rocket.manage(CSRFSecretToken(csrf_key)))
    })
}

/// Struct for CSRF Token.
/// Used in endpoints that requires
/// CSRF Protection i.e. GETs.
/// Will be used as a hidden input.
pub struct CsrfToken(String);

/// Implementations for the CSRFToken
/// struct. Mainly the .value() function.
impl CsrfToken {
    /// The .value() function returns
    /// the first and only value in the
    /// tuple, a string.
    pub fn value(self) -> String {
        self.0
    }
}

/// Request guard that returns the
/// CSRF Token and sets the CSRF Cookie.
/// If this is successful it returns the CSRF
/// Token and if it fails due to the fact
/// that it can not generate a pair, it
/// will forward the connection.
impl<'a, 'r> FromRequest<'a, 'r> for CsrfToken {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<CsrfToken, ()> {
        let csrf_secret_key = request.guard::<State<CSRFSecretToken>>()?;
        let protect = AesGcmCsrfProtection::from_key(csrf_secret_key.0);
        match protect.generate_token_pair(None, 300) {
            Ok((token, cookie)) => {
                let mut csrf_cookie = Cookie::new("csrf", cookie.b64_string());
                csrf_cookie.set_max_age(Duration::hours(24));
                csrf_cookie.set_path("/");
                request.cookies().add(csrf_cookie);
                Outcome::Success(CsrfToken(token.b64_string()))
            }
            Err(_) => Outcome::Forward(()),
        }
    }
}

/// Struct for CSRF Token.
/// Used in endpoints that requires
/// CSRF Verification i.e. POSTs.
/// Will only be used in Rust Code.
pub struct CsrfCookie(String);

/// Implementations for the CSRF Cookie
/// struct. Mainly the .value() function.
impl CsrfCookie {
    /// The .value() function returns
    /// the first and only value in the
    /// tuple, a string.
    pub fn value(self) -> String {
        self.0
    }
}

/// Request guard that returns the
/// CSRF Cookie if it is set.
/// If it's not set, it forwards the
/// connection.
impl<'a, 'r> FromRequest<'a, 'r> for CsrfCookie {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<CsrfCookie, ()> {
        let cookie = request
            .cookies()
            .get("csrf")
            .map(|cookie| cookie.value().to_string());

        match cookie {
            Some(cookie) => Outcome::Success(CsrfCookie(cookie)),
            None => Outcome::Forward(()),
        }
    }
}

/// Function takes in the CSRF Token
/// and the CSRF Cookie, both as strings
/// and tries to verify the pair.
/// Returns a boolean of whether the
/// pairs match or not.
pub fn csrf_matches(csrf_secret_key: [u8; 32], token: &str, cookie: &str) -> bool {
    let protect = AesGcmCsrfProtection::from_key(csrf_secret_key);
    let token_bytes = match BASE64.decode(token.as_bytes()) {
        Ok(token_bytes) => token_bytes,
        Err(_) => return false,
    };

    let cookie_bytes = match BASE64.decode(cookie.as_bytes()) {
        Ok(cookie_bytes) => cookie_bytes,
        Err(_) => return false,
    };

    let parsed_token = match protect.parse_token(&token_bytes) {
        Ok(parsed_token) => parsed_token,
        Err(_) => return false,
    };

    let parsed_cookie = match protect.parse_cookie(&cookie_bytes) {
        Ok(parsed_cookie) => parsed_cookie,
        Err(_) => return false,
    };

    return protect.verify_token_pair(&parsed_token, &parsed_cookie);
}
