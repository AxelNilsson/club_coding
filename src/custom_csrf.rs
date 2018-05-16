use rocket::http::Cookie;
use time::Duration;
use csrf::{AesGcmCsrfProtection, CsrfProtection};
use rocket::request::{self, FromRequest, Request};
use rocket::Outcome;
use data_encoding::BASE64;


pub struct CsrfToken(String);

impl CsrfToken {
    pub fn value(self) -> String {
        self.0
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for CsrfToken {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<CsrfToken, ()> {
        let protect = AesGcmCsrfProtection::from_key(*b"01234567012345670123456701234567");
        let (token, cookie) = protect
            .generate_token_pair(None, 300)
            .expect("couldn't generate token/cookie pair");

        let mut c = Cookie::new("csrf", cookie.b64_string());
        c.set_max_age(Duration::hours(24));
        request.cookies().add(c);
        return Outcome::Success(CsrfToken(token.b64_string()));
    }
}

pub struct CsrfCookie(String);

impl CsrfCookie {
    pub fn value(self) -> String {
        self.0
    }
}

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

pub fn csrf_matches(token: String, cookie: String) -> bool {
    let protect = AesGcmCsrfProtection::from_key(*b"01234567012345670123456701234567");
    let token_bytes = BASE64
        .decode(token.as_bytes())
        .expect("token not base64");
    let cookie_bytes = BASE64
        .decode(cookie.as_bytes())
        .expect("cookie not base64");

    let parsed_token = protect.parse_token(&token_bytes).expect("token not parsed");
    let parsed_cookie = protect
        .parse_cookie(&cookie_bytes)
        .expect("cookie not parsed");

    return protect.verify_token_pair(&parsed_token, &parsed_cookie);
}
