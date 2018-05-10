use bcrypt::{hash, verify, DEFAULT_COST};
use rocket::request::Form;
use rocket_contrib::Template;
use rocket::response::Redirect;
use club_coding::{create_new_user, create_new_user_session, establish_connection};
use rocket::http::{Cookie, Cookies};
use time::Duration;
use csrf::{AesGcmCsrfProtection, CsrfProtection};
use data_encoding::BASE64;
use rocket::Route;
use users::User as UserStruct;
use rand;

use club_coding::models::Users;
use std;
use diesel::prelude::*;

#[derive(FromForm)]
struct User {
    username: String,
    password: String,
    csrf: String,
}

fn generate_session_token(length: u8) -> Result<String, std::io::Error> {
    let bytes: Vec<u8> = (0..length).map(|_| rand::random::<u8>()).collect();
    let strings: Vec<String> = bytes.iter().map(|byte| format!("{:02X}", byte)).collect();
    return Ok(strings.join(""));
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
fn login_page(mut cookies: Cookies) -> Template {
    let protect = AesGcmCsrfProtection::from_key(*b"01234567012345670123456701234567");
    let (token, cookie) = protect
        .generate_token_pair(None, 300)
        .expect("couldn't generate token/cookie pair");

    let mut c = Cookie::new("csrf", cookie.b64_string());
    c.set_max_age(Duration::hours(24));
    cookies.add_private(c);

    let context = CSRFContext {
        header: "Login Page!".to_string(),
        csrf: token.b64_string(),
    };

    Template::render("login", &context)
}

#[post("/login", data = "<user>")]
fn login(mut cookies: Cookies, user: Form<User>) -> Result<Redirect, String> {
    match cookies.get_private("csrf") {
        Some(cookie) => {
            let input_data: User = user.into_inner();
            let protect = AesGcmCsrfProtection::from_key(*b"01234567012345670123456701234567");
            let token_bytes = BASE64
                .decode(input_data.csrf.as_bytes())
                .expect("token not base64");
            let cookie_bytes = BASE64
                .decode(cookie.value().to_string().as_bytes())
                .expect("cookie not base64");

            let parsed_token = protect.parse_token(&token_bytes).expect("token not parsed");
            let parsed_cookie = protect
                .parse_cookie(&cookie_bytes)
                .expect("cookie not parsed");

            if protect.verify_token_pair(&parsed_token, &parsed_cookie) {
                match get_password_hash_from_username(input_data.username.clone()) {
                    Ok(password_hash) => match verify(&input_data.password, &password_hash) {
                        Ok(passwords_match) => {
                            if passwords_match {
                                match generate_session_token(64) {
                                    Ok(session_token) => {
                                        let connection = establish_connection();
                                        let user_id =
                                            get_user_id_from_username(input_data.username).unwrap();
                                        create_new_user_session(
                                            &connection,
                                            user_id,
                                            session_token.clone(),
                                        );
                                        let mut c = Cookie::new("session_token", session_token);
                                        c.set_max_age(Duration::hours(24));
                                        cookies.add_private(c);
                                        Ok(Redirect::to("/"))
                                    }
                                    Err(_) => Err(String::from("Login failed")),
                                }
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
        _ => Err(String::from("registration failed")),
    }
}

#[post("/signup", data = "<user>")]
fn register_user(mut cookies: Cookies, user: Form<User>) -> String {
    match cookies.get_private("csrf") {
        Some(cookie) => {
            let input: User = user.into_inner();
            let protect = AesGcmCsrfProtection::from_key(*b"01234567012345670123456701234567");
            let token_bytes = BASE64
                .decode(input.csrf.as_bytes())
                .expect("token not base64");
            let cookie_bytes = BASE64
                .decode(cookie.value().to_string().as_bytes())
                .expect("cookie not base64");

            let parsed_token = protect.parse_token(&token_bytes).expect("token not parsed");
            let parsed_cookie = protect
                .parse_cookie(&cookie_bytes)
                .expect("cookie not parsed");

            if protect.verify_token_pair(&parsed_token, &parsed_cookie) {
                match hash(&input.password, DEFAULT_COST) {
                    Ok(hashed_password) => {
                        let connection = establish_connection();
                        create_new_user(&connection, input.username, hashed_password);
                        return String::from("User registered");
                    }
                    Err(_) => return String::from("registration failed"),
                }
            } else {
                String::from("csrf failed")
            }
        }
        _ => String::from("registration failed"),
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
fn signup_page(mut cookies: Cookies) -> Template {
    let protect = AesGcmCsrfProtection::from_key(*b"01234567012345670123456701234567");
    let (token, cookie) = protect
        .generate_token_pair(None, 300)
        .expect("couldn't generate token/cookie pair");

    let mut c = Cookie::new("csrf", cookie.b64_string());
    c.set_max_age(Duration::hours(24));
    cookies.add_private(c);

    let context = CSRFContext {
        header: "Sign up!".to_string(),
        csrf: token.b64_string(),
    };

    Template::render("signup", &context)
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
        logout
    ]
}
