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
extern crate rand;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
extern crate stripe;
extern crate time;

#[macro_use]
extern crate serde_derive;

mod member;
mod authentication;
mod subscription;
mod settings;
mod pages;
mod search;
mod videos;
mod users;
mod structs;
mod admin;
mod series;

fn main() {
    rocket::ignite()
        .mount("/", authentication::endpoints())
        .mount("/", subscription::endpoints())
        .mount("/", settings::endpoints())
        .mount("/", pages::endpoints())
        .mount("/", search::endpoints())
        .mount("/", videos::endpoints())
        .mount("/series", series::endpoints())
        .mount("/admin", admin::endpoints())
        .attach(rocket_contrib::Template::fairing())
        .launch();
}
