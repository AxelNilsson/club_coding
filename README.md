# Club Coding

Club Coding is a website for learning programming, mainly with a focus on Rust and built in Rust.

## Getting Started

These instructions will get you a copy of the project up and running on your local machine for development and testing purposes. See deployment for notes on how to deploy the project on a live system.

### Prerequisites

What things you need to install the software and how to install them

```
Diesel Client
MySQL Client
MySQL Server
Redis Server
```

### Installing

You need to create a Rocket.toml file before running the application.
You also need to create a .env file for Diesel.

```
diesel migration run
```
and then
```
cargo run
```

Then browse to http://\<yourhost\>:\<yourport\>

### Running the tests
Simply run
```
cargo test
```

### And coding style tests

The whole project is styled according to the Rust Formatter.

## Deployment

You need to create a Rocket.toml file before running the application. You want to use the \[production\] environment for deploying the application live. See more on [the official rocket page.](https://rocket.rs/guide/configuration/)
You also need to create a .env file for Diesel.

```
diesel migration run
```
and then
```
cargo run --release
```

## Built With

* [Rocket](https://rocket.rs/) - The web framework used
* [Diesel](http://diesel.rs) - Database Object-Relational Mapping.

## Authors

* **Axel Nilsson** - [GitHub](https://github.com/AxelNilsson)

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE.md](LICENSE.md) file for details
