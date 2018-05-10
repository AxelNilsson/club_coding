use club_coding::establish_connection;
use club_coding::models::{UsersSessions, UsersStripeSubscriptions};
use rocket::request::{self, FromRequest, Request};
use rocket::Outcome;

use diesel::prelude::*;

pub struct Member(i64);

impl<'a, 'r> FromRequest<'a, 'r> for Member {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Member, ()> {
        let uid = request
            .cookies()
            .get_private("session_token")
            .map(|cookie| {
                use club_coding::schema::users_sessions::dsl::*;

                let connection = establish_connection();

                let results = users_sessions
                    .filter(token.eq(cookie.value().to_string()))
                    .limit(1)
                    .load::<UsersSessions>(&connection)
                    .expect("Error loading sessions");

                if results.len() == 1 {
                    use club_coding::schema::users_stripe_subscriptions::dsl::*;

                    let connection = establish_connection();
                    let results = users_stripe_subscriptions
                        .filter(user_id.eq(results[0].user_id))
                        .limit(1)
                        .load::<UsersStripeSubscriptions>(&connection)
                        .expect("Error loading users");

                    if results.len() == 1 {
                        return results[0].user_id;
                    } else {
                        return 0;
                    }
                } else {
                    return 0;
                }
            });
        match uid {
            Some(uid) => {
                if uid > 0 {
                    return Outcome::Success(Member(uid));
                } else {
                    return Outcome::Forward(());
                }
            }
            None => return Outcome::Forward(()),
        }
    }
}
