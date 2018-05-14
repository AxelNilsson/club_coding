use users::User;

#[derive(Serialize)]
pub struct Context {
    pub header: String,
}

#[derive(Serialize)]
pub struct LoggedInContext {
    pub header: String,
    pub user: User,
}
