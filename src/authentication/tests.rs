#[cfg(test)]
mod test {
    use website;
    use rocket::local::Client;
    use rocket::http::Status;
    use rocket::http::ContentType;

    #[test]
    fn authentication() {
        let client = Client::new(website()).expect("valid rocket instance");
        let response = client.get("/").dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::HTML));
    }
}
