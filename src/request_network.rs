use futures::{Future, Stream};
use futures::future;
use hyper::Client;
use tokio_core::reactor::Core;
use hyper::Request;
use hyper::Method;
use hyper::header::{Accept, ContentType};
use hyper_tls::HttpsConnector;
use std::io::{Error, ErrorKind};

static URL: &'static str = "https://sign.wooreq.com/sign";

#[derive(Default, Serialize)]
pub struct ReqBody<'a> {
    pub to_pay: &'a str,
    pub to_address: &'a str,
    pub redirect_url: &'a str,
    pub order_id: &'a str,
    pub reason: &'a str,
    pub network: u8,
}

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
                            .fold(Vec::new(), |mut v, chunk| {
                                v.extend(&chunk[..]);
                                future::ok::<_, hyper::Error>(v)
                            })
                            .and_then(|chunks| {
                                let s = String::from_utf8(chunks).unwrap();
                                future::ok::<_, hyper::Error>(s)
                            })
                    });
                    match core.run(work) {
                        Ok(user) => Ok(user),
                        Err(_) => Err(Error::new(ErrorKind::Other, "oh no!")),
                    }
                }
                Err(_) => Err(Error::new(ErrorKind::Other, "oh no!")),
            }
        }
        Err(_) => Err(Error::new(ErrorKind::Other, "oh no!")),
    }
}
