extern crate chrono;

use schema::*;
use self::chrono::NaiveDateTime;

#[derive(Queryable, Clone)]
pub struct Groups {
    pub id: i64,
    pub uuid: String,
    pub name: String,
    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "groups"]
pub struct NewGroup {
    pub uuid: String,
    pub name: String,
}

#[derive(Queryable, Clone)]
pub struct Series {
    pub id: i64,
    pub uuid: String,
    pub title: String,
    pub slug: String,
    pub description: String,
    pub price: i32,
    pub published: bool,
    pub archived: bool,
    pub in_development: bool,
    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "series"]
pub struct NewSerie {
    pub uuid: String,
    pub title: String,
    pub slug: String,
    pub description: String,
    pub price: i32,
    pub published: bool,
    pub archived: bool,
}

#[derive(Queryable, Clone)]
pub struct Users {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub verified: bool,
    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub username: String,
    pub password: String,
}

#[derive(Queryable, Clone)]
pub struct UsersGroup {
    pub id: i64,
    pub user_id: i64,
    pub group_id: i64,
    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "users_group"]
pub struct NewUserGroup {
    pub user_id: i64,
    pub group_id: i64,
}

#[derive(Queryable, Clone)]
pub struct UsersSeriesAccess {
    pub id: i64,
    pub user_id: i64,
    pub series_id: i64,
    pub bought: bool,
    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "users_series_access"]
pub struct NewUserSeriesAccess {
    pub user_id: i64,
    pub series_id: i64,
    pub bought: bool,
}

#[derive(Queryable)]
pub struct UsersStripeCard {
    pub id: i64,
    pub user_id: i64,
    pub address_city: Option<String>,
    pub address_country: Option<String>,
    pub address_line1: Option<String>,
    pub address_line1_check: Option<String>,
    pub address_line2: Option<String>,
    pub address_state: Option<String>,
    pub address_zip: Option<String>,
    pub address_zip_check: Option<String>,
    pub brand: String,
    pub country: String,
    pub cvc_check: Option<String>,
    pub dynamic_last4: Option<String>,
    pub exp_month: i32,
    pub exp_year: i32,
    pub funding: Option<String>,
    pub card_id: Option<String>,
    pub last4: String,
    pub metadata: Option<String>,
    pub name: Option<String>,
    pub object: Option<String>,
    pub tokenization_method: Option<String>,
}

#[derive(Insertable)]
#[table_name = "users_stripe_card"]
pub struct NewUserStripeCard {
    pub user_id: i64,
    pub address_city: Option<String>,
    pub address_country: Option<String>,
    pub address_line1: Option<String>,
    pub address_line1_check: Option<String>,
    pub address_line2: Option<String>,
    pub address_state: Option<String>,
    pub address_zip: Option<String>,
    pub address_zip_check: Option<String>,
    pub brand: String,
    pub country: String,
    pub cvc_check: Option<String>,
    pub dynamic_last4: Option<String>,
    pub exp_month: i32,
    pub exp_year: i32,
    pub funding: Option<String>,
    pub card_id: Option<String>,
    pub last4: String,
    pub metadata: Option<String>,
    pub name: Option<String>,
    pub object: Option<String>,
    pub tokenization_method: Option<String>,
}

#[derive(Queryable, Clone)]
pub struct UsersStripeCustomer {
    pub id: i64,
    pub user_id: i64,
    pub uuid: String,
    pub account_balance: i64,
    pub business_vat_id: Option<String>,
    pub created_at_stripe: i64,
    pub default_source: Option<String>,
    pub delinquent: bool,
    pub desc: Option<String>,
    pub email: Option<String>,
    pub livemode: bool,
    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "users_stripe_customer"]
pub struct NewUserStripeCustomer<'a> {
    pub user_id: i64,
    pub uuid: &'a String,
    pub account_balance: i64,
    pub business_vat_id: Option<String>,
    pub created_at_stripe: i64,
    pub default_source: Option<String>,
    pub delinquent: bool,
    pub desc: Option<String>,
    pub email: Option<String>,
    pub livemode: bool,
}

#[derive(Queryable)]
pub struct UsersStripeToken {
    pub id: i64,
    pub user_id: i64,
    pub client_ip: String,
    pub created_at_stripe: i64,
    pub token_id: String,
    pub livemode: bool,
    pub object: Option<String>,
    pub type_: Option<String>,
    pub used: bool,
    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "users_stripe_token"]
pub struct NewUserStripeToken {
    pub user_id: i64,
    pub client_ip: String,
    pub created_at_stripe: i64,
    pub token_id: String,
    pub livemode: bool,
    pub object: Option<String>,
    pub type_: Option<String>,
    pub used: bool,
}

#[derive(Queryable)]
pub struct UsersSessions {
    pub id: i64,
    pub user_id: i64,
    pub token: String,
    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "users_sessions"]
pub struct NewUserSession {
    pub user_id: i64,
    pub token: String,
}

#[derive(Queryable)]
pub struct UsersVerifyEmail {
    pub id: i64,
    pub user_id: i64,
    pub token: String,
    pub used: bool,
    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "users_verify_email"]
pub struct NewUserVerifyEmail {
    pub user_id: i64,
    pub token: String,
}

#[derive(Queryable)]
pub struct UsersViews {
    pub id: i64,
    pub user_id: i64,
    pub video_id: i64,
    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "users_views"]
pub struct NewUserView {
    pub user_id: i64,
    pub video_id: i64,
}

#[derive(Queryable, Clone)]
pub struct Videos {
    pub id: i64,
    pub uuid: String,
    pub title: String,
    pub slug: String,
    pub description: String,
    pub published: bool,
    pub membership_only: bool,
    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,
    pub series: Option<i64>,
    pub episode_number: Option<i32>,
    pub archived: bool,
    pub vimeo_id: String,
}

#[derive(Insertable)]
#[table_name = "videos"]
pub struct NewVideo {
    pub uuid: String,
    pub title: String,
    pub slug: String,
    pub description: String,
    pub published: bool,
    pub membership_only: bool,
    pub series: Option<i64>,
    pub episode_number: Option<i32>,
    pub archived: bool,
    pub vimeo_id: String,
}
