use diesel::MysqlConnection;
use diesel::r2d2::ConnectionManager;
use std::ops::Deref;
use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Outcome, Request, State};
use rocket::config::{Config, Environment};

type MySqlPool = r2d2::Pool<ConnectionManager<MysqlConnection>>;

pub fn init_pool() -> MySqlPool {
    let config = Config::build(Environment::Development)
        .extra("database_url", "mysql://axel:Testing1@localhost/youtube")
        .unwrap();

    let database_url = config
        .get_str("database_url")
        .expect("DATABASE_URL must be set!");

    let manager = ConnectionManager::<MysqlConnection>::new(database_url);
    r2d2::Pool::new(manager).expect("db pool")
}

// Connection request guard type: a wrapper around an r2d2 pooled connection.
pub struct DbConn(pub r2d2::PooledConnection<ConnectionManager<MysqlConnection>>);

/// Attempts to retrieve a single connection from the managed database pool. If
/// no pool is currently managed, fails with an `InternalServerError` status. If
/// no connections are available, fails with a `ServiceUnavailable` status.
impl<'a, 'r> FromRequest<'a, 'r> for DbConn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let pool = request.guard::<State<MySqlPool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(DbConn(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
        }
    }
}

// For the convenience of using an &DbConn as an &MysqlConnection.
impl Deref for DbConn {
    type Target = MysqlConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
