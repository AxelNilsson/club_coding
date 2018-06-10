#[cfg(test)]
mod test {
    use website;
    use rocket::local::Client;
    use rocket::http::Status;
    use rocket::http::ContentType;

    #[test]
    fn not_found() {
        let client = Client::new(website()).expect("valid rocket instance");
        let response = client.get("/asfjahfkhasjkfhakjsfhkajsf").dispatch();

        assert_eq!(response.status(), Status::NotFound);
        assert_eq!(response.content_type(), Some(ContentType::HTML));
    }
}
