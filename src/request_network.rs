use reqwest::header::CONTENT_TYPE;
use std::io::{Error, ErrorKind};

/// URL for the WooREQ Sign endpoint.
/// Could possibly change in the future,
/// so make sure it is updated if the
/// library does not work.
static URL: &'static str = "https://sign.wooreq.com/sign";

/// The Request Body Struct is used for the
/// data required for the signature.
#[derive(Default, Serialize)]
pub struct ReqBody<'a> {
    /// Amount to pay.
    pub to_pay: &'a str,
    /// Address to send the currency to
    pub to_address: &'a str,
    /// URL to get redirected to once the payment
    /// has been made.
    pub redirect_url: &'a str,
    /// ID of the order.
    pub order_id: &'a str,
    /// Reason for the payment.
    pub reason: &'a str,
    /// Network to use. 1 will almost always
    /// be used. 1 is mainnet.
    pub network: u8,
}

/// Sends a request to the URL as defined by the
/// static str URL with a ReqBody. Returns either
/// an error or the Request Network String for the
/// actual payment.
pub fn wooreq_request(body: &ReqBody) -> Result<String, Error> {
    let json = serde_json::to_string(&body)?;

    let client = reqwest::Client::new();
    let mut res = match client.post(URL)
        .header(CONTENT_TYPE, "application/json")
        .body(json)
        .send() {
            Ok(res) => res,
            Err(_) => return Err(Error::new(
                ErrorKind::Other,
                "Could not connect to server at URL.",
            )),
        };

    match res.text() {
        Ok(text) => Ok(text),
        Err(_) => Err(Error::new(
                ErrorKind::Other,
                "Could not read response.",
            )),
    }
}
