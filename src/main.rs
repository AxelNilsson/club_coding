#![feature(proc_macro_hygiene, decl_macro)]
extern crate bcrypt;
extern crate chrono;
extern crate club_coding;
extern crate csrf;
extern crate data_encoding;
extern crate diesel;
extern crate futures;
extern crate hyper_tls;
extern crate r2d2;
extern crate r2d2_redis;
extern crate rand;
extern crate redis;
extern crate regex;
extern crate rocket_contrib;
extern crate serde;
extern crate serde_json;
extern crate stripe;
extern crate time;
extern crate tokio_core;
extern crate hyper;
extern crate reqwest;

#[macro_use] extern crate rocket;

#[macro_use] extern crate tera;

#[macro_use] extern crate serde_derive;

mod admin;
mod authentication;
mod charge;
mod custom_csrf;
mod database;
mod email;
mod errors;
mod pages;
mod payment;
mod request_network;
mod series;
mod settings;
mod structs;
mod users;
mod videos;

pub fn website() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", authentication::endpoints())
        .mount("/", settings::endpoints())
        .mount("/", pages::endpoints())
        .mount("/", videos::endpoints())
        .mount("/", charge::endpoints())
        .mount("/settings/payment", payment::endpoints())
        .mount("/series", series::endpoints())
        .mount("/admin", admin::endpoints())
        .attach(rocket_contrib::templates::Template::fairing())
        .attach(custom_csrf::csrf_secret_key_fairing())
        .attach(database::mysql_fairing())
        .attach(database::redis_fairing())
        .attach(structs::stripe_token_fairing())
        .attach(structs::postmark_token_fairing())
        .attach(structs::email_regex_fairing())
        .register(errors::endpoints())
}

fn main() {
    website().launch();
}
