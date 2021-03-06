use club_coding::{insert_new_card, insert_new_users_stripe_customer, insert_new_users_stripe_token};
use stripe;
use email::{EmailBody, PostmarkClient};
use database::DbConn;
use std::io::{Error, ErrorKind};
use charge::Stripe;

/// Function creates a user at
/// Stripe and returns it or, if
/// it fails, returns an error.
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

/// Struct for emails, not used
/// for updated card email but we
/// still need an empty struct
/// to use tera.
#[derive(Serialize)]
struct VerifyEmail<'a> {
    /// Verify email token
    /// to be rendered using tera.
    token: &'a str,
}

/// Function to send email to
/// the user when a card has
/// been added to the user.
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

/// Function to insert new card for
/// user and create the customer with
/// the new card at Stripe.
pub fn charge(
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
