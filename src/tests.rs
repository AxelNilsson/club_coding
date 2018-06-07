#[cfg(test)]
mod test {
    use website;
    use rocket::local::Client;
    use rocket::http::Status;
    use rocket::http::ContentType;

    #[test]
    fn index() {
        let client = Client::new(website()).expect("valid rocket instance");
        let response = client.get("/").dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::HTML));
    }

    #[test]
    fn terms_of_service() {
        let client = Client::new(website()).expect("valid rocket instance");
        let response = client.get("/terms_of_service").dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::HTML));
    }

    #[test]
    fn cookie_policy() {
        let client = Client::new(website()).expect("valid rocket instance");
        let response = client.get("/cookie_policy").dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::HTML));
    }

    #[test]
    fn privacy_policy() {
        let client = Client::new(website()).expect("valid rocket instance");
        let response = client.get("/privacy_policy").dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::HTML));
    }
}
