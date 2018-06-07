#![feature(plugin)]
#![plugin(rocket_codegen)]
#![feature(custom_derive)]
#![feature(extern_prelude)]
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
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
extern crate serde_json;
extern crate stripe;
extern crate time;
extern crate tokio_core;

#[macro_use]
extern crate tera;

#[macro_use]
extern crate hyper;

#[macro_use]
extern crate serde_derive;

mod authentication;
mod settings;
mod pages;
mod videos;
mod users;
mod structs;
mod admin;
mod series;
mod email;
mod custom_csrf;
mod charge;
mod payment;
mod database;
mod errors;
mod request_network;

#[cfg(test)]
mod tests;

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
        .attach(rocket_contrib::Template::fairing())
        .attach(database::mysql_fairing())
        .attach(database::redis_fairing())
        .attach(structs::stripe_token_fairing())
        .attach(structs::postmark_token_fairing())
        .attach(structs::email_regex_fairing())
        .catch(errors::endpoints())
}

fn main() {
    website().launch();
}
