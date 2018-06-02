use club_coding::models::{Users, UsersAndSessions, UsersGroup};
use rocket::request::{self, FromRequest, Request};
use rocket::Outcome;
use database::{DbConn, MySqlPool};
use rocket::State;
use diesel::prelude::*;

/// Gets all of the users from the database.
/// Ordered by their creation date in an
/// ascending order.
pub fn get_users(connection: &DbConn) -> Vec<Users> {
    use club_coding::schema::users::dsl::*;

    match users.order(created.asc()).load::<Users>(&**connection) {
        Ok(vec_of_users) => vec_of_users,
        Err(_) => vec![],
    }
}

/// The User Struct is used for all endpoints that requires
/// that the user is logged in.
#[derive(Serialize)]
pub struct User {
    /// The id of the user.
    pub id: i64,
    /// The username of the user.
    pub username: String,
    /// The email of the user.
    pub email: String,
    /// A boolean representing if the user
    /// is an administrator or not.
    pub admin: bool,
}

/// Request guard making sure that the user is logged in.
impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<User, ()> {
        let pool = request.guard::<State<MySqlPool>>()?;

        let username = request
            .cookies()
            .get_private("session_token")
            .map(|cookie| match pool.get() {
                Ok(connection) => {
                    use club_coding::schema::{users, users_sessions};

                    match users_sessions::table
                        .inner_join(users::table.on(users::id.eq(users_sessions::user_id)))
                        .filter(users_sessions::token.eq(cookie.value()))
                        .filter(users::verified.eq(true))
                        .select((users::id, users::username, users::email))
                        .first::<UsersAndSessions>(&*connection)
                    {
                        Ok(results) => {
                            use club_coding::schema::users_group::dsl::*;

                            let is_admin = match users_group
                                .filter(user_id.eq(results.id))
                                .filter(group_id.eq(1))
                                .first::<UsersGroup>(&*connection)
                            {
                                Ok(_) => true,
                                Err(_) => false,
                            };

                            Some(User {
                                id: results.id,
                                username: results.username,
                                email: results.email,
                                admin: is_admin,
                            })
                        }
                        Err(_) => None,
                    }
                }
                Err(_) => None,
            });
        match username {
            Some(uid) => match uid {
                Some(user) => {
                    return Outcome::Success(user);
                }
                None => return Outcome::Forward(()),
            },
            None => return Outcome::Forward(()),
        }
    }
}
