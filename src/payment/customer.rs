use club_coding::{insert_new_card, insert_new_users_stripe_token};
use database::DbConn;
use stripe;
use email::{EmailBody, PostmarkClient};
use std::io::{Error, ErrorKind};
use charge::Stripe;
use payment::database::{delete_and_get_card, get_customer};

pub fn update_customer(
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

#[derive(Serialize)]
struct VerifyEmail<'a> {
    token: &'a str,
}

pub fn send_card_updated_mail(postmark_token: &str, email: String) -> Result<(), Error> {
    let tera = compile_templates!("templates/emails/**/*");
    let verify = VerifyEmail { token: "" };
    match tera.render("card_updated.html.tera", &verify) {
        Ok(html_body) => {
            let body = EmailBody {
                from: "axel@clubcoding.com".to_string(),
                to: email,
                subject: Some("Card updated!".to_string()),
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

pub fn charge(
    connection: &DbConn,
    stripe_secret: &str,
    postmark_token: &str,
    data: &Stripe,
    user_id: i64,
    email: String,
) -> Result<(), Error> {
    let customer = get_customer(&connection, user_id);
    let _ = insert_new_card(
        &*connection,
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
        &*connection,
        user_id,
        &data.client_ip,
        data.created,
        &data.id,
        data.livemode,
        data.object.as_ref().map_or(None, |x| Some(x)),
        data.type_of_payment.as_ref().map_or(None, |x| Some(x)),
        data.used,
    )?;
    let client = stripe::Client::new(stripe_secret);
    match customer {
        Some(customer) => match update_customer(&client, &customer.uuid, &data.id) {
            Ok(_) => {
                send_card_updated_mail(postmark_token, email)?;
                Ok(())
            }
            Err(_) => Err(Error::new(ErrorKind::Other, "Could not update customer")),
        },
        None => Err(Error::new(ErrorKind::Other, "Could not get customer")),
    }
}

pub fn send_card_deleted_mail(postmark_token: &str, email: String) -> Result<(), Error> {
    let tera = compile_templates!("templates/emails/**/*");
    let verify = VerifyEmail { token: "" };
    match tera.render("card_deleted.html.tera", &verify) {
        Ok(html_body) => {
            let body = EmailBody {
                from: "axel@clubcoding.com".to_string(),
                to: email,
                subject: Some("Card deleted!".to_string()),
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

pub fn delete(
    connection: &DbConn,
    postmark_token: &str,
    user_id: i64,
    email: String,
) -> Result<(), Error> {
    let _card = delete_and_get_card(connection, user_id);
    send_card_deleted_mail(postmark_token, email)?;
    Ok(())
}
