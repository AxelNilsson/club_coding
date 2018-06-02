use bcrypt::{hash, DEFAULT_COST};
use rocket::request::{FlashMessage, Form};
use rocket_contrib::Template;
use rocket::response::{Flash, Redirect};
use club_coding::create_new_user;
use database::DbConn;
use rocket::{Route, State};
use users::User as UserStruct;
use custom_csrf::{csrf_matches, CsrfCookie, CsrfToken};
use structs::PostmarkToken;
use structs::EmailRegex;
use authentication::{login, verify};

/// GET Endpoint to signup to the site.
/// Endpoints checks if the user is
/// logged in by using the user
/// request guard. If the user is
/// logged in it redirect the
/// user to the index, otherwise
/// it forwards the request.
#[get("/signup")]
fn signup_page_loggedin(_userid: UserStruct) -> Redirect {
    Redirect::to("/")
}

/// GET Endpoint for the signup page.
/// This endpoint will kick in
/// if the user is not logged in.
/// Takes in an optional FlashMessage
/// incase there is one.
/// Responds with the Signup Template
/// in the authentication folder.
#[get("/signup", rank = 2)]
fn signup_page(token: CsrfToken, flash: Option<FlashMessage>) -> Template {
    let (name, msg) = match flash {
        Some(flash) => (flash.name().to_string(), flash.msg().to_string()),
        None => ("".to_string(), "".to_string()),
    };
    let context = login::LoginContext {
        header: "Login Page!",
        csrf: token.value(),
        flash_name: name,
        flash_msg: msg,
    };
    Template::render("authentication/signup", &context)
}

/// Struct for parsing registration forms
#[derive(FromForm)]
struct UserRegistration {
    /// The username for the user
    username: String,
    /// The password of the user
    password: String,
    /// The confirmation password of the user
    confirm_password: String,
    /// The email of the user
    email: String,
    /// CSRF Token from the form
    csrf: String,
}

/// POST Endpoint for the page to signup.
/// It requires all of the parameters in the
/// UserRegistration struct to be submitted as
/// a form. If everything is successful, it will
/// send an verification email and redirect the user
/// to the index. Otherwise it will redirect to the
/// signup endpoint with an appropriate message.
#[post("/signup", data = "<user>")]
fn register_user(
    conn: DbConn,
    email_regex: State<EmailRegex>,
    postmark_token: State<PostmarkToken>,
    csrf_cookie: CsrfCookie,
    user: Form<UserRegistration>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let input: UserRegistration = user.into_inner();
    if email_regex.0.is_match(&input.email) {
        if input.password == input.confirm_password {
            if csrf_matches(input.csrf, csrf_cookie.value()) {
                match hash(&input.password, DEFAULT_COST) {
                    Ok(hashed_password) => match create_new_user(
                        &*conn,
                        &input.username,
                        &hashed_password,
                        &input.email,
                    ) {
                        Ok(new_user) => match verify::send_verify_email(
                            &*conn,
                            &postmark_token.0,
                            new_user.id,
                            input.email,
                        ) {
                            Ok(_) => Ok(Flash::success(
                                Redirect::to("/"),
                                "Registration successful! Please check your email.",
                            )),
                            Err(_) => Err(Flash::error(
                                Redirect::to("/signup"),
                                "An error occured, please try again later.",
                            )),
                        },
                        Err(_) => Err(Flash::error(
                            Redirect::to("/signup"),
                            "An error occured, please try again later.",
                        )),
                    },
                    Err(_) => Err(Flash::error(
                        Redirect::to("/signup"),
                        "An error occured, please try again later.",
                    )),
                }
            } else {
                Err(Flash::error(Redirect::to("/signup"), "CSRF Failed."))
            }
        } else {
            Err(Flash::error(
                Redirect::to("/signup"),
                "Passwords don't match.",
            ))
        }
    } else {
        Err(Flash::error(Redirect::to("/signup"), "Email is not valid."))
    }
}

/// Assembles all of the endpoints.
/// The upside of assembling all of the endpoints here
/// is that we don't have to update the main function but
/// instead we can keep all of the changes in here.
pub fn endpoints() -> Vec<Route> {
    routes![register_user, signup_page_loggedin, signup_page,]
}
