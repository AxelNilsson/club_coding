extern crate chrono;
#[macro_use]
extern crate diesel;
extern crate rocket;

pub mod models;
pub mod schema;

use diesel::prelude::*;
use std::io::{Error, ErrorKind};

use self::models::{NewGroup, NewSerie, NewUser, NewUserGroup, NewUserRecoverEmail,
                   NewUserSeriesAccess, NewUserSession, NewUserStripeCard, NewUserStripeCharge,
                   NewUserStripeCustomer, NewUserStripeToken, NewUserVerifyEmail, NewUserView,
                   NewVideo, Users};

pub fn create_new_group(conn: &MysqlConnection, uuid: &str, name: &str) -> Result<(), Error> {
    use schema::groups;

    let new_group = NewGroup {
        uuid: uuid,
        name: name,
    };

    match diesel::insert_into(groups::table)
        .values(&new_group)
        .execute(conn)
    {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::new(ErrorKind::Other, "No group table found")),
    }
}

pub fn create_new_series(
    conn: &MysqlConnection,
    uuid: &str,
    title: &str,
    slug: &str,
    description: &str,
    price: i32,
    published: bool,
    archived: bool,
) -> Result<(), Error> {
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

    match diesel::insert_into(series::table)
        .values(&new_video)
        .execute(conn)
    {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::new(ErrorKind::Other, "No series table found")),
    }
}

pub fn create_new_user_session(
    conn: &MysqlConnection,
    user_id: i64,
    token: &str,
) -> Result<(), Error> {
    use schema::users_sessions;

    let new_user_session = NewUserSession {
        user_id: user_id,
        token: token,
    };

    match diesel::insert_into(users_sessions::table)
        .values(&new_user_session)
        .execute(conn)
    {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::new(
            ErrorKind::Other,
            "No users sessions table found",
        )),
    }
}

pub fn create_new_user(
    conn: &MysqlConnection,
    username: &str,
    password: &str,
    email: &str,
) -> Result<Users, Error> {
    use schema::users;

    let new_user = NewUser {
        username: username,
        password: password,
        email: email,
    };

    match diesel::insert_into(users::table)
        .values(&new_user)
        .execute(conn)
    {
        Ok(_) => match users::table.order(users::id.desc()).first(conn) {
            Ok(user) => Ok(user),
            Err(_) => Err(Error::new(ErrorKind::Other, "No user found")),
        },
        Err(_) => Err(Error::new(ErrorKind::Other, "No user found")),
    }
}

pub fn create_new_user_group(
    conn: &MysqlConnection,
    user_id: i64,
    group_id: i64,
) -> Result<(), Error> {
    use schema::users_group;

    let new_user_group = NewUserGroup {
        user_id: user_id,
        group_id: group_id,
    };

    match diesel::insert_into(users_group::table)
        .values(&new_user_group)
        .execute(conn)
    {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::new(ErrorKind::Other, "No users group table found")),
    }
}

pub fn create_new_user_series_access(
    conn: &MysqlConnection,
    user_id: i64,
    series_id: i64,
    bought: bool,
) -> Result<(), Error> {
    use schema::users_series_access;

    let new_user_series_access = NewUserSeriesAccess {
        user_id: user_id,
        series_id: series_id,
        bought: bought,
    };

    match diesel::insert_into(users_series_access::table)
        .values(&new_user_series_access)
        .execute(conn)
    {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::new(
            ErrorKind::Other,
            "No users series access table found",
        )),
    }
}

pub fn create_new_users_recover_email(
    conn: &MysqlConnection,
    user_id: i64,
    token: &str,
) -> Result<(), Error> {
    use schema::users_recover_email;

    let new_user_recover_email = NewUserRecoverEmail {
        user_id: user_id,
        token: token,
    };

    match diesel::insert_into(users_recover_email::table)
        .values(&new_user_recover_email)
        .execute(conn)
    {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::new(
            ErrorKind::Other,
            "No users recover email table found",
        )),
    }
}

pub fn insert_new_card(
    conn: &MysqlConnection,
    user_id: i64,
    address_city: Option<&str>,
    address_country: Option<&str>,
    address_line1: Option<&str>,
    address_line1_check: Option<&str>,
    address_line2: Option<&str>,
    address_state: Option<&str>,
    address_zip: Option<&str>,
    address_zip_check: Option<&str>,
    brand: &str,
    country: &str,
    cvc_check: Option<&str>,
    dynamic_last4: Option<&str>,
    exp_month: i32,
    exp_year: i32,
    funding: Option<&str>,
    card_id: Option<&str>,
    last4: &str,
    metadata: Option<&str>,
    name: Option<&str>,
    object: Option<&str>,
    tokenization_method: Option<&str>,
) -> Result<(), Error> {
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

    match diesel::insert_into(users_stripe_card::table)
        .values(&new_card)
        .execute(conn)
    {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::new(
            ErrorKind::Other,
            "No users stripe card table found",
        )),
    }
}

