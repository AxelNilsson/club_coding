use users::User;
use rocket::fairing::AdHoc;
use regex::Regex;

/// Context for rendering tera templates
/// for logged in endpoints.
#[derive(Serialize)]
pub struct LoggedInContext<'a> {
    /// Header used in tera templates.
    /// Mainly used for the title.
    pub header: &'a str,
    /// The user struct used by templates.
    /// For example the username for the toolbar.
    pub user: User,
}

/// Context for rendering tera templates
/// for not logged in endpoints.
#[derive(Serialize)]
pub struct Context<'a> {
    /// Header used in tera templates.
    /// Mainly used for the title.
    pub header: &'a str,
}

/// Struct for Stripe States.
/// Used in endpoints that requires the
/// stripe keys.
pub struct StripeToken {
    /// Publishable Key that you can
    /// show in public, for example in
    /// an HTML File for the JavaScript
    /// Stripe Library.
    pub publishable_key: String,
    /// Secret Key that you can not
    /// show in public. Only to be used
    /// in Rust code.
    pub secret_key: String,
}

/// Returns a AdHoc Fairing with two Stripe Tokens
/// Will panic if no Stripe Tokens are set in
/// Rocket.toml File
pub fn stripe_token_fairing() -> rocket::fairing::AdHoc {
    AdHoc::on_attach("Stripe", |rocket| {
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

/// Struct for Postmark Token.
/// Used in endpoints that requires the
/// postmark token. It is a Secret Key
/// that you can not show in public.
/// Only to be used in Rust code.
pub struct PostmarkToken(pub String);

/// Returns a AdHoc Fairing with the Postmark Token
/// Will panic if no Postmark Token is set in
/// Rocket.toml File
pub fn postmark_token_fairing() -> rocket::fairing::AdHoc {
    AdHoc::on_attach("Postmark", |rocket| {
        let config = rocket.config().clone();

        let postmark_token = config
            .get_str("postmark_token")
            .expect("postmark_token key not specified");

        Ok(rocket.manage(PostmarkToken(postmark_token.to_string())))
    })
}

/// Struct for E-Mail Regex.
/// By using an fairing we do not
/// have to make a new Regex each
/// time we are going to use it.
pub struct EmailRegex(pub regex::Regex);

/// Returns a AdHoc Fairing with the E-Mail Regex
/// Will panic if the Regex is not valid.
pub fn email_regex_fairing() -> rocket::fairing::AdHoc {
    AdHoc::on_attach("EmailRegex", |rocket| {
        let email_regex = Regex::new(r"(^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$)")
            .expect("email regex not valid");

        Ok(rocket.manage(EmailRegex(email_regex)))
    })
}
