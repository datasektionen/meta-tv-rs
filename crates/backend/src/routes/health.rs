#[get("/health")]
pub fn health() -> &'static str {
    "OK"
}

#[cfg(test)]
mod tests {
    use rocket::http::Status;
    use rocket::local::blocking::Client;

    #[test]
    fn health() {
        let client = Client::tracked(crate::rocket()).unwrap();
        let response = client.get("/api/health").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string(), Some("OK".into()));
    }
}
