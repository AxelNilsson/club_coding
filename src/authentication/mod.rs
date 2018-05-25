use bcrypt::{hash, verify, DEFAULT_COST};
use rocket::request::{FlashMessage, Form};
use rocket_contrib::Template;
use rocket::response::{Flash, Redirect};
use club_coding::{create_new_user, create_new_user_session, create_new_users_recover_email,
                  create_new_users_verify_email, establish_connection};
use rocket::http::{Cookie, Cookies};
use time::Duration;
use rocket::Route;
use users::User as UserStruct;
use {rand, std};
use email::{EmailBody, PostmarkClient};
use club_coding::models::{Users, UsersRecoverEmail, UsersVerifyEmail};
use custom_csrf::{csrf_matches, CsrfCookie, CsrfToken};
use diesel::prelude::*;

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
        .filter(verified.eq(true))
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

fn get_user_id_from_email(name: String) -> Option<i64> {
    use club_coding::schema::users::dsl::*;

    let connection = establish_connection();
    let results = users
        .filter(email.eq(name))
        .filter(verified.eq(true))
        .limit(1)
        .load::<Users>(&connection)
        .expect("Error loading users");

    if results.len() == 1 {
        Some(results[0].id)
    } else {
        None
    }
}

#[get("/login")]
fn login_page_loggedin(_user: UserStruct) -> Redirect {
    Redirect::to("/")
}

#[derive(Serialize)]
struct LoginContext {
    header: String,
    csrf: String,
    flash_name: String,
    flash_msg: String,
}

#[get("/login", rank = 2)]
fn login_page(token: CsrfToken, flash: Option<FlashMessage>) -> Template {
    let (name, msg) = match flash {
        Some(flash) => (flash.name().to_string(), flash.msg().to_string()),
        None => ("".to_string(), "".to_string()),
    };
    let context = LoginContext {
        header: "Login Page!".to_string(),
        csrf: token.value(),
        flash_name: name,
        flash_msg: msg,
    };
    Template::render("login", &context)
}

#[post("/login", data = "<user>")]
fn login(
    csrf_cookie: CsrfCookie,
    mut cookies: Cookies,
    user: Form<User>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let input_data: User = user.into_inner();
    if csrf_matches(input_data.csrf, csrf_cookie.value()) {
        match get_password_hash_from_username(input_data.username.clone()) {
            Ok(password_hash) => match verify(&input_data.password, &password_hash) {
                Ok(passwords_match) => {
                    if passwords_match {
                        let session_token = generate_token(64);
                        let connection = establish_connection();
                        match get_user_id_from_username(input_data.username) {
                            Ok(user_id) => {
                                create_new_user_session(
                                    &connection,
                                    user_id,
                                    session_token.clone(),
                                );
                                let mut c = Cookie::new("session_token", session_token);
                                c.set_max_age(Duration::hours(24));
                                cookies.add_private(c);
                                Ok(Flash::success(Redirect::to("/"), "You're now logged in."))
                            }
                            Err(_) => {
                                Err(Flash::error(Redirect::to("/login"), "User not verified"))
                            }
                        }
                    } else {
                        Err(Flash::error(Redirect::to("/login"), "Password incorrect"))
                    }
                }
                Err(_) => Err(Flash::error(Redirect::to("/login"), "An error occurred")),
            },
            Err(_) => Err(Flash::error(Redirect::to("/login"), "No user found")),
        }
    } else {
        Err(Flash::error(Redirect::to("/login"), "CSRF Failed."))
    }
}

#[derive(FromForm)]
struct UserRegistration {
    username: String,
    password: String,
    confirm_password: String,
    email: String,
    csrf: String,
}

