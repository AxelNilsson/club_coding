use database::DbConn;
use club_coding::models::{Users, UsersVerifyEmail};
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

/// Gets the UsersVerifyEmail struct
/// from the verification token
pub fn get_verify_email_by_token(
    connection: &DbConn,
    uuid_token: &str,
) -> Option<UsersVerifyEmail> {
    use club_coding::schema::users_verify_email::dsl::*;

    match users_verify_email
        .filter(token.eq(uuid_token))
        .first::<UsersVerifyEmail>(&**connection)
    {
        Ok(result) => Some(result),
        Err(_) => None,
    }
}

/// Invalidates verification token
/// and sets the user to verified.
pub fn invalidate_token_and_verify_user(
    connection: &DbConn,
    verification_id: i64,
    user_id: i64,
) -> Result<(), Error> {
    use club_coding::schema::users_verify_email;

    match diesel::update(users_verify_email::table.find(verification_id))
        .set(users_verify_email::used.eq(true))
        .execute(&**connection)
    {
        Ok(_) => {}
        Err(_) => {
            return Err(Error::new(
                ErrorKind::Other,
                "Could not update verification token.",
            ))
        }
    }

    use club_coding::schema::users;

    match diesel::update(users::table.find(user_id))
        .set(users::verified.eq(true))
        .execute(&**connection)
    {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::new(ErrorKind::Other, "Could not verify user.")),
    }
}
