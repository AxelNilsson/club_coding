use users::User;
use rocket::fairing::AdHoc;

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
