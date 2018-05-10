use rocket::Route;
use rocket::request::Form;
use rocket_contrib::Template;
use rocket::response::{Flash, Redirect};
use club_coding::{establish_connection, insert_new_card, insert_new_subscription,
                  insert_new_users_stripe_customer, insert_new_users_stripe_token};
use club_coding::models::{Users, UsersSessions};
use rocket::http::Cookies;
use users::User;
use member::Member;
use structs::LoggedInContext;
use stripe;
use std;
use diesel::prelude::*;

fn get_user_id_from_session_token(session_token: String) -> Result<i64, std::io::Error> {
    use club_coding::schema::users_sessions::dsl::*;

    let connection = establish_connection();

    let results = users_sessions
        .filter(token.eq(session_token))
        .limit(1)
        .load::<UsersSessions>(&connection)
        .expect("Error loading sessions");

    if results.len() == 1 {
        return Ok(results[0].user_id);
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "no token found",
        ));
    }
}

pub fn get_user_id_from_cookies(cookies: &mut Cookies) -> Result<i64, std::io::Error> {
    match cookies.get_private("session_token") {
        Some(cookie) => match get_user_id_from_session_token(cookie.value().to_string()) {
            Ok(user_id) => Ok(user_id),
            Err(error) => Err(error),
        },
        _ => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "no token found",
            ))
        }
    }
}

fn get_user_from_cookie(cookies: &mut Cookies) -> Result<Users, std::io::Error> {
    let user_id = get_user_id_from_cookies(cookies)?;

    use club_coding::schema::users::dsl::*;

    let connection = establish_connection();
    let results = users
        .filter(id.eq(user_id))
        .limit(1)
        .load::<Users>(&connection)
        .expect("Error loading sessions");

    if results.len() == 1 {
        return Ok(results[0].clone());
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "no token found",
        ));
    }
}

#[get("/settings/subscription")]
fn member_page(user: User, _user: Member) -> Template {
    let context = LoggedInContext {
        header: "Club Coding".to_string(),
        username: user.username,
    };
    Template::render("subscription", &context)
}

#[get("/settings/subscription", rank = 2)]
fn user_page(user: User) -> Template {
    let context = LoggedInContext {
        header: "Club Coding".to_string(),
        username: user.username,
    };
    Template::render("subscription", &context)
}

#[get("/settings/subscription", rank = 3)]
fn nouser_page() -> Redirect {
    Redirect::to("/")
}

#[derive(Debug, FromForm)]
struct Stripe {
    card_address_city: Option<String>,
    card_address_country: Option<String>,
    card_address_line1: Option<String>,
    card_address_line1_check: Option<String>,
    card_address_line2: Option<String>,
    card_address_state: Option<String>,
    card_address_zip: Option<String>,
    card_address_zip_check: Option<String>,
    card_brand: String,
    card_country: String,
    card_cvc_check: Option<String>,
    card_dynamic_last4: Option<String>,
    card_exp_month: i32,
    card_exp_year: i32,
    card_funding: Option<String>,
    card_id: Option<String>,
    card_last4: String,
    card_metadata: Option<String>,
    card_name: Option<String>,
    card_object: Option<String>,
    card_tokenization_method: Option<String>,
    client_ip: String,
    created: i64,
    id: String,
    livemode: bool,
    object: Option<String>,
    #[form(field = "type")]
    type_of_payment: Option<String>,
    used: bool,
}

fn create_customer(client: &stripe::Client, email: &str, token: &str) -> stripe::Customer {
    // Create the customer
    stripe::Customer::create(
        &client,
        stripe::CustomerParams {
            email: Some(email),
            source: Some(stripe::CustomerSource::Token(token)),
            account_balance: None,
            business_vat_id: None,
            coupon: None,
            description: None,
            metadata: None,
            shipping: None,
        },
    ).unwrap()
}

fn create_subscription(
    client: &stripe::Client,
    plan: &str,
    customer_id: &str,
) -> stripe::Subscription {
    stripe::Subscription::create(
        &client,
        stripe::SubscriptionParams {
            application_fee_percent: None,
            coupon: None,
            metadata: None,
            plan: None,
            prorate: None,
            proration_date: None,
            quantity: None,
            source: None,
            tax_percent: None,
            trial_end: None,
            trial_period_days: None,
            customer: Some(&customer_id),
            items: Some(vec![
                (stripe::ItemParams {
                    plan: plan,
                    quantity: None,
                }),
            ]),
        },
    ).unwrap()
}

