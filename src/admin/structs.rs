use club_coding::models::{UsersAndSessions, UsersGroup};
use rocket::request::{self, FromRequest, Request};
use rocket::Outcome;
use database::{MySqlPool, RedisPool};
use rocket::State;
use redis::Commands;
use diesel::prelude::*;

/// Context for rendering tera templates
/// for administrator endpoints.
#[derive(Serialize)]
pub struct LoggedInContext<'a> {
    /// Header used in tera templates.
    /// Mainly used for the title.
    pub header: &'a str,
    /// The administrator struct used by templates.
    /// For example the username for the toolbar.
    pub user: Administrator,
}

/// The Administrator Struct is used for all endpoints that requires
/// that the administrator is logged in and an administrator.
/// The difference between the user struct and the administrator
/// struct is that the administrator struct requires the user
/// is an administrator aswell, otherwise it's going to
/// forward the request.
#[derive(Serialize, Deserialize)]
pub struct Administrator {
    /// The id of the administrator.
    pub id: i64,
    /// The username of the administrator.
    pub username: String,
    /// The email of the administrator.
    pub email: String,
    /// A boolean representing if the user
    /// is an administrator or not.
    pub admin: bool,
}

/// Request guard making sure that the user is logged in
/// and an administrator.
impl<'a, 'r> FromRequest<'a, 'r> for Administrator {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Administrator, ()> {
        let mysql_pool = request.guard::<State<MySqlPool>>()?;
        let redis_pool = request.guard::<State<RedisPool>>()?;

        let username = request.cookies().get_private("session_token").map(
            |cookie| match redis_pool.get() {
                Ok(redis_conn) => match redis_conn.get::<&str, String>(cookie.value()) {
                    Ok(result) => {
                        let user: Administrator = match serde_json::from_str(&result) {
                            Ok(user) => user,
                            Err(_) => return None,
                        };
                        if user.admin {
                            return Some(user);
                        }
                        return None;
                    }
                    Err(_) => match mysql_pool.get() {
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

                                    match users_group
                                        .filter(user_id.eq(results.id))
                                        .filter(group_id.eq(1))
                                        .first::<UsersGroup>(&*connection)
                                    {
                                        Ok(_) => {
                                            let user = Administrator {
                                                id: results.id,
                                                username: results.username,
                                                email: results.email,
                                                admin: true,
                                            };

                                            let json_string = match serde_json::to_string(&user) {
                                                Ok(json_string) => json_string,
                                                Err(_) => return Some(user),
                                            };

                                            match redis_conn.set_ex::<&str, String, String>(
                                                cookie.value(),
                                                json_string,
                                                86400,
                                            ) {
                                                Ok(_) => {}
                                                Err(_) => {}
                                            }

                                            return Some(user);
                                        }
                                        Err(_) => None,
                                    }
                                }
                                Err(_) => None,
                            }
                        }
                        Err(_) => None,
                    },
                },
                Err(_) => None,
            },
        );
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
