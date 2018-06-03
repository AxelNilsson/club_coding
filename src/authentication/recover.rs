use bcrypt::{hash, DEFAULT_COST};
use rocket::request::{FlashMessage, Form};
use rocket_contrib::Template;
use rocket::response::{Flash, Redirect};
use club_coding::create_new_users_recover_email;
use database::DbConn;
use rocket::Route;
use users::User as UserStruct;
use email::{EmailBody, PostmarkClient};
use custom_csrf::{csrf_matches, CsrfCookie, CsrfToken};
use std::io::{Error, ErrorKind};
use structs::PostmarkToken;
use rocket::State;
use structs::EmailRegex;
use authentication;
use authentication::verify::VerifyEmail;

/// GET Endpoint for the recover email
/// page. Endpoints checks if
/// the user is logged in by using
/// the user request guard. If the
/// user is logged in it redirect
/// the user to the index, otherwise
/// it forwards the request.
#[get("/recover/email")]
fn recover_email_loggedin_page(_userid: UserStruct) -> Redirect {
    Redirect::to("/")
}

/// GET Endpoint for the recover email
/// page. This endpoint will kick in
/// if the user is not logged in.
/// Takes in an optional FlashMessage
/// incase there is one. Responds with
/// the Send Recover Template in the
/// authentication folder.
#[get("/recover/email", rank = 2)]
fn recover_email_page(csrf_token: CsrfToken, flash: Option<FlashMessage>) -> Template {
    let (name, msg) = match flash {
        Some(flash) => (flash.name().to_string(), flash.msg().to_string()),
        None => ("".to_string(), "".to_string()),
    };
    let context = authentication::login::LoginContext {
        header: "recover_email",
        csrf: csrf_token.value(),
        flash_name: name,
        flash_msg: msg,
    };
    Template::render("authentication/send_recover", &context)
}

