use club_coding::{create_new_request_network_payments, create_new_user_series_access,
                  insert_new_users_stripe_charge};
use club_coding::models::UsersStripeCustomer;
use users::User;
use std::io::{Error, ErrorKind};
use stripe::Source::Card;
use database::DbConn;
use videos::database;
use email::{EmailBody, PostmarkClient};
use request_network::{wooreq_request, ReqBody};
use authentication;
use series;

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

/// Sends an email if the purchase
/// of a series is succesful.
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

/// Charges the card that is sent in through the
/// Stripe Customer by the amount the series costs
/// as specified in the MySQL Database.
/// Returns an OK or the error.
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

/// Generates a Request Network Payment
/// with the use of the WooREQ website
/// and inserts it into the database.
pub fn generate_and_create_req_payment(
    conn: &DbConn,
    uuid: &str,
    user_id: i64,
    serie_id: i64,
) -> Result<String, Error> {
    let token = authentication::generate_token(30);

    let serie = match series::database::get_serie_by_id(conn, serie_id) {
        Some(serie) => serie,
        None => return Err(Error::new(ErrorKind::Other, "No serie found.")),
    };

    let new_payment_id = create_new_request_network_payments(
        conn,
        &token,
        user_id,
        serie_id,
        &(serie.price as f32 / (600 * 100) as f32).to_string(),
        "0xadB2A92a1dD0D95Fcf0d70b2272244BDbd686464",
        &format!("Buying \"{}\" at Club Coding.", serie.title),
    )?;

    let body = ReqBody {
        to_pay: &(serie.price as f32 / (600 * 100) as f32).to_string(),
        to_address: "0xadB2A92a1dD0D95Fcf0d70b2272244BDbd686464",
        redirect_url: &format!("https://clubcoding.com/watch/{}/buy/req/{}", uuid, token),
        order_id: &new_payment_id.to_string(),
        reason: &format!("Buying \"{}\" at Club Coding.", serie.title),
        network: 1,
    };

    wooreq_request(&body)
}

/// Validates the request network token
/// and returns either OK or an error.
/// Checks the validity of the token
/// sets used to false (invalidates it).
/// Grants access to the user and sends
/// email to the user thanking for the
/// purchase.
pub fn validate_req_bought(
    conn: &DbConn,
    user: User,
    postmark_token: &str,
    uuid: &str,
    token: &str,
) -> Result<(), Error> {
    match database::get_video_data_from_uuid(&conn, uuid) {
        Ok(video) => {
            let request_payment = match database::get_request_payment(conn, token) {
                Some(payment) => payment,
                None => return Err(Error::new(ErrorKind::Other, "Request Token doesn't exist.")),
            };
            if request_payment.used {
                return Err(Error::new(ErrorKind::Other, "Request Token already used."));
            }

            let serie = match series::database::get_serie_by_id(conn, video.serie_id) {
                Some(serie) => serie,
                None => return Err(Error::new(ErrorKind::Other, "Serie doesn't exist.")),
            };

            if serie.id != request_payment.serie_id {
                return Err(Error::new(
                    ErrorKind::Other,
                    "An error occured, please try again later.",
                ));
            }

            match create_new_user_series_access(&*conn, user.id, serie.id, true) {
                Ok(_) => {}
                Err(_) => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        "An error occured, please try again later.",
                    ));
                }
            }
            match send_bought_email(postmark_token, &user.email) {
                Ok(_) => {}
                Err(_) => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        "An error occured, please try again later.",
                    ))
                }
            }
            match database::invalidate_request_payment(&conn, request_payment.id) {
                Ok(_) => {}
                Err(_) => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        "An error occured, please try again later.",
                    ))
                }
            }
            Ok(())
        }
        Err(_video_not_found) => return Err(Error::new(ErrorKind::Other, "Video doesn't exist.")),
    }
}
