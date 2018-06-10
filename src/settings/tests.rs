#[cfg(test)]
mod test {
    use website;
    use rocket::local::Client;
    use rocket::http::Status;

    #[test]
    fn update_password_nologin() {
        let client = Client::new(website()).expect("valid rocket instance");
        let response = client.get("/settings/password").dispatch();

        assert_eq!(response.status(), Status::SeeOther);
        assert_eq!(response.content_type(), None);
    }
}
