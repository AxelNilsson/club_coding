pub mod customer;
use rocket::Route;
use rocket::request::Form;
use rocket_contrib::Template;
use rocket::response::{Flash, Redirect};
use users::User;
use rocket::request::FlashMessage;
use database::DbConn;
use structs::{PostmarkToken, StripeToken};
use rocket::State;
use charge::customer::charge;
use custom_csrf::{csrf_matches, CsrfCookie, CsrfToken};

#[cfg(test)]
mod tests;

#[derive(Serialize)]
pub struct ChargeContext<'a> {
    /// Header used in tera templates.
    /// Mainly used for the title.
    pub header: &'a str,
    /// CSRF Token. Used as a hidden
    /// input in the form.
    pub csrf: String,
    /// The user struct used by templates.
    /// For example the username for the toolbar.
    pub user: User,
    /// Stripe Publishable Key for use with the
    /// Stripe JS Library.
    pub publishable_key: &'a str,
    /// Flash name if the request is redirected
    /// with one.
    pub flash_name: String,
    /// Flash message if the request is redirected
    /// with one.
    pub flash_msg: String,
}

/// GET Endpoint for the page of
/// adding a card. Endpoints checks if the
/// user is logged in by using the
/// user request guard. If the user
/// is not logged in it forwards
/// the request.
/// Responds with the Add Card Template
/// in the series folder.
#[get("/card/add")]
fn add_card_page(
    user: User,
    token: CsrfToken,
    stripe_token: State<StripeToken>,
    flash: Option<FlashMessage>,
) -> Template {
    let (name, msg) = match flash {
        Some(flash) => (flash.name().to_string(), flash.msg().to_string()),
        None => ("".to_string(), "".to_string()),
    };
    let context = ChargeContext {
        header: "Add card",
        csrf: token.value(),
        user: user,
        publishable_key: &stripe_token.publishable_key,
        flash_name: name,
        flash_msg: msg,
    };
    Template::render("payment/add_card", &context)
}

/// GET Endpoint for the page of
/// adding a card. Endpoints checks if the
/// user is logged in by using the
/// user request guard. If the user
/// is not logged in it forwards
/// the request. The UUID is used to
/// send the user back to the video
/// after payment has been made.
/// Responds with the Add Card Template
/// in the series folder.
#[get("/card/add/<_uuid>")]
fn add_card_uuid_page(
    user: User,
    token: CsrfToken,
    stripe_token: State<StripeToken>,
    flash: Option<FlashMessage>,
    _uuid: String,
) -> Template {
    let (name, msg) = match flash {
        Some(flash) => (flash.name().to_string(), flash.msg().to_string()),
        None => ("".to_string(), "".to_string()),
    };
    let context = ChargeContext {
        header: "Add card",
        csrf: token.value(),
        user: user,
        publishable_key: &stripe_token.publishable_key,
        flash_name: name,
        flash_msg: msg,
    };
    Template::render("payment/add_card", &context)
}

/// Struct for all of the data that we
/// recieve from Stripe when using the
/// Stripe JS Library.
#[derive(FromForm)]
pub struct Stripe {
    /// CSRF Token from the form
    pub csrf: String,
    // Card Address City
    pub card_address_city: Option<String>,
    /// Card Address Country
    pub card_address_country: Option<String>,
    /// Card Address Line 1
    pub card_address_line1: Option<String>,
    /// Card Address Line 1 check
    pub card_address_line1_check: Option<String>,
    /// Card Address Line 2
    pub card_address_line2: Option<String>,
    /// Card Address state.
    pub card_address_state: Option<String>,
    /// Card Address ZIP.
    pub card_address_zip: Option<String>,
    /// Address ZIP check of the card
    pub card_address_zip_check: Option<String>,
    /// Brand of the card
    pub card_brand: String,
    /// Country of the card
    pub card_country: String,
    /// CVC check of the card
    pub card_cvc_check: Option<String>,
    /// Last four numbers of the card.
    pub card_dynamic_last4: Option<String>,
    /// Month the card expires.
    pub card_exp_month: i32,
    /// Year the card expires.
    pub card_exp_year: i32,
    /// Card Funding
    pub card_funding: Option<String>,
    /// String ID of the card
    pub card_id: Option<String>,
    /// Last four numbers of the card.
    pub card_last4: String,
    /// Meta data of the Card.
    pub card_metadata: Option<String>,
    /// Name of the Card.
    pub card_name: Option<String>,
    /// Card Object.
    pub card_object: Option<String>,
    /// Card Tokenization Method
    pub card_tokenization_method: Option<String>,
    /// IP of the user sending the
    /// request.
    pub client_ip: String,
    /// Unix timestamp of when the
    /// card was created.
    pub created: i64,
    /// String ID of the Token.
    pub id: String,
    /// Boolean showing if we are
    /// using livemode or testmode.
    pub livemode: bool,
    /// The object
    pub object: Option<String>,
    /// Type of Payment
    #[form(field = "type")]
    pub type_of_payment: Option<String>,
    /// Boolean indicating if the
    /// card has been used or not.
    pub used: bool,
}