pub fn insert_new_users_stripe_charge(
    conn: &MysqlConnection,
    user_id: i64,
    series_id: i64,
    uuid: &str,
    amount: i32,
    amount_refunded: i32,
    balance_transaction: Option<&str>,
    captured: bool,
    created_at_stripe: i64,
    description: Option<&str>,
    destination: Option<&str>,
    dispute: Option<&str>,
    failure_code: Option<&str>,
    failure_message: Option<&str>,
    livemode: bool,
    on_behalf_of: Option<&str>,
    order: Option<&str>,
    paid: bool,
    refunded: bool,
    source_id: &str,
    source_transfer: Option<&str>,
    statement_descriptor: Option<&str>,
    status: &str,
) -> Result<(), Error> {
    use schema::users_stripe_charge;

    let new_charge = NewUserStripeCharge {
        user_id: user_id,
        series_id: series_id,
        uuid: uuid,
        amount: amount,
        amount_refunded: amount_refunded,
        balance_transaction: balance_transaction,
        captured: captured,
        created_at_stripe: created_at_stripe,
        description: description,
        destination: destination,
        dispute: dispute,
        failure_code: failure_code,
        failure_message: failure_message,
        livemode: livemode,
        on_behalf_of: on_behalf_of,
        order: order,
        paid: paid,
        refunded: refunded,
        source_id: source_id,
        source_transfer: source_transfer,
        statement_descriptor: statement_descriptor,
        status: status,
    };

    match diesel::insert_into(users_stripe_charge::table)
        .values(&new_charge)
        .execute(conn)
    {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::new(
            ErrorKind::Other,
            "No users stripe charge table found",
        )),
    }
}

pub fn insert_new_users_stripe_token(
    conn: &MysqlConnection,
    user_id: i64,
    client_ip: &str,
    created_at_stripe: i64,
    token_id: &str,
    livemode: bool,
    object: Option<&str>,
    type_: Option<&str>,
    used: bool,
) -> Result<(), Error> {
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

    match diesel::insert_into(users_stripe_token::table)
        .values(&new_stripe)
        .execute(conn)
    {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::new(
            ErrorKind::Other,
            "No users stripe token table found",
        )),
    }
}

pub fn insert_new_users_stripe_customer(
    conn: &MysqlConnection,
    user_id: i64,
    uuid: &String,
    account_balance: i64,
    business_vat_id: Option<&str>,
    created_at_stripe: i64,
    default_source: Option<&str>,
    delinquent: bool,
    desc: Option<&str>,
    email: Option<&str>,
    livemode: bool,
) -> Result<(), Error> {
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

    match diesel::insert_into(users_stripe_customer::table)
        .values(&new_stripe)
        .execute(conn)
    {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::new(
            ErrorKind::Other,
            "No users stripe customer table found",
        )),
    }
}

pub fn create_new_users_verify_email(
    conn: &MysqlConnection,
    user_id: i64,
    token: &str,
) -> Result<(), Error> {
    use schema::users_verify_email;

    let new_user_verify_email = NewUserVerifyEmail {
        user_id: user_id,
        token: token,
    };

    match diesel::insert_into(users_verify_email::table)
        .values(&new_user_verify_email)
        .execute(conn)
    {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::new(
            ErrorKind::Other,
            "No users verify email table found",
        )),
    }
}

pub fn create_new_user_view(
    conn: &MysqlConnection,
    user_id: i64,
    video_id: i64,
) -> Result<(), Error> {
    use schema::users_views;

    let new_user_view = NewUserView {
        user_id: user_id,
        video_id: video_id,
    };

    match diesel::insert_into(users_views::table)
        .values(&new_user_view)
        .execute(conn)
    {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::new(ErrorKind::Other, "No users view table found")),
    }
}

pub fn create_new_video(
    conn: &MysqlConnection,
    uuid: &str,
    title: &str,
    slug: &str,
    description: &str,
    published: bool,
    membership_only: bool,
    series: Option<i64>,
    episode_number: Option<i32>,
    archived: bool,
    vimeo_id: &str,
) -> Result<(), Error> {
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

    match diesel::insert_into(videos::table)
        .values(&new_video)
        .execute(conn)
    {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::new(ErrorKind::Other, "No videos table found")),
    }
}