/// Function to send verification
/// email to the user when the user
/// has registered.
fn send_recover_mail(postmark_token: &str, token: &str, email: &str) -> Result<(), Error> {
    let tera = compile_templates!("templates/emails/**/*");
    let verify = VerifyEmail { token: token };
    match tera.render("recover_account.html.tera", &verify) {
        Ok(html_body) => {
            let body = EmailBody {
                from: "axel@clubcoding.com".to_string(),
                to: email.to_string(),
                subject: Some("Recover your account".to_string()),
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

/// POST Endpoint for the recover email
/// page. Endpoints checks if
/// the user is logged in by using
/// the user request guard. If the
/// user is logged in it redirect
/// the user to the index, otherwise
/// it forwards the request.
#[post("/recover/email")]
fn send_recover_email_loggedin(_userid: UserStruct) -> Redirect {
    Redirect::to("/")
}

/// Struct for parsing recover
/// account forms
#[derive(FromForm)]
struct RecoverAccount {
    /// The email of the user
    email: String,
    /// CSRF Token from the form
    csrf: String,
}

/// Generates a random token.
/// Inserts that token into the database.
/// Sends a recover account email to the
/// user with that token.
fn creates_and_sends_recover_email(
    connection: &DbConn,
    postmark_token: &str,
    user_id: i64,
    email: &str,
) -> Result<(), Error> {
    let token = authentication::generate_token(30);
    match create_new_users_recover_email(connection, user_id, &token) {
        Ok(_) => match send_recover_mail(postmark_token, &token, email) {
            Ok(_) => Ok(()),
            Err(_) => return Err(Error::new(ErrorKind::Other, "Could send recover email.")),
        },
        Err(_) => {
            return Err(Error::new(
                ErrorKind::Other,
                "Could not insert recovery data to database.",
            ))
        }
    }
}

/// POST Endpoint for the recover email
/// page. This endpoint will kick in
/// if the user is not logged in.
/// It requires all of the parameters
/// in the RecoverAccount struct to
/// be submitted as a form.
/// Checks if the CSRF matches and
/// the email is valid. If everything
/// succeeds it redirects to the index
/// otherwise it redirects to the index
/// and gives an appropriate message.
#[post("/recover/email", data = "<user>", rank = 2)]
fn send_recover_email(
    conn: DbConn,
    email_regex: State<EmailRegex>,
    postmark_token: State<PostmarkToken>,
    csrf_cookie: CsrfCookie,
    user: Form<RecoverAccount>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let input: RecoverAccount = user.into_inner();
    if !email_regex.0.is_match(&input.email) {
        return Err(Flash::error(
            Redirect::to("/recover/email"),
            "Email is not valid.",
        ));
    }
    if !csrf_matches(&input.csrf, &csrf_cookie.value()) {
        return Err(Flash::error(
            Redirect::to("/recover/email"),
            "CSRF Doesn't match.",
        ));
    }

    let user_id: i64 = match authentication::database::get_user_id_from_email(&conn, &input.email) {
        Some(user_id) => user_id,
        None => {
            return Err(Flash::error(
                Redirect::to("/recover/email"),
                "Email not found.",
            ))
        }
    };

    match creates_and_sends_recover_email(&conn, &postmark_token.0, user_id, &input.email) {
        Ok(_) => Ok(Flash::success(
            Redirect::to("/"),
            "Email sent. Please check your inbox.",
        )),
        Err(_) => Err(Flash::error(
            Redirect::to("/recover/email"),
            "An error occured, please try again later.",
        )),
    }
}

/// GET Endpoint for the recover email
/// page. Endpoints checks if
/// the user is logged in by using
/// the user request guard. If the
/// user is logged in it redirect
/// the user to the index, otherwise
/// it forwards the request.
#[get("/email/recover/<_uuid>")]
fn update_password_loggedin_page(_uuid: String, _userid: UserStruct) -> Redirect {
    Redirect::to("/")
}

/// GET Endpoint for the recover account
/// page. This endpoint will kick in
/// if the user is not logged in.
/// Checks if the UUID is in the
/// database and is valid. If it is,
/// it responds with the Recover Email
/// Template in the authentication folder.
/// Otherwise it redirects to the index with
/// an appropriate message.
#[get("/email/recover/<uuid>", rank = 2)]
fn update_password_page(
    conn: DbConn,
    csrf_token: CsrfToken,
    flash: Option<FlashMessage>,
    uuid: String,
) -> Result<Template, Flash<Redirect>> {
    let result = match authentication::database::get_recovery_by_token(&conn, &uuid) {
        Some(result) => result,
        None => return Err(Flash::error(Redirect::to("/"), "Link incorrect.")),
    };

    if result.used {
        return Err(Flash::error(Redirect::to("/"), "Link already used."));
    }

    let (name, msg) = match flash {
        Some(flash) => (flash.name().to_string(), flash.msg().to_string()),
        None => ("".to_string(), "".to_string()),
    };

    let context = authentication::login::LoginContext {
        header: "Recover Email",
        csrf: csrf_token.value(),
        flash_name: name,
        flash_msg: msg,
    };
    Ok(Template::render("authentication/recover_email", &context))
}

/// POST Endpoint for the recover email
/// page. Endpoints checks if
/// the user is logged in by using
/// the user request guard. If the
/// user is logged in it redirect
/// the user to the index, otherwise
/// it forwards the request.
#[post("/email/recover/<_uuid>")]
fn update_password_loggedin(_uuid: String, _userid: UserStruct) -> Redirect {
    Redirect::to("/")
}

/// Struct for parsing update
/// account password forms
#[derive(FromForm)]
struct UpdatePassword {
    /// The password of the user
    password: String,
    /// The confirmation password of the user
    confirm_password: String,
    /// CSRF Token from the form
    csrf: String,
}

/// POST Endpoint for the recover email
/// page. This endpoint will kick in
/// if the user is not logged in.
/// It requires all of the parameters
/// in the RecoverAccount struct to
/// be submitted as a form.
/// Checks if the UUID is in the
/// database and is valid.
/// Checks if the CSRF matches and
/// the email is valid. If everything
/// succeeds it updates the password in
/// the database and redirects to the
/// index otherwise it redirects to
/// the index or the current UUID page
/// and gives an appropriate message.
#[post("/email/recover/<uuid>", data = "<user>", rank = 2)]
fn update_password(
    conn: DbConn,
    uuid: String,
    csrf_cookie: CsrfCookie,
    user: Form<UpdatePassword>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let input: UpdatePassword = user.into_inner();
    if !(input.password == input.confirm_password) {
        return Err(Flash::error(
            Redirect::to(&format!("/email/recover/{}", uuid)),
            "Passwords not matching.",
        ));
    }

    let result = match authentication::database::get_recovery_by_token(&conn, &uuid) {
        Some(result) => result,
        None => return Err(Flash::error(Redirect::to("/"), "Link incorrect.")),
    };

    if result.used {
        return Err(Flash::error(Redirect::to("/"), "Link already used."));
    }

    if !csrf_matches(&input.csrf, &csrf_cookie.value()) {
        return Err(Flash::error(
            Redirect::to(&format!("/email/recover/{}", uuid)),
            "CSRF Doesn't match.",
        ));
    }

    let hashed_password = match hash(&input.password, DEFAULT_COST) {
        Ok(hashed_password) => hashed_password,
        Err(_) => {
            return Err(Flash::error(
                Redirect::to(&format!("/email/recover/{}", uuid)),
                "An error occured, please try again later.",
            ))
        }
    };

    match authentication::database::invalidate_token_and_update_password(
        &conn,
        result.id,
        result.user_id,
        &hashed_password,
    ) {
        Ok(_) => Ok(Flash::success(
            Redirect::to("/"),
            "Password updated, please sign in.",
        )),
        Err(_) => Err(Flash::error(
            Redirect::to(&format!("/email/recover/{}", uuid)),
            "An error occured, please try again later.",
        )),
    }
}

/// Assembles all of the endpoints.
/// The upside of assembling all of the endpoints here
/// is that we don't have to update the main function but
/// instead we can keep all of the changes in here.
pub fn endpoints() -> Vec<Route> {
    routes![
        update_password_loggedin_page,
        update_password_loggedin,
        update_password_page,
        update_password,
        recover_email_loggedin_page,
        send_recover_email_loggedin,
        recover_email_page,
        send_recover_email,
    ]
}
