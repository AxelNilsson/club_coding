use bcrypt::{hash, verify, DEFAULT_COST};
use rocket::request::Form;
use rocket_contrib::Template;
use rocket::response::Redirect;
use club_coding::{create_new_user, create_new_user_session, create_new_users_verify_email,
                  establish_connection};
use rocket::http::{Cookie, Cookies};
use time::Duration;
use rocket::Route;
use users::User as UserStruct;
use rand;
use email::{EmailBody, PostmarkClient};
use club_coding::models::{Users, UsersVerifyEmail};
use std;
use custom_csrf::{csrf_matches, CsrfCookie, CsrfToken};
use diesel::prelude::*;
use rocket::response::Flash;

#[derive(FromForm)]
struct User {
    username: String,
    password: String,
    csrf: String,
}

fn generate_token(length: u8) -> String {
    let bytes: Vec<u8> = (0..length).map(|_| rand::random::<u8>()).collect();
    let strings: Vec<String> = bytes.iter().map(|byte| format!("{:02X}", byte)).collect();
    return strings.join("");
}

fn get_password_hash_from_username(name: String) -> Result<String, std::io::Error> {
    use club_coding::schema::users::dsl::*;

    let connection = establish_connection();
    let results = users
        .filter(username.eq(name))
        .limit(1)
        .load::<Users>(&connection)
        .expect("Error loading users");

    if results.len() == 1 {
        return Ok(results[0].password.to_string());
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "No user found",
        ));
    }
}

fn get_user_id_from_username(name: String) -> Result<i64, std::io::Error> {
    use club_coding::schema::users::dsl::*;

    let connection = establish_connection();
    let results = users
        .filter(username.eq(name))
        .limit(1)
        .load::<Users>(&connection)
        .expect("Error loading users");

    if results.len() == 1 {
        return Ok(results[0].id);
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "No user found",
        ));
    }
}

#[get("/login")]
fn login_page_loggedin(_user: UserStruct) -> Redirect {
    Redirect::to("/")
}

#[get("/login", rank = 2)]
fn login_page(token: CsrfToken) -> Template {
    let context = CSRFContext {
        header: "Login Page!".to_string(),
        csrf: token.value(),
    };
    Template::render("index", &context)
}

#[post("/login", data = "<user>")]
fn login(
    csrf_cookie: CsrfCookie,
    mut cookies: Cookies,
    user: Form<User>,
) -> Result<Redirect, String> {
    let input_data: User = user.into_inner();
    if csrf_matches(input_data.csrf, csrf_cookie.value()) {
        match get_password_hash_from_username(input_data.username.clone()) {
            Ok(password_hash) => match verify(&input_data.password, &password_hash) {
                Ok(passwords_match) => {
                    if passwords_match {
                        let session_token = generate_token(64);
                        let connection = establish_connection();
                        let user_id = get_user_id_from_username(input_data.username).unwrap();
                        create_new_user_session(&connection, user_id, session_token.clone());
                        let mut c = Cookie::new("session_token", session_token);
                        c.set_max_age(Duration::hours(24));
                        cookies.add_private(c);
                        Ok(Redirect::to("/"))
                    } else {
                        Err(String::from("Password incorrect"))
                    }
                }
                Err(_) => Err(String::from("An error occurred")),
            },
            Err(_) => Err(String::from("No user in database")),
        }
    } else {
        Err(String::from("csrf failed"))
    }
}

#[post("/signup", data = "<user>")]
fn register_user(
    csrf_cookie: CsrfCookie,
    user: Form<User>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let input: User = user.into_inner();
    if csrf_matches(input.csrf, csrf_cookie.value()) {
        match hash(&input.password, DEFAULT_COST) {
            Ok(hashed_password) => {
                let connection = establish_connection();
                let new_user =
                    create_new_user(&connection, input.username.clone(), hashed_password);
                let token = generate_token(30);
                create_new_users_verify_email(&connection, new_user.id, token.clone());
                let body = EmailBody {
                    from: "axel@clubcoding.com".to_string(),
                    to: input.username.clone(),
                    subject: Some("Welcome to ClubCoding!".to_string()),
                    html_body: Some(format!("<html><body><a href='https://clubcoding.com/email/verify/{}'>Please press this link to confirm your e-mail.</a></body></html>", token)),
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
                postmark_client.send_email(&body).unwrap();

                Ok(Flash::success(
                    Redirect::to("/"),
                    "Registration successful! Please check your email.",
                ))
            }
            Err(_) => Err(Flash::error(
                Redirect::to("/signup"),
                "An error occured, please try again later.",
            )),
        }
    } else {
        Err(Flash::error(Redirect::to("/signup"), "CSRF Failed."))
    }
}

#[derive(Serialize)]
struct CSRFContext {
    header: String,
    csrf: String,
}

#[get("/signup")]
fn signup_page_loggedin(_userid: UserStruct) -> Redirect {
    Redirect::to("/")
}

#[get("/signup", rank = 2)]
fn signup_page(token: CsrfToken) -> Template {
    let context = CSRFContext {
        header: "Sign up!".to_string(),
        csrf: token.value(),
    };
    Template::render("signup", &context)
}

#[get("/email/verify/<uuid>")]
fn verify_email(uuid: String) -> Result<Flash<Redirect>, Flash<Redirect>> {
    use club_coding::schema::users_verify_email::dsl::*;

    let connection = establish_connection();
    let results = users_verify_email
        .filter(token.eq(uuid))
        .limit(1)
        .load::<UsersVerifyEmail>(&connection)
        .expect("Error loading users verify email");

    if results.len() == 1 {
        if results[0].used {
            Err(Flash::error(Redirect::to("/"), "Link already used."))
        } else {
            diesel::update(users_verify_email.find(results[0].id))
                .set(used.eq(true))
                .execute(&connection)
                .unwrap();

            use club_coding::schema::users::dsl::*;
            diesel::update(users.find(results[0].user_id))
                .set(verified.eq(true))
                .execute(&connection)
                .unwrap();

            Ok(Flash::success(
                Redirect::to("/"),
                "Email verified, please sign in.",
            ))
        }
    } else {
        Err(Flash::error(Redirect::to("/"), "Link incorrect."))
    }
}

#[get("/logout")]
fn logout(mut cookies: Cookies) -> Redirect {
    cookies.remove_private(Cookie::named("session_token"));
    Redirect::to("/")
}

pub fn endpoints() -> Vec<Route> {
    routes![
        login_page,
        login_page_loggedin,
        login,
        register_user,
        signup_page_loggedin,
        signup_page,
        verify_email,
        logout
    ]
}
