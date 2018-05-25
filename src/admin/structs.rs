use club_coding::establish_connection;
use club_coding::models::{Users, UsersGroup, UsersSessions};
use rocket::request::{self, FromRequest, Request};
use rocket::Outcome;

use diesel::prelude::*;

#[derive(Serialize)]
pub struct LoggedInContext {
    pub header: String,
    pub user: Administrator,
}

#[derive(Serialize)]
pub struct Administrator {
    pub id: i64,
    pub username: String,
    pub admin: bool,
}

impl<'a, 'r> FromRequest<'a, 'r> for Administrator {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Administrator, ()> {
        let username = request
            .cookies()
            .get_private("session_token")
            .map(|cookie| {
                use club_coding::schema::users_sessions::dsl::*;

                let connection = establish_connection();

                match users_sessions
                    .filter(token.eq(cookie.value().to_string()))
                    .limit(1)
                    .load::<UsersSessions>(&connection)
                {
                    Ok(results) => {
                        if results.len() == 1 {
                            use club_coding::schema::users::dsl::*;

                            let connection = establish_connection();
                            match users
                                .filter(id.eq(results[0].user_id))
                                .limit(1)
                                .load::<Users>(&connection)
                            {
                                Ok(results) => {
                                    if results.len() == 1 {
                                        use club_coding::schema::users_group::dsl::*;

                                        let connection = establish_connection();
                                        match users_group
                                            .filter(user_id.eq(results[0].id))
                                            .filter(group_id.eq(1))
                                            .limit(1)
                                            .load::<UsersGroup>(&connection)
                                        {
                                            Ok(admin) => {
                                                if admin.len() == 1 {
                                                    return Some(Administrator {
                                                        id: results[0].id,
                                                        username: results[0].username.clone(),
                                                        admin: true,
                                                    });
                                                } else {
                                                    return None;
                                                }
                                            }
                                            Err(_) => None,
                                        }
                                    } else {
                                        return None;
                                    }
                                }
                                Err(_) => None,
                            }
                        } else {
                            return None;
                        }
                    }
                    Err(_) => None,
                }
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
