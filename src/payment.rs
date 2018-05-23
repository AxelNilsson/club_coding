use rocket::Route;
use rocket::request::Form;
use rocket_contrib::Template;
use rocket::response::{Flash, Redirect};
use club_coding::{establish_connection, insert_new_card, insert_new_users_stripe_token};
use users::User;
use stripe;
use std;
use rocket::request::FlashMessage;
use club_coding::models::{UsersStripeCard, UsersStripeCustomer};
use diesel::prelude::*;

#[derive(Serialize)]
struct ChargeContext {
    header: String,
    user: User,
    flash_name: String,
    flash_msg: String,
}

fn customer_exists(uid: i64) -> Option<UsersStripeCustomer> {
    use club_coding::schema::users_stripe_customer::dsl::*;

    let connection = establish_connection();

    let user: Vec<UsersStripeCustomer> = users_stripe_customer
        .filter(user_id.eq(uid))
        .limit(1)
        .load::<UsersStripeCustomer>(&connection)
        .expect("Error loading users");

    if user.len() == 1 {
        Some(user[0].clone())
    } else {
        None
    }
}

#[get("/")]
fn payments_page(user: User, flash: Option<FlashMessage>) -> Result<Template, Redirect> {
    match customer_exists(user.id) {
        Some(_) => {
            let (name, msg) = match flash {
                Some(flash) => (flash.name().to_string(), flash.msg().to_string()),
                None => ("".to_string(), "".to_string()),
            };
            let context = ChargeContext {
                header: "Club Coding".to_string(),
                user: user,
                flash_name: name,
                flash_msg: msg,
            };
            Ok(Template::render("payment", &context))
        }
        None => Err(Redirect::to("/card/add")),
    }
}

#[get("/card/update")]
fn update_card_page(user: User, flash: Option<FlashMessage>) -> Template {
    let (name, msg) = match flash {
        Some(flash) => (flash.name().to_string(), flash.msg().to_string()),
        None => ("".to_string(), "".to_string()),
    };
    let context = ChargeContext {
        header: "Club Coding".to_string(),
        user: user,
        flash_name: name,
        flash_msg: msg,
    };
    Template::render("update_card", &context)
}

#[derive(Debug, FromForm)]
struct Stripe {
    card_address_city: Option<String>,
    card_address_country: Option<String>,
    card_address_line1: Option<String>,
    card_address_line1_check: Option<String>,
    card_address_line2: Option<String>,
    card_address_state: Option<String>,
    card_address_zip: Option<String>,
    card_address_zip_check: Option<String>,
    card_brand: String,
    card_country: String,
    card_cvc_check: Option<String>,
    card_dynamic_last4: Option<String>,
    card_exp_month: i32,
    card_exp_year: i32,
    card_funding: Option<String>,
    card_id: Option<String>,
    card_last4: String,
    card_metadata: Option<String>,
    card_name: Option<String>,
    card_object: Option<String>,
    card_tokenization_method: Option<String>,
    client_ip: String,
    created: i64,
    id: String,
    livemode: bool,
    object: Option<String>,
    #[form(field = "type")]
    type_of_payment: Option<String>,
    used: bool,
}

fn get_customer(connection: &MysqlConnection, uid: i64) -> UsersStripeCustomer {
    use club_coding::schema::users_stripe_customer::dsl::*;

    users_stripe_customer
        .filter(user_id.eq(uid))
        .first(connection)
        .expect("Error loading user")
}

fn update_customer(
    client: &stripe::Client,
    customer_id: &str,
    token: &str,
) -> Result<stripe::Customer, stripe::Error> {
    // Create the customer
    stripe::Customer::update(
        &client,
        customer_id,
        stripe::CustomerParams {
            source: Some(stripe::CustomerSource::Token(token)),
            email: None,
            account_balance: None,
            business_vat_id: None,
            coupon: None,
            description: None,
            metadata: None,
            shipping: None,
        },
    )
}

fn charge(data: &Stripe, user_id: i64) -> Result<(), std::io::Error> {
    let connection = establish_connection();
    let customer = get_customer(&connection, user_id);
    insert_new_card(
        &connection,
        user_id,
        data.card_address_city.clone(),
        data.card_address_country.clone(),
        data.card_address_line1.clone(),
        data.card_address_line1_check.clone(),
        data.card_address_line2.clone(),
        data.card_address_state.clone(),
        data.card_address_zip.clone(),
        data.card_address_zip_check.clone(),
        data.card_brand.clone(),
        data.card_country.clone(),
        data.card_cvc_check.clone(),
        data.card_dynamic_last4.clone(),
        data.card_exp_month,
        data.card_exp_year,
        data.card_funding.clone(),
        data.card_id.clone(),
        data.card_last4.clone(),
        data.card_metadata.clone(),
        data.card_name.clone(),
        data.card_object.clone(),
        data.card_tokenization_method.clone(),
    );
    insert_new_users_stripe_token(
        &connection,
        user_id,
        data.client_ip.clone(),
        data.created.clone(),
        data.id.clone(),
        data.livemode,
        data.object.clone(),
        data.type_of_payment.clone(),
        data.used,
    );
    let client = stripe::Client::new("sk_test_cztFtKdeTEnlPLL6DpvkbjFf");
    update_customer(&client, &customer.uuid, &(data.id.clone())).unwrap();
    Ok(())
}

#[post("/card/update", data = "<form_data>")]
fn update_card(user: User, form_data: Form<Stripe>) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let data = form_data.into_inner();
    match charge(&data, user.id) {
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

fn delete_and_get_card(connection: &MysqlConnection, uid: i64) -> Option<String> {
    use club_coding::schema::users_stripe_card::dsl::*;

    let card: UsersStripeCard = users_stripe_card
        .filter(user_id.eq(uid))
        .first(connection)
        .expect("Error loading user");

    diesel::delete(users_stripe_card.find(card.id))
        .execute(connection)
        .unwrap();

    return card.card_id;
}

fn delete(user_id: i64) -> Result<(), std::io::Error> {
    let connection = establish_connection();
    let _card = delete_and_get_card(&connection, user_id);
    Ok(())
}

#[post("/card/delete")]
fn delete_card(user: User) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match delete(user.id) {
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
