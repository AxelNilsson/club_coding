#[cfg(test)]
mod test {
    use website;
    use rocket::local::Client;
    use rocket::http::Status;
    use rocket::http::ContentType;

    #[test]
    fn series() {
        let client = Client::new(website()).expect("valid rocket instance");
        let response = client
            .get("/series/9C2D35BB3D02B96AA0D5F994FBDA32B4C4349988A5A531A5")
            .dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::HTML));
    }
}
