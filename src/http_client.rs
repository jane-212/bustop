use std::sync::Arc;

use futures::future::BoxFuture;
use gpui::App;
use http_client::http::{HeaderValue, header};
use http_client::{AsyncBody, HttpClient, Request, Response, Url};
use reqwest_client::ReqwestClient;

pub fn init(cx: &mut App) {
    let http_client = MultiHttpClient::new();
    cx.set_http_client(Arc::new(http_client));
}

pub struct MultiHttpClient {
    client: ReqwestClient,
}

impl MultiHttpClient {
    fn new() -> Self {
        let client = ReqwestClient::user_agent("bustop").expect("failed to create http client");

        Self { client }
    }
}

impl HttpClient for MultiHttpClient {
    fn type_name(&self) -> &'static str {
        self.client.type_name()
    }

    fn user_agent(&self) -> Option<&HeaderValue> {
        self.client.user_agent()
    }

    fn send(
        &self,
        mut req: Request<AsyncBody>,
    ) -> BoxFuture<'static, anyhow::Result<Response<AsyncBody>>> {
        let uri = req.uri();
        let host = HostSite::from(uri.host());
        let headers = req.headers_mut();
        match host {
            HostSite::Avatar => {
                headers.insert(
                    header::REFERER,
                    HeaderValue::from_static("https://www.javbus.com/"),
                );
            }
            HostSite::Preview => {
                headers.insert(
                    header::REFERER,
                    HeaderValue::from_static("https://www.javbus.com/forum/forum.php"),
                );
            }
            HostSite::Image => {
                headers.insert(
                    header::REFERER,
                    HeaderValue::from_static("https://www.javbus.com/"),
                );
            }
            HostSite::Unknown => (),
        }

        self.client.send(req)
    }

    fn proxy(&self) -> Option<&Url> {
        self.client.proxy()
    }
}

enum HostSite {
    Avatar,
    Preview,
    Image,
    Unknown,
}

impl From<Option<&str>> for HostSite {
    fn from(value: Option<&str>) -> Self {
        match value {
            Some(host) => match host {
                "uc.javbus22.com" => HostSite::Avatar,
                "www.javbus.com" => HostSite::Preview,
                "forum.javcdn.cc" => HostSite::Image,
                _ => HostSite::Unknown,
            },
            None => HostSite::Unknown,
        }
    }
}
