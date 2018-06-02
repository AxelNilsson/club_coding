use database::DbConn;
use club_coding::models::Users;
use std::io::{Error, ErrorKind};
use diesel::prelude::*;

/// Gets the password hash of a user from
/// the username of the user.
pub fn get_password_hash_from_username(connection: &DbConn, name: &str) -> Result<String, Error> {
    use club_coding::schema::users::dsl::*;

    match users
        .filter(username.eq(name))
        .first::<Users>(&**connection)
    {
        Ok(results) => Ok(results.password),
        Err(_) => Err(Error::new(ErrorKind::Other, "No user found")),
    }
}

/// Gets the user id of a user from
/// the username of the user.
pub fn get_user_id_from_username(connection: &DbConn, name: &str) -> Result<i64, Error> {
    use club_coding::schema::users::dsl::*;

    match users
        .filter(username.eq(name))
        .filter(verified.eq(true))
        .first::<Users>(&**connection)
    {
        Ok(result) => Ok(result.id),
        Err(_) => Err(Error::new(ErrorKind::Other, "No user found")),
    }
}

/// Gets the user id of a user from
/// the email of the user.
pub fn get_user_id_from_email(connection: &DbConn, name: &str) -> Option<i64> {
    use club_coding::schema::users::dsl::*;

    match users
        .filter(email.eq(name))
        .filter(verified.eq(true))
        .first::<Users>(&**connection)
    {
        Ok(result) => Some(result.id),
        Err(_) => None,
    }
}
