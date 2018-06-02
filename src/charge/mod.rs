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

#[derive(Serialize)]
pub struct ChargeContext<'a> {
    pub header: &'a str,
    pub user: User,
    pub publishable_key: &'a str,
    pub flash_name: String,
    pub flash_msg: String,
}

#[get("/card/add")]
fn add_card_page(
    user: User,
    stripe_token: State<StripeToken>,
    flash: Option<FlashMessage>,
) -> Template {
    let (name, msg) = match flash {
        Some(flash) => (flash.name().to_string(), flash.msg().to_string()),
        None => ("".to_string(), "".to_string()),
    };
    let context = ChargeContext {
        header: "Club Coding",
        user: user,
        publishable_key: &stripe_token.publishable_key,
        flash_name: name,
        flash_msg: msg,
    };
    Template::render("payment/add_card", &context)
}

#[get("/card/add/<_uuid>")]
fn add_card_uuid_page(
    user: User,
    stripe_token: State<StripeToken>,
    flash: Option<FlashMessage>,
    _uuid: String,
) -> Template {
    let (name, msg) = match flash {
        Some(flash) => (flash.name().to_string(), flash.msg().to_string()),
        None => ("".to_string(), "".to_string()),
    };
    let context = ChargeContext {
        header: "Club Coding",
        user: user,
        publishable_key: &stripe_token.publishable_key,
        flash_name: name,
        flash_msg: msg,
    };
    Template::render("payment/add_card", &context)
}

#[derive(Debug, FromForm)]
pub struct Stripe {
    pub card_address_city: Option<String>,
    pub card_address_country: Option<String>,
    pub card_address_line1: Option<String>,
    pub card_address_line1_check: Option<String>,
    pub card_address_line2: Option<String>,
    pub card_address_state: Option<String>,
    pub card_address_zip: Option<String>,
    pub card_address_zip_check: Option<String>,
    pub card_brand: String,
    pub card_country: String,
    pub card_cvc_check: Option<String>,
    pub card_dynamic_last4: Option<String>,
    pub card_exp_month: i32,
    pub card_exp_year: i32,
    pub card_funding: Option<String>,
    pub card_id: Option<String>,
    pub card_last4: String,
    pub card_metadata: Option<String>,
    pub card_name: Option<String>,
    pub card_object: Option<String>,
    pub card_tokenization_method: Option<String>,
    pub client_ip: String,
    pub created: i64,
    pub id: String,
    pub livemode: bool,
    pub object: Option<String>,
    #[form(field = "type")]
    pub type_of_payment: Option<String>,
    pub used: bool,
}

#[post("/card/add", data = "<form_data>")]
fn add_card(
    conn: DbConn,
    stripe_token: State<StripeToken>,
    postmark: State<PostmarkToken>,
    user: User,
    form_data: Form<Stripe>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let data = form_data.into_inner();
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

#[post("/card/add/<uuid>", data = "<form_data>")]
fn add_card_uuid(
    conn: DbConn,
    stripe_token: State<StripeToken>,
    postmark: State<PostmarkToken>,
    user: User,
    form_data: Form<Stripe>,
    uuid: String,
) -> Result<Redirect, Flash<Redirect>> {
    let data = form_data.into_inner();
    match charge(
        &conn,
        &stripe_token.secret_key,
        &postmark.0,
        &data,
        &user.email,
        user.id,
    ) {
        Ok(()) => Ok(Redirect::to(&format!("/watch/{}/buy", uuid))),
        _ => Err(Flash::error(
            Redirect::to("/card/add"),
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
