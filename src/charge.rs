use rocket::Route;
use rocket::request::Form;
use rocket_contrib::Template;
use rocket::response::{Flash, Redirect};
use club_coding::{insert_new_card, insert_new_users_stripe_customer, insert_new_users_stripe_token};
use users::User;
use stripe;
use rocket::request::FlashMessage;
use email::{EmailBody, PostmarkClient};
use database::DbConn;
use structs::{PostmarkToken, StripeToken};
use rocket::State;
use std::io::{Error, ErrorKind};

#[derive(Serialize)]
struct ChargeContext {
    header: String,
    user: User,
    flash_name: String,
    flash_msg: String,
}

#[get("/card/add")]
fn add_card_page(user: User, flash: Option<FlashMessage>) -> Template {
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
    Template::render("add_card", &context)
}

#[get("/card/add/<_uuid>")]
fn add_card_uuid_page(user: User, flash: Option<FlashMessage>, _uuid: String) -> Template {
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
    Template::render("add_card", &context)
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

fn create_customer(
    client: &stripe::Client,
    email: &str,
    token: &str,
) -> Result<stripe::Customer, Error> {
    // Create the customer
    match stripe::Customer::create(
        &client,
        stripe::CustomerParams {
            email: Some(email),
            source: Some(stripe::CustomerSource::Token(token)),
            account_balance: None,
            business_vat_id: None,
            coupon: None,
            description: None,
            metadata: None,
            shipping: None,
        },
    ) {
        Ok(customer) => Ok(customer),
        Err(_) => Err(Error::new(ErrorKind::Other, "Could not create customer")),
    }
}

#[derive(Serialize)]
struct VerifyEmail<'a> {
    token: &'a str,
}

fn send_card_added_mail(postmark_token: &str, email: String) -> Result<(), Error> {
    let tera = compile_templates!("templates/emails/**/*");
    let verify = VerifyEmail { token: "" };
    match tera.render("card_added.html.tera", &verify) {
        Ok(html_body) => {
            let body = EmailBody {
                from: "axel@clubcoding.com".to_string(),
                to: email,
                subject: Some("Card added!".to_string()),
                html_body: Some(html_body),
                cc: None,
                bcc: None,
                tag: None,
                text_body: None,
                reply_to: None,
                headers: None,
                track_opens: None,
                track_links: None,
            };
            let postmark_client = PostmarkClient::new(postmark_token);
            postmark_client.send_email(&body)?;
            Ok(())
        }
        Err(_) => Err(Error::new(ErrorKind::Other, "couldn't render template")),
    }
}

fn charge(
    connection: &DbConn,
    stripe_secret: &str,
    postmark_token: &str,
    data: &Stripe,
    email: &str,
    user_id: i64,
) -> Result<(), Error> {
    let client = stripe::Client::new(stripe_secret);
    let _ = insert_new_card(
        &connection,
        user_id,
        data.card_address_city.as_ref().map_or(None, |x| Some(x)),
        data.card_address_country.as_ref().map_or(None, |x| Some(x)),
        data.card_address_line1.as_ref().map_or(None, |x| Some(x)),
        data.card_address_line1_check
            .as_ref()
            .map_or(None, |x| Some(x)),
        data.card_address_line2.as_ref().map_or(None, |x| Some(x)),
        data.card_address_state.as_ref().map_or(None, |x| Some(x)),
        data.card_address_zip.as_ref().map_or(None, |x| Some(x)),
        data.card_address_zip_check
            .as_ref()
            .map_or(None, |x| Some(x)),
        &data.card_brand,
        &data.card_country,
        data.card_cvc_check.as_ref().map_or(None, |x| Some(x)),
        data.card_dynamic_last4.as_ref().map_or(None, |x| Some(x)),
        data.card_exp_month,
        data.card_exp_year,
        data.card_funding.as_ref().map_or(None, |x| Some(x)),
        data.card_id.as_ref().map_or(None, |x| Some(x)),
        &data.card_last4,
        data.card_metadata.as_ref().map_or(None, |x| Some(x)),
        data.card_name.as_ref().map_or(None, |x| Some(x)),
        data.card_object.as_ref().map_or(None, |x| Some(x)),
        data.card_tokenization_method
            .as_ref()
            .map_or(None, |x| Some(x)),
    )?;
    let _ = insert_new_users_stripe_token(
        &connection,
        user_id,
        &data.client_ip,
        data.created,
        &data.id,
        data.livemode,
        data.object.as_ref().map_or(None, |x| Some(x)),
        data.type_of_payment.as_ref().map_or(None, |x| Some(x)),
        data.used,
    )?;
    match create_customer(&client, email, &data.id) {
        Ok(customer) => {
            let _ = insert_new_users_stripe_customer(
                &connection,
                user_id,
                &customer.id,
                customer.account_balance,
                customer.business_vat_id.as_ref().map_or(None, |x| Some(x)),
                customer.created as i64,
                customer.default_source.as_ref().map_or(None, |x| Some(x)),
                customer.delinquent,
                customer.desc.as_ref().map_or(None, |x| Some(x)),
                customer.email.as_ref().map_or(None, |x| Some(x)),
                customer.livemode,
            )?;
            match send_card_added_mail(postmark_token, email.to_string()) {
                Ok(_) => Ok(()),
                Err(_) => Err(Error::new(ErrorKind::Other, "Could not send email")),
            }
        }
        Err(err) => Err(err),
    }
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

pub fn endpoints() -> Vec<Route> {
    routes![add_card_page, add_card, add_card_uuid_page, add_card_uuid]
}
