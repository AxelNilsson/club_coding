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
pub struct NewGroup<'a> {
    pub uuid: &'a str,
    pub name: &'a str,
}

#[derive(Queryable, Clone)]
pub struct NewsletterSubscribers {
    pub id: i64,
    pub email: String,
    pub active: bool,
    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "newsletter_subscribers"]
pub struct NewNewsletterSubscriber<'a> {
    pub email: &'a str,
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
pub struct NewSerie<'a> {
    pub uuid: &'a str,
    pub title: &'a str,
    pub slug: &'a str,
    pub description: &'a str,
    pub price: i32,
    pub published: bool,
    pub archived: bool,
}

#[derive(Queryable, Clone)]
pub struct Users {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub email: String,
    pub verified: bool,
    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub password: &'a str,
    pub email: &'a str,
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

#[derive(Queryable)]
pub struct UsersRecoverEmail {
    pub id: i64,
    pub user_id: i64,
    pub token: String,
    pub used: bool,
    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "users_recover_email"]
pub struct NewUserRecoverEmail<'a> {
    pub user_id: i64,
    pub token: &'a str,
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
pub struct NewUserStripeCard<'a> {
    pub user_id: i64,
    pub address_city: Option<&'a str>,
    pub address_country: Option<&'a str>,
    pub address_line1: Option<&'a str>,
    pub address_line1_check: Option<&'a str>,
    pub address_line2: Option<&'a str>,
    pub address_state: Option<&'a str>,
    pub address_zip: Option<&'a str>,
    pub address_zip_check: Option<&'a str>,
    pub brand: &'a str,
    pub country: &'a str,
    pub cvc_check: Option<&'a str>,
    pub dynamic_last4: Option<&'a str>,
    pub exp_month: i32,
    pub exp_year: i32,
    pub funding: Option<&'a str>,
    pub card_id: Option<&'a str>,
    pub last4: &'a str,
    pub metadata: Option<&'a str>,
    pub name: Option<&'a str>,
    pub object: Option<&'a str>,
    pub tokenization_method: Option<&'a str>,
}

#[derive(Queryable)]
pub struct UsersStripeCharge {
    pub id: i64,
    pub user_id: i64,
    pub series_id: i64,
    pub uuid: String,
    pub amount: i32,
    pub amount_refunded: i32,
    pub balance_transaction: Option<String>,
    pub captured: bool,
    pub created_at_stripe: i64,
    pub description: Option<String>,
    pub destination: Option<String>,
    pub dispute: Option<String>,
    pub failure_code: Option<String>,
    pub failure_message: Option<String>,
    pub livemode: bool,
    pub on_behalf_of: Option<String>,
    pub order: Option<String>,
    pub paid: bool,
    pub refunded: bool,
    pub source_id: String,
    pub source_transfer: Option<String>,
    pub statement_descriptor: Option<String>,
    pub status: String,
}

#[derive(Insertable)]
#[table_name = "users_stripe_charge"]
pub struct NewUserStripeCharge<'a> {
    pub user_id: i64,
    pub series_id: i64,
    pub uuid: &'a str,
    pub amount: i32,
    pub amount_refunded: i32,
    pub balance_transaction: Option<&'a str>,
    pub captured: bool,
    pub created_at_stripe: i64,
    pub description: Option<&'a str>,
    pub destination: Option<&'a str>,
    pub dispute: Option<&'a str>,
    pub failure_code: Option<&'a str>,
    pub failure_message: Option<&'a str>,
    pub livemode: bool,
    pub on_behalf_of: Option<&'a str>,
    pub order: Option<&'a str>,
    pub paid: bool,
    pub refunded: bool,
    pub source_id: &'a str,
    pub source_transfer: Option<&'a str>,
    pub statement_descriptor: Option<&'a str>,
    pub status: &'a str,
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
    pub uuid: &'a str,
    pub account_balance: i64,
    pub business_vat_id: Option<&'a str>,
    pub created_at_stripe: i64,
    pub default_source: Option<&'a str>,
    pub delinquent: bool,
    pub desc: Option<&'a str>,
    pub email: Option<&'a str>,
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
pub struct NewUserStripeToken<'a> {
    pub user_id: i64,
    pub client_ip: &'a str,
    pub created_at_stripe: i64,
    pub token_id: &'a str,
    pub livemode: bool,
    pub object: Option<&'a str>,
    pub type_: Option<&'a str>,
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

#[derive(Queryable)]
pub struct UsersAndSessions {
    pub id: i64,
    pub username: String,
    pub email: String,
}

#[derive(Insertable)]
#[table_name = "users_sessions"]
pub struct NewUserSession<'a> {
    pub user_id: i64,
    pub token: &'a str,
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
pub struct NewUserVerifyEmail<'a> {
    pub user_id: i64,
    pub token: &'a str,
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
    pub serie_id: i64,
    pub episode_number: i32,
    pub archived: bool,
    pub vimeo_id: String,
}

#[derive(Insertable)]
#[table_name = "videos"]
pub struct NewVideo<'a> {
    pub uuid: &'a str,
    pub title: &'a str,
    pub slug: &'a str,
    pub description: &'a str,
    pub published: bool,
    pub membership_only: bool,
    pub serie_id: i64,
    pub episode_number: i32,
    pub archived: bool,
    pub vimeo_id: &'a str,
}

#[derive(Queryable)]
pub struct VideoJoin {
    pub id: i64,
    pub uuid: String,
    pub title: String,
    pub description: String,
    pub serie_id: i64,
    pub vimeo_id: String,
    pub membership_only: bool,
    pub series_title: String,
    pub price: i32,
}
