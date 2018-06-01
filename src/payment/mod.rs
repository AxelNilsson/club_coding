pub mod customer;
pub mod database;

use rocket::Route;
use rocket::request::Form;
use rocket_contrib::Template;
use rocket::response::{Flash, Redirect};
use database::DbConn;
use users::User;
use rocket::request::FlashMessage;
use structs::{PostmarkToken, StripeToken};
use rocket::State;
use charge::ChargeContext;
use charge::Stripe;
use payment::customer::{charge, delete};
use payment::database::{customer_exists, get_charges};

#[derive(Serialize)]
pub struct Charge {
    amount: i32,
    date: String,
    series: String,
}

#[derive(Serialize)]
struct PaymentsContext {
    header: String,
    user: User,
    flash_name: String,
    flash_msg: String,
    charges: Vec<Charge>,
}

#[get("/")]
fn payments_page(
    conn: DbConn,
    user: User,
    flash: Option<FlashMessage>,
) -> Result<Template, Redirect> {
    match customer_exists(&conn, user.id) {
        Some(_) => {
            let charges = get_charges(&conn, user.id);
            let (name, msg) = match flash {
                Some(flash) => (flash.name().to_string(), flash.msg().to_string()),
                None => ("".to_string(), "".to_string()),
            };
            let context = PaymentsContext {
                header: "Club Coding".to_string(),
                user: user,
                flash_name: name,
                flash_msg: msg,
                charges: charges,
            };
            Ok(Template::render("payment/payment", &context))
        }
        None => Err(Redirect::to("/card/add")),
    }
}

#[get("/card/update")]
fn update_card_page(
    user: User,
    stripe_token: State<StripeToken>,
    flash: Option<FlashMessage>,
) -> Template {
    let (name, msg) = match flash {
        Some(flash) => (flash.name().to_string(), flash.msg().to_string()),
        None => ("".to_string(), "".to_string()),
    };
    let context = ChargeContext {
        header: "Club Coding".to_string(),
        user: user,
        publishable_key: &stripe_token.publishable_key,
        flash_name: name,
        flash_msg: msg,
    };
    Template::render("payment/update_card", &context)
}

#[post("/card/update", data = "<form_data>")]
fn update_card(
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
        user.id,
        user.email,
    ) {
        Ok(()) => Ok(Flash::success(
            Redirect::to("/"),
            "Card updated. Great choice!",
        )),
        _ => Err(Flash::error(
            Redirect::to("/settings/payment/card/update"),
            "An error occured, please try again later.",
        )),
    }
}

#[post("/card/delete")]
fn delete_card(
    conn: DbConn,
    postmark: State<PostmarkToken>,
    user: User,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match delete(&conn, &postmark.0, user.id, user.email) {
        Ok(()) => Ok(Flash::success(Redirect::to("/"), "Oh no! Card deleted.")),
        _ => Err(Flash::error(
            Redirect::to("/settings/payment"),
            "An error occured, please try again later.",
        )),
    }
}

pub fn endpoints() -> Vec<Route> {
    routes![payments_page, update_card_page, update_card, delete_card]
}
