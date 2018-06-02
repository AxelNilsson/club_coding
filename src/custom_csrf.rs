use rocket::http::Cookie;
use time::Duration;
use csrf::{AesGcmCsrfProtection, CsrfProtection};
use rocket::request::{self, FromRequest, Request};
use rocket::Outcome;
use data_encoding::BASE64;

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
        let protect = AesGcmCsrfProtection::from_key(*b"01234567012345670123456701234567");
        match protect.generate_token_pair(None, 300) {
            Ok((token, cookie)) => {
                let mut c = Cookie::new("csrf", cookie.b64_string());
                c.set_max_age(Duration::hours(24));
                request.cookies().add(c);
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
pub fn csrf_matches(token: String, cookie: String) -> bool {
    let protect = AesGcmCsrfProtection::from_key(*b"01234567012345670123456701234567");
    match BASE64.decode(token.as_bytes()) {
        Ok(token_bytes) => match BASE64.decode(cookie.as_bytes()) {
            Ok(cookie_bytes) => match protect.parse_token(&token_bytes) {
                Ok(parsed_token) => match protect.parse_cookie(&cookie_bytes) {
                    Ok(parsed_cookie) => protect.verify_token_pair(&parsed_token, &parsed_cookie),
                    Err(_) => false,
                },
                Err(_) => false,
            },
            Err(_) => false,
        },
        Err(_) => false,
    }
}
