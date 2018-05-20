extern crate chrono;
#[macro_use]
extern crate diesel;
extern crate dotenv;

pub mod models;
pub mod schema;

use diesel::prelude::*;
use dotenv::dotenv;
use std::env;

use self::models::{NewGroup, NewSerie, NewUser, NewUserGroup, NewUserSeriesAccess, NewUserSession,
                   NewUserStripeCard, NewUserStripeCustomer, NewUserStripeToken,
                   NewUserVerifyEmail, NewUserView, NewVideo, Users};

pub fn establish_connection() -> MysqlConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set!");
    MysqlConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub fn create_new_group(conn: &MysqlConnection, uuid: String, name: String) {
    use schema::groups;

    let new_group = NewGroup {
        uuid: uuid,
        name: name,
    };

    diesel::insert_into(groups::table)
        .values(&new_group)
        .execute(conn)
        .expect("Error saving new group");
}

pub fn create_new_series(
    conn: &MysqlConnection,
    uuid: String,
    title: String,
    slug: String,
    description: String,
    price: i32,
    published: bool,
    archived: bool,
) {
    use schema::series;

    let new_video = NewSerie {
        uuid: uuid,
        title: title,
        slug: slug,
        description: description,
        price: price,
        published: published,
        archived: archived,
    };

    diesel::insert_into(series::table)
        .values(&new_video)
        .execute(conn)
        .expect("Error saving new user");
}

pub fn create_new_user_session(conn: &MysqlConnection, user_id: i64, token: String) {
    use schema::users_sessions;

    let new_user_session = NewUserSession {
        user_id: user_id,
        token: token,
    };

    diesel::insert_into(users_sessions::table)
        .values(&new_user_session)
        .execute(conn)
        .expect("Error saving new session");
}

pub fn create_new_user(conn: &MysqlConnection, username: String, password: String) -> Users {
    use schema::users;

    let new_user = NewUser {
        username: username,
        password: password,
    };

    diesel::insert_into(users::table)
        .values(&new_user)
        .execute(conn)
        .expect("Error saving new user");

    users::table.order(users::id.desc()).first(conn).unwrap()
}

pub fn create_new_user_group(conn: &MysqlConnection, user_id: i64, group_id: i64) {
    use schema::users_group;

    let new_user_group = NewUserGroup {
        user_id: user_id,
        group_id: group_id,
    };

    diesel::insert_into(users_group::table)
        .values(&new_user_group)
        .execute(conn)
        .expect("Error saving new user group");
}

pub fn create_new_user_series_access(
    conn: &MysqlConnection,
    user_id: i64,
    series_id: i64,
    bought: bool,
) {
    use schema::users_series_access;

    let new_user_series_access = NewUserSeriesAccess {
        user_id: user_id,
        series_id: series_id,
        bought: bought,
    };

    diesel::insert_into(users_series_access::table)
        .values(&new_user_series_access)
        .execute(conn)
        .expect("Error saving new user series access");
}

pub fn insert_new_card(
    conn: &MysqlConnection,
    user_id: i64,
    address_city: Option<String>,
    address_country: Option<String>,
    address_line1: Option<String>,
    address_line1_check: Option<String>,
    address_line2: Option<String>,
    address_state: Option<String>,
    address_zip: Option<String>,
    address_zip_check: Option<String>,
    brand: String,
    country: String,
    cvc_check: Option<String>,
    dynamic_last4: Option<String>,
    exp_month: i32,
    exp_year: i32,
    funding: Option<String>,
    card_id: Option<String>,
    last4: String,
    metadata: Option<String>,
    name: Option<String>,
    object: Option<String>,
    tokenization_method: Option<String>,
) {
    use schema::users_stripe_card;

    let new_card = NewUserStripeCard {
        user_id: user_id,
        address_city: address_city,
        address_country: address_country,
        address_line1: address_line1,
        address_line1_check: address_line1_check,
        address_line2: address_line2,
        address_state: address_state,
        address_zip: address_zip,
        address_zip_check: address_zip_check,
        brand: brand,
        country: country,
        cvc_check: cvc_check,
        dynamic_last4: dynamic_last4,
        exp_month: exp_month,
        exp_year: exp_year,
        funding: funding,
        card_id: card_id,
        last4: last4,
        metadata: metadata,
        name: name,
        object: object,
        tokenization_method: tokenization_method,
    };

    diesel::insert_into(users_stripe_card::table)
        .values(&new_card)
        .execute(conn)
        .expect("Error saving new card");
}

pub fn insert_new_users_stripe_token(
    conn: &MysqlConnection,
    user_id: i64,
    client_ip: String,
    created_at_stripe: i64,
    token_id: String,
    livemode: bool,
    object: Option<String>,
    type_: Option<String>,
    used: bool,
) {
    use schema::users_stripe_token;

    let new_stripe = NewUserStripeToken {
        user_id: user_id,
        client_ip: client_ip,
        created_at_stripe: created_at_stripe,
        token_id: token_id,
        livemode: livemode,
        object: object,
        type_: type_,
        used: used,
    };

    diesel::insert_into(users_stripe_token::table)
        .values(&new_stripe)
        .execute(conn)
        .expect("Error saving new stripe");
}

pub fn insert_new_users_stripe_customer(
    conn: &MysqlConnection,
    user_id: i64,
    uuid: &String,
    account_balance: i64,
    business_vat_id: Option<String>,
    created_at_stripe: i64,
    default_source: Option<String>,
    delinquent: bool,
    desc: Option<String>,
    email: Option<String>,
    livemode: bool,
) {
    use schema::users_stripe_customer;

    let new_stripe = NewUserStripeCustomer {
        user_id: user_id,
        uuid: uuid,
        account_balance: account_balance,
        business_vat_id: business_vat_id,
        created_at_stripe: created_at_stripe,
        default_source: default_source,
        delinquent: delinquent,
        desc: desc,
        email: email,
        livemode: livemode,
    };

    diesel::insert_into(users_stripe_customer::table)
        .values(&new_stripe)
        .execute(conn)
        .expect("Error saving new stripe");
}

pub fn create_new_users_verify_email(conn: &MysqlConnection, user_id: i64, token: String) {
    use schema::users_verify_email;

    let new_user_verify_email = NewUserVerifyEmail {
        user_id: user_id,
        token: token,
    };

    diesel::insert_into(users_verify_email::table)
        .values(&new_user_verify_email)
        .execute(conn)
        .expect("Error saving new user verify email");
}

pub fn create_new_user_view(conn: &MysqlConnection, user_id: i64, video_id: i64) {
    use schema::users_views;

    let new_user_view = NewUserView {
        user_id: user_id,
        video_id: video_id,
    };

    diesel::insert_into(users_views::table)
        .values(&new_user_view)
        .execute(conn)
        .expect("Error saving new user view");
}

pub fn create_new_video(
    conn: &MysqlConnection,
    uuid: String,
    title: String,
    slug: String,
    description: String,
    published: bool,
    membership_only: bool,
    series: Option<i64>,
    episode_number: Option<i32>,
    archived: bool,
    vimeo_id: String,
) {
    use schema::videos;

    let new_video = NewVideo {
        uuid: uuid,
        title: title,
        slug: slug,
        description: description,
        published: published,
        membership_only: membership_only,
        series: series,
        episode_number: episode_number,
        archived: archived,
        vimeo_id: vimeo_id,
    };

    diesel::insert_into(videos::table)
        .values(&new_video)
        .execute(conn)
        .expect("Error saving new user");
}
