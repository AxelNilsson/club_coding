use club_coding::establish_connection;
use club_coding::models::{UsersSessions, UsersStripeSubscriptions};
use rocket::request::{self, FromRequest, Request};
use rocket::Outcome;
use chrono::{DateTime, NaiveDateTime};
use std::cmp::Ordering;
use chrono::prelude::*;

use diesel::prelude::*;

#[derive(Serialize)]
pub struct Member {
    current_period_start: String,
    current_period_end: String,
    active: bool,
}

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
                        let end = NaiveDateTime::from_timestamp(results[0].current_period_end, 0);
                        let end_dt = DateTime::<Utc>::from_utc(end, Utc);
                        let utc: DateTime<Utc> = Utc::now();
                        if end_dt.cmp(&utc) == Ordering::Greater {
                            return Some(Member {
                                current_period_start: NaiveDateTime::from_timestamp(
                                    results[0].current_period_start,
                                    0,
                                ).to_string(),
                                current_period_end: end.to_string(),
                                active: !results[0].cancel_at_period_end,
                            });
                        } else {
                            return None;
                        }
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            });
        match uid {
            Some(uid) => match uid {
                Some(uid) => return Outcome::Success(uid),
                None => return Outcome::Forward(()),
            },
            None => return Outcome::Forward(()),
        }
    }
}