/// POST Endpoint for the page to add a
/// card to your account. Endpoints checks
/// if the  user is logged in by using the
/// user request guard. If the user
/// is not logged in it forwards
/// the request.
/// It requires some of the parameters in the
/// Stripe struct submitted as a form.
/// If everything is successful, it will add
/// a card to the user and redirect the user
/// to the index. Otherwise it will redirect
/// the user back to the card add page.
#[post("/card/add", data = "<form_data>")]
fn add_card(
    conn: DbConn,
    user: User,
    csrf_cookie: CsrfCookie,
    stripe_token: State<StripeToken>,
    postmark: State<PostmarkToken>,
    form_data: Form<Stripe>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let data = form_data.into_inner();
    if !csrf_matches(&data.csrf, &csrf_cookie.value()) {
        return Err(Flash::error(Redirect::to("/card/add"), "CSRF Failed."));
    }
    match charge(
        &conn,
        &stripe_token.secret_key,
        &postmark.0,
        &data,
        &user.email,
        user.id,
    ) {
        Ok(()) => Ok(Flash::success(
            Redirect::to("/"),
            "Card added. Welcome to the club!",
        )),
        _ => Err(Flash::error(
            Redirect::to("/card/add"),
            "An error occured, please try again later.",
        )),
    }
}

/// POST Endpoint for the page to add a
/// card to your account. Endpoints checks
/// if the  user is logged in by using the
/// user request guard. If the user
/// is not logged in it forwards
/// the request.
/// It requires some of the parameters in the
/// Stripe struct submitted as a form.
/// If everything is successful, it will add
/// a card to the user and redirect the user
/// to the watch video page for the UUID.
/// Otherwise it will redirect the user back
/// to the card add page.
#[post("/card/add/<uuid>", data = "<form_data>")]
fn add_card_uuid(
    conn: DbConn,
    user: User,
    csrf_cookie: CsrfCookie,
    stripe_token: State<StripeToken>,
    postmark: State<PostmarkToken>,
    form_data: Form<Stripe>,
    uuid: String,
) -> Result<Redirect, Flash<Redirect>> {
    let data = form_data.into_inner();
    if !csrf_matches(&data.csrf, &csrf_cookie.value()) {
        return Err(Flash::error(
            Redirect::to(&format!("/card/add/{}", uuid)),
            "CSRF Failed.",
        ));
    }
    match charge(
        &conn,
        &stripe_token.secret_key,
        &postmark.0,
        &data,
        &user.email,
        user.id,
    ) {
        Ok(()) => Ok(Redirect::to(&format!("/watch/{}/buy/fiat", uuid))),
        _ => Err(Flash::error(
            Redirect::to(&format!("/card/add/{}", uuid)),
            "An error occured, please try again later.",
        )),
    }
}

/// Assembles all of the endpoints.
/// The upside of assembling all of the endpoints here
/// is that we don't have to update the main function but
/// instead we can keep all of the changes in here.
pub fn endpoints() -> Vec<Route> {
    routes![add_card_page, add_card, add_card_uuid_page, add_card_uuid]
}