pub fn send_verify_email(connection: &MysqlConnection, user_id: i64, email: String) {
    let token = generate_token(30);
    create_new_users_verify_email(connection, user_id, token.clone());
    let body = EmailBody {
        from: "axel@clubcoding.com".to_string(),
        to: email,
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
}

#[post("/signup", data = "<user>")]
fn register_user(
    csrf_cookie: CsrfCookie,
    user: Form<UserRegistration>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let input: UserRegistration = user.into_inner();
    if input.password == input.confirm_password {
        if csrf_matches(input.csrf, csrf_cookie.value()) {
            match hash(&input.password, DEFAULT_COST) {
                Ok(hashed_password) => {
                    let connection = establish_connection();
                    let new_user = create_new_user(
                        &connection,
                        input.username.clone(),
                        hashed_password,
                        input.email.clone(),
                    );
                    send_verify_email(&connection, new_user.id, input.email);
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
    } else {
        Err(Flash::error(
            Redirect::to("/signup"),
            "Passwords don't match.",
        ))
    }
}

#[get("/signup")]
fn signup_page_loggedin(_userid: UserStruct) -> Redirect {
    Redirect::to("/")
}

#[get("/signup", rank = 2)]
fn signup_page(token: CsrfToken, flash: Option<FlashMessage>) -> Template {
    let (name, msg) = match flash {
        Some(flash) => (flash.name().to_string(), flash.msg().to_string()),
        None => ("".to_string(), "".to_string()),
    };
    let context = LoginContext {
        header: "Login Page!".to_string(),
        csrf: token.value(),
        flash_name: name,
        flash_msg: msg,
    };
    Template::render("signup", &context)
}

#[get("/email/verify/<_uuid>")]
fn verify_email_loggedin(_uuid: String, _userid: UserStruct) -> Redirect {
    Redirect::to("/")
}

#[get("/email/verify/<uuid>", rank = 2)]
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

#[get("/recover/email")]
fn send_recover_email_loggedin_page(_userid: UserStruct) -> Redirect {
    Redirect::to("/")
}

#[get("/recover/email", rank = 2)]
fn send_recover_email_page(csrf_token: CsrfToken, flash: Option<FlashMessage>) -> Template {
    let (name, msg) = match flash {
        Some(flash) => (flash.name().to_string(), flash.msg().to_string()),
        None => ("".to_string(), "".to_string()),
    };
    let context = LoginContext {
        header: "recover_email".to_string(),
        csrf: csrf_token.value(),
        flash_name: name,
        flash_msg: msg,
    };
    Template::render("send_recover", &context)
}

fn send_recover_mail(token: String, email: String) {
    let body = EmailBody {
        from: "axel@clubcoding.com".to_string(),
        to: email,
        subject: Some("Recover your account".to_string()),
        html_body: Some(format!("<html><body><a href='https://clubcoding.com/email/recover/{}'>Please press this link to recover your account.</a></body></html>", token)),
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
}

#[post("/recover/email")]
fn send_recover_email_loggedin(_userid: UserStruct) -> Redirect {
    Redirect::to("/")
}

#[derive(FromForm)]
struct RecoverAccount {
    email: String,
    csrf: String,
}

#[post("/recover/email", data = "<user>", rank = 2)]
fn send_recover_email(
    csrf_cookie: CsrfCookie,
    user: Form<RecoverAccount>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let input: RecoverAccount = user.into_inner();
    if csrf_matches(input.csrf, csrf_cookie.value()) {
        let connection = establish_connection();
        match get_user_id_from_email(input.email.clone()) {
            Some(user_id) => {
                let token = generate_token(30);
                send_recover_mail(token.clone(), input.email);
                create_new_users_recover_email(&connection, user_id, token);
                Ok(Flash::success(
                    Redirect::to("/"),
                    "Email sent. Please check your inbox.",
                ))
            }
            None => Err(Flash::error(
                Redirect::to("/recover/email"),
                "Email not found.",
            )),
        }
    } else {
        Err(Flash::error(
            Redirect::to("/recover/email"),
            "CSRF Doesn't match.",
        ))
    }
}

#[get("/email/recover/<_uuid>")]
fn recover_email_loggedin_page(_uuid: String, _userid: UserStruct) -> Redirect {
    Redirect::to("/")
}

#[get("/email/recover/<uuid>", rank = 2)]
fn recover_email_page(
    csrf_token: CsrfToken,
    flash: Option<FlashMessage>,
    uuid: String,
) -> Result<Template, Flash<Redirect>> {
    use club_coding::schema::users_recover_email::dsl::*;

    let connection = establish_connection();
    let results = users_recover_email
        .filter(token.eq(uuid))
        .limit(1)
        .load::<UsersRecoverEmail>(&connection)
        .expect("Error loading users verify email");

    if results.len() == 1 {
        if results[0].used {
            Err(Flash::error(Redirect::to("/"), "Link already used."))
        } else {
            let (name, msg) = match flash {
                Some(flash) => (flash.name().to_string(), flash.msg().to_string()),
                None => ("".to_string(), "".to_string()),
            };
            let context = LoginContext {
                header: "recover_email".to_string(),
                csrf: csrf_token.value(),
                flash_name: name,
                flash_msg: msg,
            };
            Ok(Template::render("recover_email", &context))
        }
    } else {
        Err(Flash::error(Redirect::to("/"), "Link incorrect."))
    }
}

#[post("/email/recover/<_uuid>")]
fn recover_email_loggedin(_uuid: String, _userid: UserStruct) -> Redirect {
    Redirect::to("/")
}

#[derive(FromForm)]
struct UpdatePassword {
    password: String,
    confirm_password: String,
    csrf: String,
}

#[post("/email/recover/<uuid>", data = "<user>", rank = 2)]
fn recover_email(
    uuid: String,
    csrf_cookie: CsrfCookie,
    user: Form<UpdatePassword>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let input: UpdatePassword = user.into_inner();
    if input.password == input.confirm_password {
        use club_coding::schema::users_recover_email::dsl::*;

        let connection = establish_connection();
        let results = users_recover_email
            .filter(token.eq(uuid.clone()))
            .limit(1)
            .load::<UsersRecoverEmail>(&connection)
            .expect("Error loading users recover email");

        if results.len() == 1 {
            if results[0].used {
                Err(Flash::error(Redirect::to("/"), "Link already used."))
            } else {
                if csrf_matches(input.csrf, csrf_cookie.value()) {
                    match hash(&input.password, DEFAULT_COST) {
                        Ok(hashed_password) => {
                            diesel::update(users_recover_email.find(results[0].id))
                                .set(used.eq(true))
                                .execute(&connection)
                                .unwrap();

                            use club_coding::schema::users::dsl::*;
                            diesel::update(users.find(results[0].user_id))
                                .set(password.eq(hashed_password))
                                .execute(&connection)
                                .unwrap();

                            Ok(Flash::success(
                                Redirect::to("/"),
                                "Password updated, please sign in.",
                            ))
                        }
                        Err(_) => Err(Flash::error(
                            Redirect::to(&format!("/email/recover/{}", uuid)),
                            "An error occured, please try again later.",
                        )),
                    }
                } else {
                    Err(Flash::error(
                        Redirect::to(&format!("/email/recover/{}", uuid)),
                        "CSRF Doesn't match.",
                    ))
                }
            }
        } else {
            Err(Flash::error(
                Redirect::to(&format!("/email/recover/{}", uuid)),
                "Link incorrect.",
            ))
        }
    } else {
        Err(Flash::error(
            Redirect::to(&format!("/email/recover/{}", uuid)),
            "Passwords not matching.",
        ))
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
        verify_email_loggedin,
        verify_email,
        recover_email_loggedin_page,
        recover_email_loggedin,
        recover_email_page,
        recover_email,
        send_recover_email_loggedin_page,
        send_recover_email_loggedin,
        send_recover_email_page,
        send_recover_email,
        logout
    ]
}
