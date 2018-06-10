use futures::{Future, Stream};
use futures::future;
use hyper::Client;
use tokio_core::reactor::Core;
use hyper::Request;
use hyper::Method;
use hyper::header::{Accept, ContentType};
use hyper_tls::HttpsConnector;
use std::io::{Error, ErrorKind};

/// Header for authentication for Postmark.
header! { (PostmarkToken, "X-Postmark-Server-Token") => [String] }

/// URL for the Postmark email endpoint.
/// Should not change in the future,
/// but make sure it is updated if the
/// library does not work.
static URL: &'static str = "https://api.postmarkapp.com/email";

#[derive(Serialize)]
pub struct Headers {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Value")]
    pub value: String,
}

/// The body of the Email endpoint.
#[derive(Default, Serialize)]
pub struct EmailBody {
    #[serde(rename = "From")]
    pub from: String,
    #[serde(rename = "To")]
    pub to: String,
    #[serde(rename = "Cc", skip_serializing_if = "Option::is_none")]
    pub cc: Option<String>,
    #[serde(rename = "Bcc", skip_serializing_if = "Option::is_none")]
    pub bcc: Option<String>,
    #[serde(rename = "Subject", skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(rename = "Tag", skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(rename = "HtmlBody", skip_serializing_if = "Option::is_none")]
    pub html_body: Option<String>,
    #[serde(rename = "TextBody", skip_serializing_if = "Option::is_none")]
    pub text_body: Option<String>,
    #[serde(rename = "ReplyTo", skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
    #[serde(rename = "Headers", skip_serializing_if = "Option::is_none")]
    pub headers: Option<Vec<Headers>>,
    #[serde(rename = "TrackOpens", skip_serializing_if = "Option::is_none")]
    pub track_opens: Option<bool>,
    #[serde(rename = "TrackLinks", skip_serializing_if = "Option::is_none")]
    pub track_links: Option<String>,
    //attachments array
}

/// The structure of the Postmark response.
#[derive(Deserialize, Debug)]
pub struct Response {
    #[serde(rename = "To")]
    to: String,
    #[serde(rename = "SubmittedAt")]
    submitted_at: String,
    #[serde(rename = "MessageID")]
    message_id: String,
    #[serde(rename = "ErrorCode")]
    error_code: i64,
    #[serde(rename = "Message")]
    message: String,
}

/// The client used for the Postmark.
pub struct PostmarkClient {
    /// API Token for Postmark.
    postmark_token: String,
}

impl PostmarkClient {
    /// Function fora  new PostmarkClient.
    /// Takes in an API token and returns a
    /// PostmarkClient.
    pub fn new<Str: Into<String>>(postmark_token: Str) -> PostmarkClient {
        PostmarkClient {
            postmark_token: postmark_token.into(),
        }
    }

    /// Function that sends the actual request
    /// to the email endpoint. Takes in a reference
    /// to an EmailBody and returns a either the Response
    /// or the error that occured.
    pub fn send_email(&self, body: &EmailBody) -> Result<Response, Error> {
        if body.html_body != None || body.text_body != None {
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
                            req.headers_mut()
                                .set(PostmarkToken(self.postmark_token.clone()));

                            req.set_body(json_body);

                            let work = client.request(req).and_then(|res| {
                                res.body()
                                    .fold(Vec::new(), |mut v, chunk| {
                                        v.extend(&chunk[..]);
                                        future::ok::<_, hyper::Error>(v)
                                    })
                                    .and_then(|chunks| match String::from_utf8(chunks) {
                                        Ok(string) => future::ok::<_, hyper::Error>(string),
                                        Err(error) => future::err::<_, hyper::Error>(
                                            hyper::Error::Utf8(error.utf8_error()),
                                        ),
                                    })
                            });
                            match core.run(work) {
                                Ok(resp) => match serde_json::from_str(&resp) {
                                    Ok(response) => Ok(response),
                                    Err(_) => Err(Error::new(
                                        ErrorKind::Other,
                                        "Could not serialize response.",
                                    )),
                                },
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
        } else {
            Err(Error::new(
                ErrorKind::Other,
                "Please provide either a HTML body or a text body.",
            ))
        }
    }
}
