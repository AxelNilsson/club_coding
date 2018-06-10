#[cfg(test)]
mod test {
    use website;
    use rocket::local::Client;
    use rocket::http::Status;
    use rocket::http::ContentType;

    #[test]
    fn watch() {
        let client = Client::new(website()).expect("valid rocket instance");
        let response = client
            .get("/watch/07F812BDA6CAB3CA44CE372E8CD511D58551167FA0945D93")
            .dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::HTML));
    }
}
