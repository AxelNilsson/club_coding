use users::User;
use rocket::fairing::AdHoc;
use regex::Regex;

#[derive(Serialize)]
pub struct LoggedInContext<'a> {
    pub header: &'a str,
    pub user: User,
}

#[derive(Serialize)]
pub struct Context<'a> {
    pub header: &'a str,
}

pub struct StripeToken {
    pub publishable_key: String,
    pub secret_key: String,
}

/// Returns a AdHoc Fairing with two Stripe Tokens
/// Will panic if no Stripe Tokens are set in
/// Rocket.toml File
pub fn stripe_token_fairing() -> rocket::fairing::AdHoc {
    AdHoc::on_attach(|rocket| {
        let config = rocket.config().clone();

        let publishable = config
            .get_str("stripe_publishable")
            .expect("stripe_publishable key not specified");

        let secret = config
            .get_str("stripe_secret")
            .expect("stripe_secret key not specified");

        Ok(rocket.manage(StripeToken {
            publishable_key: publishable.to_string(),
            secret_key: secret.to_string(),
        }))
    })
}

pub struct PostmarkToken(pub String);

/// Returns a AdHoc Fairing with the Postmark Token
/// Will panic if no Postmark Token is set in
/// Rocket.toml File
pub fn postmark_token_fairing() -> rocket::fairing::AdHoc {
    AdHoc::on_attach(|rocket| {
        let config = rocket.config().clone();

        let postmark_token = config
            .get_str("postmark_token")
            .expect("postmark_token key not specified");

        Ok(rocket.manage(PostmarkToken(postmark_token.to_string())))
    })
}

pub struct EmailRegex {
    pub regex: regex::Regex,
}

/// Returns a AdHoc Fairing with the Postmark Token
/// Will panic if no Postmark Token is set in
/// Rocket.toml File
pub fn email_regex_fairing() -> rocket::fairing::AdHoc {
    AdHoc::on_attach(|rocket| {
        let email_regex = Regex::new(r"(^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$)")
            .expect("email regex not valid");

        Ok(rocket.manage(EmailRegex { regex: email_regex }))
    })
}

pub struct PasswordRegex {
    pub regex: regex::Regex,
}

/// Returns a AdHoc Fairing with the Postmark Token
/// Will panic if no Postmark Token is set in
/// Rocket.toml File
pub fn password_regex_fairing() -> rocket::fairing::AdHoc {
    AdHoc::on_attach(|rocket| {
        let password_regex = Regex::new(r"(^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$)")
            .expect("password regex not valid");

        Ok(rocket.manage(PasswordRegex {
            regex: password_regex,
        }))
    })
}
