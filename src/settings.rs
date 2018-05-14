use rocket::Route;

use bcrypt::{hash, verify, DEFAULT_COST};
use rocket_contrib::{Json, Template};
use rocket::response::Redirect;
use club_coding::establish_connection;
use club_coding::models::Users;

use structs::LoggedInContext;
use users::User;
use diesel;
use diesel::prelude::*;
use std;

#[get("/settings/password")]
fn password_page(user: User) -> Template {
    let context = LoggedInContext {
        header: "Club Coding".to_string(),
        user: user,
    };
    Template::render("password", &context)
}

#[get("/settings/password", rank = 2)]
fn password_page_nouser() -> Redirect {
    Redirect::to("/")
}

#[derive(Deserialize)]
struct UpdatePasswordStruct {
    old_password: String,
    new_password: String,
    confirm_new_password: String,
}

#[derive(Serialize)]
struct Message {
    text: String,
}

fn get_password_hash_from_userid(user_id: i64) -> Result<String, std::io::Error> {
    use club_coding::schema::users::dsl::*;

    let connection = establish_connection();
    let results = users
        .filter(id.eq(user_id))
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

#[post("/settings/password", data = "<json_data>")]
fn update_password(user: User, json_data: Json<UpdatePasswordStruct>) -> Json<Message> {
    if json_data.new_password == json_data.confirm_new_password {
        match get_password_hash_from_userid(user.id) {
            Ok(password_hash) => match verify(&json_data.old_password, &password_hash) {
                Ok(passwords_match) => {
                    if passwords_match {
                        match hash(&json_data.new_password, DEFAULT_COST) {
                            Ok(hashed_password) => {
                                use club_coding::schema::users::dsl::*;

                                let connection = establish_connection();
                                diesel::update(users.filter(id.eq(user.id)))
                                    .set(password.eq(hashed_password))
                                    .execute(&connection)
                                    .unwrap();

                                Json(Message {
                                    text: "updated".to_string(),
                                })
                            }
                            Err(_) => Json(Message {
                                text: "An error occured".to_string(),
                            }),
                        }
                    } else {
                        Json(Message {
                            text: "old password incorrect".to_string(),
                        })
                    }
                }
                Err(_) => Json(Message {
                    text: "An error occured".to_string(),
                }),
            },
            Err(_) => Json(Message {
                text: "no password found in database".to_string(),
            }),
        }
    } else {
        Json(Message {
            text: "passwords not matching".to_string(),
        })
    }
}

pub fn endpoints() -> Vec<Route> {
    routes![password_page, password_page_nouser, update_password]
}
