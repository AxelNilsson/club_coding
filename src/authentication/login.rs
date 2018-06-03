use bcrypt::verify;
use rocket::request::{FlashMessage, Form};
use rocket_contrib::Template;
use rocket::response::{Flash, Redirect};
use club_coding::create_new_user_session;
use database::DbConn;
use rocket::http::{Cookie, Cookies};
use time::Duration;
use rocket::Route;
use users::User as UserStruct;
use custom_csrf::{csrf_matches, CsrfCookie, CsrfToken};
use authentication;

/// Struct for parsing login forms
#[derive(FromForm)]
struct User {
    /// The username for the user
    username: String,
    /// The password of the user
    password: String,
    /// CSRF Token from the form
    csrf: String,
}

/// GET Endpoint to login to the site.
/// Endpoints checks if the user is
/// logged in by using the user
/// request guard. If the user is
/// logged in it redirect the
/// user to the index, otherwise
/// it forwards the request.
#[get("/login")]
fn login_page_loggedin(_user: UserStruct) -> Redirect {
    Redirect::to("/")
}

/// Context for login page
#[derive(Serialize)]
pub struct LoginContext<'a> {
    /// Header used in tera templates.
    /// Mainly used for the title.
    pub header: &'a str,
    /// CSRF Token. Used as a hidden
    /// input in the form.
    pub csrf: String,
    /// Flash name if the request is redirected
    /// with one.
    pub flash_name: String,
    /// Flash message if the request is redirected
    /// with one.
    pub flash_msg: String,
}

/// GET Endpoint for the login page.
/// This endpoint will kick in
/// if the user is not logged in.
/// Takes in an optional FlashMessage
/// incase there is one.
/// Responds with the Login Template
/// in the authentication folder.
#[get("/login", rank = 2)]
fn login_page(token: CsrfToken, flash: Option<FlashMessage>) -> Template {
    let (name, msg) = match flash {
        Some(flash) => (flash.name().to_string(), flash.msg().to_string()),
        None => ("".to_string(), "".to_string()),
    };
    let context = LoginContext {
        header: "Login Page!",
        csrf: token.value(),
        flash_name: name,
        flash_msg: msg,
    };
    Template::render("authentication/login", &context)
}

/// POST Endpoint for the page to login.
/// It requires all of the parameters in the
/// User struct to be submitted as a form.
/// If everything is successful, it will set
/// a Session Token and redirect the user to
/// the index. Otherwise it will redirect
/// back to the login page with an appropriate
/// message.
#[post("/login", data = "<user>")]
fn login(
    conn: DbConn,
    csrf_cookie: CsrfCookie,
    mut cookies: Cookies,
    user: Form<User>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let input_data: User = user.into_inner();
    if !csrf_matches(&input_data.csrf, &csrf_cookie.value()) {
        return Err(Flash::error(Redirect::to("/login"), "CSRF Failed."));
    }

    let password_hash: String = match authentication::database::get_password_hash_from_username(
        &conn,
        &input_data.username,
    ) {
        Ok(password_hash) => password_hash,
        Err(_) => return Err(Flash::error(Redirect::to("/login"), "No user found")),
    };

    let passwords_match: bool = match verify(&input_data.password, &password_hash) {
        Ok(passwords_match) => passwords_match,
        Err(_) => return Err(Flash::error(Redirect::to("/login"), "An error occurred")),
    };

    if !passwords_match {
        return Err(Flash::error(Redirect::to("/login"), "Password incorrect"));
    }

    let user_id =
        match authentication::database::get_user_id_from_username(&conn, &input_data.username) {
            Ok(user_id) => user_id,
            Err(_) => return Err(Flash::error(Redirect::to("/login"), "User not verified")),
        };

    let session_token = authentication::generate_token(64);
    match create_new_user_session(&*conn, user_id, &session_token) {
        Ok(_) => {
            let mut c = Cookie::new("session_token", session_token);
            c.set_max_age(Duration::hours(24));
            cookies.add_private(c);
            Ok(Flash::success(Redirect::to("/"), "You're now logged in."))
        }
        Err(_) => Err(Flash::error(
            Redirect::to("/login"),
            "An error occured, please try again later.",
        )),
    }
}

#[get("/logout")]
fn logout(mut cookies: Cookies) -> Redirect {
    cookies.remove_private(Cookie::named("session_token"));
    Redirect::to("/")
}

/// Assembles all of the endpoints.
/// The upside of assembling all of the endpoints here
/// is that we don't have to update the main function but
/// instead we can keep all of the changes in here.
pub fn endpoints() -> Vec<Route> {
    routes![login_page, login_page_loggedin, login, logout]
}
