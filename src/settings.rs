use rocket::Route;
use bcrypt::{hash, verify, DEFAULT_COST};
use rocket_contrib::{Json, Template};
use rocket::response::Redirect;
use club_coding::models::Users;
use database::DbConn;
use users::User;
use std::io::{Error, ErrorKind};
use diesel;
use diesel::prelude::*;

/// Context for rendering tera templates
/// for logged in endpoints.
#[derive(Serialize)]
pub struct PasswordContext<'a> {
    /// Header used in tera templates.
    /// Mainly used for the title.
    pub header: &'a str,
    /// The user struct used by templates.
    /// For example the username for the toolbar.
    pub user: User,
}

/// GET Endpoint for the page to change your
/// password. Endpoints checks if the
/// user is logged in by using the
/// user request guard. If the user
/// is not logged in it forwards
/// the request.
/// Responds with the Password Template
/// in the settings folder.
#[get("/settings/password")]
fn password_page(user: User) -> Template {
    let context = PasswordContext {
        header: "Update Password",
        user: user,
    };
    Template::render("settings/password", &context)
}

/// GET Endpoint for the page to change your
/// password. This endpoint will kick in
/// if the user is not logged in and will
/// redirect the user to the index.
#[get("/settings/password", rank = 2)]
fn password_page_nouser() -> Redirect {
    Redirect::to("/")
}

/// Struct for updating the
/// password for a user.
/// Will fail if not all three
/// are supplied.
#[derive(Deserialize)]
struct UpdatePasswordStruct {
    /// The old (current) password of the user.
    old_password: String,
    /// The new password for the user.
    new_password: String,
    /// Confirming the password by entering it twice.
    confirm_new_password: String,
}

/// Struct for responding with a JSON
/// message.
#[derive(Serialize)]
struct Message<'a> {
    /// The message that will be
    /// forwarded to the user.
    text: &'a str,
}

/// The function gets the password hash
/// stored in the database for the user
/// by using the userid.
/// Responds with either the password hash
/// or an error.
fn get_password_hash_from_userid(connection: &DbConn, user_id: i64) -> Result<String, Error> {
    use club_coding::schema::users::dsl::*;

    match users.find(user_id).first::<Users>(&**connection) {
        Ok(result) => Ok(result.password),
        Err(_) => return Err(Error::new(ErrorKind::Other, "No user found")),
    }
}

/// Hashes the password sent in
/// and updates the user specified
/// by the user_id with the new
/// password.
fn hash_and_update_password(
    connection: &DbConn,
    user_id: i64,
    new_password: &str,
) -> Result<(), Error> {
    match hash(new_password, DEFAULT_COST) {
        Ok(hashed_password) => {
            use club_coding::schema::users::dsl::*;

            match diesel::update(users.filter(id.eq(user_id)))
                .set(password.eq(hashed_password))
                .execute(&**connection)
            {
                Ok(_) => Ok(()),
                Err(_) => Err(Error::new(
                    ErrorKind::Other,
                    "Could not update database with new password.",
                )),
            }
        }
        Err(_) => Err(Error::new(ErrorKind::Other, "Could not hash password.")),
    }
}

/// POST Endpoint for the page to change your
/// password. Endpoints checks if the
/// user is logged in by using the
/// user request guard. If the user
/// is not logged in it forwards
/// the request.
/// It requires all of the parameters in the
/// UpdatePasswordStruct in a JSON format.
/// If everything is successful, it will update
/// the users password and return the Message
/// struct in a JSON format.
#[post("/settings/password", data = "<json_data>")]
fn update_password<'a>(
    conn: DbConn,
    user: User,
    json_data: Json<UpdatePasswordStruct>,
) -> Json<Message<'a>> {
    if !(json_data.new_password == json_data.confirm_new_password) {
        return Json(Message {
            text: "The new passwords are not matching.",
        });
    }

    let password_hash: String = match get_password_hash_from_userid(&conn, user.id) {
        Ok(password_hash) => password_hash,
        Err(_) => {
            return Json(Message {
                text: "No password found in database for the user.",
            })
        }
    };
    let passwords_match: bool = match verify(&json_data.old_password, &password_hash) {
        Ok(passwords_match) => passwords_match,
        Err(_) => {
            return Json(Message {
                text: "An unknown error occured. Please try again later.",
            })
        }
    };
    if !passwords_match {
        return Json(Message {
            text: "The old password is incorrect.",
        });
    } else {
        match hash_and_update_password(&conn, user.id, &json_data.new_password) {
            Ok(_) => Json(Message {
                text: "Your password has been updated",
            }),
            Err(_) => Json(Message {
                text: "An unknown error occured. Please try again later.",
            }),
        }
    }
}

/// Assembles all of the endpoints.
/// The upside of assembling all of the endpoints here
/// is that we don't have to update the main function but
/// instead we can keep all of the changes in here.
pub fn endpoints() -> Vec<Route> {
    routes![password_page, password_page_nouser, update_password]
}
