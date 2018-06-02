use database::DbConn;
use club_coding::models::{Series, UsersStripeCard, UsersStripeCharge, UsersStripeCustomer};
use chrono::NaiveDateTime;
use payment::Charge;
use diesel::prelude::*;

/// Returns vector of charges belonging to
/// user specified by user id.
pub fn get_charges(connection: &DbConn, uid: i64) -> Vec<Charge> {
    use club_coding::schema::users_stripe_charge::dsl::*;

    match users_stripe_charge
        .filter(user_id.eq(uid))
        .order(id.desc())
        .load::<UsersStripeCharge>(&**connection)
    {
        Ok(charges) => {
            let mut to_return: Vec<Charge> = vec![];
            for charge in charges {
                use club_coding::schema::series::dsl::*;

                let serie: Option<Series> =
                    match series.filter(id.eq(charge.series_id)).first(&**connection) {
                        Ok(serie) => Some(serie),
                        Err(_) => None,
                    };

                match serie {
                    Some(serie) => {
                        to_return.push(Charge {
                            amount: charge.amount,
                            date: NaiveDateTime::from_timestamp(charge.created_at_stripe, 0)
                                .to_string(),
                            series: serie.title,
                        });
                    }
                    None => {}
                }
            }
            to_return
        }
        Err(_) => vec![],
    }
}

/// Checks if the Stripe Customer Exists and
/// returns either Some Stripe Customer or
/// None.
pub fn get_customer(connection: &DbConn, uid: i64) -> Option<UsersStripeCustomer> {
    use club_coding::schema::users_stripe_customer::dsl::*;

    match users_stripe_customer
        .filter(user_id.eq(uid))
        .first::<UsersStripeCustomer>(&**connection)
    {
        Ok(user) => Some(user),
        Err(_) => None,
    }
}

/// Gets the first card belonging
/// to a user and deletes it.
/// Returns the card_id for the card.
pub fn delete_and_get_card(connection: &DbConn, uid: i64) -> Option<String> {
    use club_coding::schema::users_stripe_card::dsl::*;

    let card: Option<UsersStripeCard> = match users_stripe_card
        .filter(user_id.eq(uid))
        .first(&**connection)
    {
        Ok(card) => Some(card),
        Err(_) => None,
    };

    match card {
        Some(card) => {
            match diesel::delete(users_stripe_card.filter(user_id.eq(uid))).execute(&**connection) {
                Ok(_) => card.card_id,
                Err(_) => None,
            }
        }
        None => None,
    }
}
