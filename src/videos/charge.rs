use club_coding::{create_new_user_series_access, insert_new_users_stripe_charge};
use club_coding::models::UsersStripeCustomer;
use users::User;
use std::io::{Error, ErrorKind};
use stripe::Source::Card;
use database::DbConn;
use videos::database;
use email::{EmailBody, PostmarkClient};

#[derive(Serialize)]
struct VerifyEmail<'a> {
    token: &'a str,
}

fn send_bought_email(postmark_token: &str, email: &str) -> Result<(), Error> {
    let tera = compile_templates!("templates/emails/**/*");
    let verify = VerifyEmail { token: "" };
    match tera.render("series_bought.html.tera", &verify) {
        Ok(html_body) => {
            let body = EmailBody {
                from: "axel@clubcoding.com".to_string(),
                to: email.to_string(),
                subject: Some("Series bought!".to_string()),
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

pub fn charge_card(
    conn: &DbConn,
    stripe_secret: &str,
    postmark_token: &str,
    series_id: i64,
    user: &User,
    stripe_customer: &UsersStripeCustomer,
) -> Result<(), Error> {
    let serie = match database::get_serie(&conn, series_id) {
        Some(serie) => serie,
        None => return Err(Error::new(ErrorKind::Other, "no serie")),
    };
    match stripe_customer.default_source {
        Some(ref customer_source) => {
            // Create the customer
            let client = stripe::Client::new(stripe_secret);

            let charge = match stripe::Charge::create(
                &client,
                stripe::ChargeParams {
                    amount: Some(serie.price as u64),
                    currency: Some(stripe::Currency::USD),
                    application_fee: None,
                    capture: None,
                    description: None,
                    destination: None,
                    fraud_details: None,
                    transfer_group: None,
                    on_behalf_of: None,
                    metadata: None,
                    receipt_email: None,
                    shipping: None,
                    customer: Some(stripe_customer.uuid.clone()),
                    source: Some(stripe::CustomerSource::Token(&customer_source)),
                    statement_descriptor: None,
                },
            ) {
                Ok(charge) => charge,
                Err(_) => return Err(Error::new(ErrorKind::Other, "couldn't create charge")),
            };
            let failure_code: Option<String> = match charge.failure_code {
                Some(code) => Some(code.to_string()),
                None => None,
            };
            let source_id = match charge.source {
                Card(card) => card.id,
            };
            let _ = insert_new_users_stripe_charge(
                &*conn,
                user.id,
                series_id,
                &charge.id,
                charge.amount as i32,
                charge.amount_refunded as i32,
                charge
                    .balance_transaction
                    .as_ref()
                    .map_or(None, |x| Some(x)),
                charge.captured,
                charge.created,
                charge.description.as_ref().map_or(None, |x| Some(x)),
                charge.destination.as_ref().map_or(None, |x| Some(x)),
                charge.dispute.as_ref().map_or(None, |x| Some(x)),
                failure_code.as_ref().map_or(None, |x| Some(x)),
                charge.failure_message.as_ref().map_or(None, |x| Some(x)),
                charge.livemode,
                charge.on_behalf_of.as_ref().map_or(None, |x| Some(x)),
                charge.order.as_ref().map_or(None, |x| Some(x)),
                charge.paid,
                charge.refunded,
                &source_id,
                charge.source_transfer.as_ref().map_or(None, |x| Some(x)),
                charge
                    .statement_descriptor
                    .as_ref()
                    .map_or(None, |x| Some(x)),
                &charge.status,
            )?;
            let _ = create_new_user_series_access(&*conn, user.id, series_id, true)?;
            let _ = send_bought_email(postmark_token, &user.email)?;
            Ok(())
        }
        None => Err(Error::new(ErrorKind::Other, "no customer_source")),
    }
}
