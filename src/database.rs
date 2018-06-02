use diesel::MysqlConnection;
use diesel::r2d2::ConnectionManager;
use std::ops::Deref;
use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Outcome, Request, State};
use rocket::fairing::AdHoc;

/// Redefines MySQLPool as an r2d2 pool with
/// a connection manager to a MySQL connection.
pub type MySqlPool = r2d2::Pool<ConnectionManager<MysqlConnection>>;

/// Returns a AdHoc Fairing with a connection
/// to the MySQL database.
/// Will panic if no MySQL credentials are set
/// in Rocket.toml File
pub fn fairing() -> rocket::fairing::AdHoc {
    AdHoc::on_attach(|rocket| {
        let config = rocket.config().clone();

        let user = config
            .get_str("mysql_user")
            .expect("mysql_user not specified");

        let password = config
            .get_str("mysql_password")
            .expect("mysql_password not specified");

        let host = config
            .get_str("mysql_host")
            .expect("mysql_host not specified");

        let port = config
            .get_str("mysql_port")
            .expect("mysql_port not specified");

        let database = config
            .get_str("mysql_database")
            .expect("mysql_database not specified");

        let database_url = format!(
            "mysql://{}:{}@{}:{}/{}",
            user, password, host, port, database
        );

        let manager = ConnectionManager::<MysqlConnection>::new(database_url);
        let pool = r2d2::Pool::new(manager).expect("db pool");

        Ok(rocket.manage(pool))
    })
}

/// Connection request guard type:
/// a wrapper around an r2d2 pooled connection.
pub struct DbConn(pub r2d2::PooledConnection<ConnectionManager<MysqlConnection>>);

/// Request guard for retrieving a single connection from the managed database pool. If
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

/// For the convenience of using an
/// &DbConn as an &MysqlConnection.
impl Deref for DbConn {
    type Target = MysqlConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