fn charge(data: &Stripe, plan: &str, username: &str, user_id: i64) -> Result<(), std::io::Error> {
    let client = stripe::Client::new("sk_test_cztFtKdeTEnlPLL6DpvkbjFf");
    let connection = establish_connection();
    insert_new_card(
        &connection,
        user_id,
        data.card_address_city.clone(),
        data.card_address_country.clone(),
        data.card_address_line1.clone(),
        data.card_address_line1_check.clone(),
        data.card_address_line2.clone(),
        data.card_address_state.clone(),
        data.card_address_zip.clone(),
        data.card_address_zip_check.clone(),
        data.card_brand.clone(),
        data.card_country.clone(),
        data.card_cvc_check.clone(),
        data.card_dynamic_last4.clone(),
        data.card_exp_month,
        data.card_exp_year,
        data.card_funding.clone(),
        data.card_id.clone(),
        data.card_last4.clone(),
        data.card_metadata.clone(),
        data.card_name.clone(),
        data.card_object.clone(),
        data.card_tokenization_method.clone(),
    );
    insert_new_users_stripe_token(
        &connection,
        user_id,
        data.client_ip.clone(),
        data.created.clone(),
        data.id.clone(),
        data.livemode,
        data.object.clone(),
        data.type_of_payment.clone(),
        data.used,
    );
    let customer = create_customer(&client, username, &(data.id.clone()));
    insert_new_users_stripe_customer(
        &connection,
        user_id,
        customer.id.clone(),
        customer.account_balance,
        customer.business_vat_id,
        customer.created as i64,
        customer.default_source,
        customer.delinquent,
        customer.desc,
        customer.email,
        customer.livemode,
    );
    let subscription = create_subscription(&client, plan, &customer.id);
    insert_new_subscription(
        &connection,
        user_id,
        subscription.id,
        subscription.application_fee_percent,
        subscription.cancel_at_period_end,
        subscription.canceled_at,
        subscription.created,
        subscription.current_period_start,
        subscription.current_period_end,
        subscription.customer,
        subscription.ended_at,
        subscription.livemode,
        subscription.quantity as i64,
        subscription.start,
        subscription.status,
        subscription.tax_percent,
        subscription.trial_start,
        subscription.trial_end,
    );
    Ok(())
}

#[post("/charge/monthly", data = "<form_data>")]
fn charge_monthly(
    mut cookies: Cookies,
    form_data: Form<Stripe>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match get_user_from_cookie(&mut cookies) {
        Ok(user) => {
            let data = form_data.into_inner();
            let monthly_plan = "plan_ChdX4TFEThzwWe";
            match charge(&data, monthly_plan, &user.username, user.id) {
                Ok(()) => Ok(Flash::success(
                    Redirect::to("/"),
                    "Monthly subscription activated. Welcome to the club!",
                )),
                _ => Err(Flash::error(
                    Redirect::to("/settings/subscription"),
                    "An error occured, please try again later.",
                )),
            }
        }
        Err(_not_logged_in) => Err(Flash::error(
            Redirect::to("/settings/subscription"),
            "An error occured, please try again later.",
        )),
    }
}

#[post("/charge/quarterly", data = "<form_data>")]
fn charge_quarterly(
    mut cookies: Cookies,
    form_data: Form<Stripe>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match get_user_from_cookie(&mut cookies) {
        Ok(user) => {
            let data = form_data.into_inner();
            let quarterly_plan = "plan_ChdYJPgVLiHbaz";
            match charge(&data, quarterly_plan, &user.username, user.id) {
                Ok(()) => Ok(Flash::success(
                    Redirect::to("/"),
                    "Quarterly subscription activated. Welcome to the club!",
                )),
                _ => Err(Flash::error(
                    Redirect::to("/settings/subscription"),
                    "An error occured, please try again later.",
                )),
            }
        }
        Err(_not_logged_in) => Err(Flash::error(
            Redirect::to("/settings/subscription"),
            "An error occured, please try again later.",
        )),
    }
}

#[post("/charge/yearly", data = "<form_data>")]
fn charge_yearly(
    mut cookies: Cookies,
    form_data: Form<Stripe>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match get_user_from_cookie(&mut cookies) {
        Ok(user) => {
            let data = form_data.into_inner();
            let yearly_plan = "plan_ChdZSLPkhnIroM";
            match charge(&data, yearly_plan, &user.username, user.id) {
                Ok(()) => Ok(Flash::success(
                    Redirect::to("/"),
                    "Yearly subscription activated. Welcome to the club!",
                )),
                _ => Err(Flash::error(
                    Redirect::to("/settings/subscription"),
                    "An error occured, please try again later.",
                )),
            }
        }
        Err(_not_logged_in) => Err(Flash::error(
            Redirect::to("/settings/subscription"),
            "An error occured, please try again later.",
        )),
    }
}

pub fn endpoints() -> Vec<Route> {
    routes![
        member_page,
        user_page,
        nouser_page,
        charge_monthly,
        charge_quarterly,
        charge_yearly
    ]
}
