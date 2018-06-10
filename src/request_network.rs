use futures::{Future, Stream};
use futures::future;
use hyper::Client;
use tokio_core::reactor::Core;
use hyper::Request;
use hyper::Method;
use hyper::header::{Accept, ContentType};
use hyper_tls::HttpsConnector;
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
    let json_body = serde_json::to_string(&body)?;

    let mut core = Core::new()?;
    match HttpsConnector::new(4, &core.handle()) {
        Ok(connector) => {
            let client = Client::configure()
                .connector(connector)
                .build(&core.handle());

            match URL.parse() {
                Ok(uri) => {
                    let mut req = Request::new(Method::Post, uri);
                    req.headers_mut().set(Accept::json());
                    req.headers_mut().set(ContentType::json());

                    req.set_body(json_body);

                    let work = client.request(req).and_then(|res| {
                        res.body()
                            .fold(Vec::new(), |mut vector, chunk| {
                                vector.extend(&chunk[..]);
                                future::ok::<_, hyper::Error>(vector)
                            })
                            .and_then(|chunks| match String::from_utf8(chunks) {
                                Ok(string) => future::ok::<_, hyper::Error>(string),
                                Err(error) => future::err::<_, hyper::Error>(hyper::Error::Utf8(
                                    error.utf8_error(),
                                )),
                            })
                    });
                    match core.run(work) {
                        Ok(user) => Ok(user),
                        Err(_) => Err(Error::new(
                            ErrorKind::Other,
                            "Could not connect to server at URL.",
                        )),
                    }
                }
                Err(_) => Err(Error::new(
                    ErrorKind::Other,
                    "Could not parse URL. Please provide a valid URL.",
                )),
            }
        }
        Err(_) => Err(Error::new(
            ErrorKind::Other,
            "Could not create HTTPS connector.",
        )),
    }
}
