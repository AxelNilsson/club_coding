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
use payment::database::{get_charges, get_customer};
use custom_csrf::{csrf_matches, CsrfCookie, CsrfToken};

#[cfg(test)]
mod tests;

#[derive(Serialize)]
pub struct Charge {
    /// Amount of the charge.
    /// The amount is defined
    /// by USD * 100 and therefor
    /// is not a float.
    amount: i32,
    /// The date in a string format.
    date: String,
    /// The name of the series.
    series: String,
}

#[derive(Serialize)]
struct PaymentsContext<'a> {
    /// Header used in tera templates.
    /// Mainly used for the title.
    header: &'a str,
    /// The user struct used by templates.
    /// For example the username for the toolbar.
    user: User,
    /// Flash name if the request is redirected
    /// with one.
    flash_name: String,
    /// Flash message if the request is redirected
    /// with one.
    flash_msg: String,
    /// Vector of charges belonging
    /// to the User.
    charges: Vec<Charge>,
}

/// GET Endpoint to view payment data
/// belonging to the logged in user.
/// Endpoints checks if the user is
/// logged in by using the user
/// request guard. If the user is
/// not logged in it forwards the
/// request. If everything is succesful
/// it will render the payment page
/// in the payment directory otherwise
/// it will redirect to the add card
/// update page.
#[get("/")]
fn payments_page(
    conn: DbConn,
    user: User,
    flash: Option<FlashMessage>,
) -> Result<Template, Redirect> {
    match get_customer(&conn, user.id) {
        Some(_) => {
            let charges = get_charges(&conn, user.id);
            let (name, msg) = match flash {
                Some(flash) => (flash.name().to_string(), flash.msg().to_string()),
                None => ("".to_string(), "".to_string()),
            };
            let context = PaymentsContext {
                header: "Payments",
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

/// GET Endpoint to update the cards
/// belonging to the logged in user.
/// Endpoints checks if the user is
/// logged in by using the user
/// request guard. If the user is
/// not logged in it forwards the
/// request. If everything is succesful
/// it will render the update card page
/// it will redirect to the add card
/// update page.
#[get("/card/update")]
fn update_card_page(
    conn: DbConn,
    user: User,
    stripe_token: State<StripeToken>,
    flash: Option<FlashMessage>,
    token: CsrfToken,
) -> Result<Template, Flash<Redirect>> {
    match get_customer(&conn, user.id) {
        Some(_) => {
            let (name, msg) = match flash {
                Some(flash) => (flash.name().to_string(), flash.msg().to_string()),
                None => ("".to_string(), "".to_string()),
            };
            let context = ChargeContext {
                header: "Update card",
                csrf: token.value(),
                user: user,
                publishable_key: &stripe_token.publishable_key,
                flash_name: name,
                flash_msg: msg,
            };
            Ok(Template::render("payment/update_card", &context))
        }
        None => Err(Flash::error(
            Redirect::to("/card/add"),
            "No card found on account.",
        )),
    }
}

/// POST Endpoint to update the cards
/// belonging to the logged in user
/// and sends that user an email telling
/// the user that their card was updated.
/// Endpoints checks if the user is
/// logged in by using the user
/// request guard. If the user is
/// not logged in it forwards the
/// request. If everything is succesful
/// it will redirect to the index otherwise
/// it will redirect to the card update page.
#[post("/card/update", data = "<form_data>")]
fn update_card(
    conn: DbConn,
    user: User,
    csrf_cookie: CsrfCookie,
    stripe_token: State<StripeToken>,
    postmark: State<PostmarkToken>,
    form_data: Form<Stripe>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match get_customer(&conn, user.id) {
        Some(_) => {
            let data = form_data.into_inner();
            if !csrf_matches(&data.csrf, &csrf_cookie.value()) {
                return Err(Flash::error(
                    Redirect::to("/settings/payment/card/update"),
                    "CSRF Failed.",
                ));
            }
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
        None => Err(Flash::error(
            Redirect::to("/settings/payment/card/update"),
            "No card found on account.",
        )),
    }
}

/// POST Endpoint to delete all cards
/// belonging to the logged in user
/// and sends that user an email telling
/// the user that their card was deleted.
/// Endpoints checks if the user is
/// logged in by using the user
/// request guard. If the user is
/// not logged in it forwards the
/// request. If everything is succesful
/// it will redirect to the index otherwise
/// it will redirect to the payment page.
#[post("/card/delete")]
fn delete_card(
    conn: DbConn,
    postmark: State<PostmarkToken>,
    user: User,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match get_customer(&conn, user.id) {
        Some(_) => match delete(&conn, &postmark.0, user.id, user.email) {
            Ok(()) => Ok(Flash::success(Redirect::to("/"), "Oh no! Card deleted.")),
            _ => Err(Flash::error(
                Redirect::to("/settings/payment"),
                "An error occured, please try again later.",
            )),
        },
        None => Err(Flash::error(
            Redirect::to("/card/add"),
            "No card found on account.",
        )),
    }
}

/// Assembles all of the endpoints.
/// The upside of assembling all of the endpoints here
/// is that we don't have to update the main function but
/// instead we can keep all of the changes in here.
pub fn endpoints() -> Vec<Route> {
    routes![payments_page, update_card_page, update_card, delete_card]
}
