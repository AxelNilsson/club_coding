use reqwest::header::{CONTENT_TYPE, HeaderName};
use std::io::{Error, ErrorKind};

/// URL for the Postmark email endpoint.
/// Should not change in the future,
/// but make sure it is updated if the
/// library does not work.
static URL: &'static str = "https://api.postmarkapp.com/email";
static POSTMARK_HEADER: &'static str = "x-postmark-server-token";

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
            let json = serde_json::to_string(&body)?;

            let client = reqwest::Client::new();
            let mut res = match client.post(URL)
                .header(CONTENT_TYPE, "application/json")
                .header(HeaderName::from_static(POSTMARK_HEADER), self.postmark_token.clone())
                .body(json)
                .send() {
                    Ok(res) => res,
                    Err(_) => return Err(Error::new(
                        ErrorKind::Other,
                        "Could not connect to server at URL.",
                    )),
                };

            let text = match res.text() {
                Ok(text) => text,
                Err(_) => return Err(Error::new(
                    ErrorKind::Other,
                    "Could not read response.",
                )),
            };

            let response = match serde_json::from_str(&text) {
                Ok(response) => Ok(response),
                Err(_) => Err(Error::new(
                    ErrorKind::Other,
                    "Could not serialize response.",
                )),
            };
            response
        } else {
            Err(Error::new(
                ErrorKind::Other,
                "Please provide either a HTML body or a text body.",
            ))
        }
    }
}
