use rocket::response::{Flash, Redirect};
use club_coding::create_new_users_verify_email;
use database::DbConn;
use rocket::Route;
use users::User as UserStruct;
use email::{EmailBody, PostmarkClient};
use std::io::{Error, ErrorKind};
use authentication;
use diesel::prelude::*;

/// Struct for emails, not used
/// for updated card email but we
/// still need an empty struct
/// to use tera.
#[derive(Serialize)]
pub struct VerifyEmail<'a> {
    /// Verify email token
    /// to be rendered using tera.
    pub token: &'a str,
}

/// Function to send verification
/// email to the user when the user
/// has registered.
pub fn send_verify_email(
    connection: &MysqlConnection,
    postmark_token: &str,
    user_id: i64,
    email: &str,
) -> Result<(), Error> {
    let token = authentication::generate_token(30);
    create_new_users_verify_email(connection, user_id, &token)?;
    let tera = compile_templates!("templates/emails/**/*");
    let verify = VerifyEmail { token: &token };
    match tera.render("verify_account.html.tera", &verify) {
        Ok(html_body) => {
            let body = EmailBody {
                from: "axel@clubcoding.com".to_string(),
                to: email.to_string(),
                subject: Some("Welcome to ClubCoding!".to_string()),
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

/// GET Endpoint to verify email at
/// the site. Endpoints checks if
/// the user is logged in by using
/// the user request guard. If the
/// user is logged in it redirect
/// the user to the index, otherwise
/// it forwards the request.
#[get("/email/verify/<_uuid>")]
fn verify_email_loggedin(_uuid: String, _userid: UserStruct) -> Redirect {
    Redirect::to("/")
}

/// GET Endpoint for the verify email
/// page. This endpoint will kick in
/// if the user is not logged in.
/// Checks if the UUID is in the
/// database and is valid. If it is,
/// it verifies the user and makes
/// the UUID invalid. If everything
/// works it redirects to the index and
/// otherwise it redirects to the index
/// with an appropriate error message.
#[get("/email/verify/<uuid>", rank = 2)]
fn verify_email(conn: DbConn, uuid: String) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let result = match authentication::database::get_verify_email_by_token(&conn, &uuid) {
        Some(result) => result,
        None => return Err(Flash::error(Redirect::to("/"), "Link incorrect.")),
    };
    if result.used {
        Err(Flash::error(Redirect::to("/"), "Link already used."))
    } else {
        match authentication::database::invalidate_token_and_verify_user(
            &conn,
            result.id,
            result.user_id,
        ) {
            Ok(_) => Ok(Flash::success(
                Redirect::to("/"),
                "Email verified, please sign in.",
            )),
            Err(_) => Err(Flash::error(
                Redirect::to("/"),
                "An error occured, please try again later.",
            )),
        }
    }
}

/// Assembles all of the endpoints.
/// The upside of assembling all of the endpoints here
/// is that we don't have to update the main function but
/// instead we can keep all of the changes in here.
pub fn endpoints() -> Vec<Route> {
    routes![verify_email_loggedin, verify_email,]
}
