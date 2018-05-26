use rocket::Route;
use rocket::request::Form;
use rocket_contrib::Template;
use rocket::response::{Flash, Redirect};
use club_coding::{insert_new_card, insert_new_users_stripe_token};
use database::DbConn;
use users::User;
use stripe;
use rocket::request::FlashMessage;
use club_coding::models::{Series, UsersStripeCard, UsersStripeCharge, UsersStripeCustomer};
use chrono::NaiveDateTime;
use email::{EmailBody, PostmarkClient};
use diesel::prelude::*;
use std::io::{Error, ErrorKind};

#[derive(Serialize)]
struct ChargeContext {
    header: String,
    user: User,
    flash_name: String,
    flash_msg: String,
}

fn customer_exists(connection: &DbConn, uid: i64) -> Option<UsersStripeCustomer> {
    use club_coding::schema::users_stripe_customer::dsl::*;

    match users_stripe_customer
        .filter(user_id.eq(uid))
        .limit(1)
        .load::<UsersStripeCustomer>(&**connection)
    {
        Ok(user) => {
            if user.len() == 1 {
                Some(user[0].clone())
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

#[derive(Serialize)]
struct Charge {
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

fn get_charges(connection: &DbConn, uid: i64) -> Vec<Charge> {
    use club_coding::schema::users_stripe_charge::dsl::*;

    match users_stripe_charge
        .filter(user_id.eq(uid))
        .order(id.desc())
        .load::<UsersStripeCharge>(&**connection)
    {
        Ok(charges) => {
            let mut to_return: Vec<Charge> = vec![];
            for charge in charges {
                use club_coding::schema::series::dsl::*;

                let serie: Option<Series> =
                    match series.filter(id.eq(charge.series_id)).first(&**connection) {
                        Ok(serie) => Some(serie),
                        Err(_) => None,
                    };

                match serie {
                    Some(serie) => {
                        to_return.push(Charge {
                            amount: charge.amount,
                            date: NaiveDateTime::from_timestamp(charge.created_at_stripe, 0)
                                .to_string(),
                            series: serie.title,
                        });
                    }
                    None => {}
                }
            }
            to_return
        }
        Err(_) => vec![],
    }
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

fn get_customer(connection: &DbConn, uid: i64) -> Option<UsersStripeCustomer> {
    use club_coding::schema::users_stripe_customer::dsl::*;

    match users_stripe_customer
        .filter(user_id.eq(uid))
        .first(&**connection)
    {
        Ok(user) => Some(user),
        Err(_) => None,
    }
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

fn send_card_updated_mail(email: String) -> Result<(), Error> {
    let body = EmailBody {
        from: "axel@clubcoding.com".to_string(),
        to: email,
        subject: Some("Card updated!".to_string()),
        html_body: Some(
            "<html><body>A card has been updated on your account.</body></html>".to_string(),
        ),
        cc: None,
        bcc: None,
        tag: None,
        text_body: None,
        reply_to: None,
        headers: None,
        track_opens: None,
        track_links: None,
    };
    let postmark_client = PostmarkClient::new("5f60334c-c829-45c6-aa34-08144c70559c");
    postmark_client.send_email(&body)?;
    Ok(())
}

fn charge(connection: &DbConn, data: &Stripe, user_id: i64, email: String) -> Result<(), Error> {
    let customer = get_customer(&connection, user_id);
    let _ = insert_new_card(
        &*connection,
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
    )?;
    let _ = insert_new_users_stripe_token(
        &*connection,
        user_id,
        data.client_ip.clone(),
        data.created.clone(),
        data.id.clone(),
        data.livemode,
        data.object.clone(),
        data.type_of_payment.clone(),
        data.used,
    )?;
    let client = stripe::Client::new("sk_test_cztFtKdeTEnlPLL6DpvkbjFf");
    match customer {
        Some(customer) => match update_customer(&client, &customer.uuid, &(data.id.clone())) {
            Ok(_) => {
                send_card_updated_mail(email)?;
                Ok(())
            }
            Err(_) => Err(Error::new(ErrorKind::Other, "Could not update customer")),
        },
        None => Err(Error::new(ErrorKind::Other, "Could not get customer")),
    }
}

#[post("/card/update", data = "<form_data>")]
fn update_card(
    conn: DbConn,
    user: User,
    form_data: Form<Stripe>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let data = form_data.into_inner();
    match charge(&conn, &data, user.id, user.email.clone()) {
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

fn delete_and_get_card(connection: &DbConn, uid: i64) -> Option<String> {
    use club_coding::schema::users_stripe_card::dsl::*;

    let card: Option<UsersStripeCard> = match users_stripe_card
        .filter(user_id.eq(uid))
        .first(&**connection)
    {
        Ok(card) => Some(card),
        Err(_) => None,
    };

    match card {
        Some(card) => {
            match diesel::delete(users_stripe_card.find(card.id)).execute(&**connection) {
                Ok(_) => card.card_id,
                Err(_) => None,
            }
        }
        None => None,
    }
}

fn send_card_deleted_mail(email: String) -> Result<(), Error> {
    let body = EmailBody {
        from: "axel@clubcoding.com".to_string(),
        to: email,
        subject: Some("Card deleted!".to_string()),
        html_body: Some(
            "<html><body>A card has been deleted from your account.</body></html>".to_string(),
        ),
        cc: None,
        bcc: None,
        tag: None,
        text_body: None,
        reply_to: None,
        headers: None,
        track_opens: None,
        track_links: None,
    };
    let postmark_client = PostmarkClient::new("5f60334c-c829-45c6-aa34-08144c70559c");
    postmark_client.send_email(&body)?;
    Ok(())
}

fn delete(connection: &DbConn, user_id: i64, email: String) -> Result<(), Error> {
    let _card = delete_and_get_card(connection, user_id);
    send_card_deleted_mail(email)?;
    Ok(())
}

#[post("/card/delete")]
fn delete_card(conn: DbConn, user: User) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match delete(&conn, user.id, user.email.clone()) {
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
