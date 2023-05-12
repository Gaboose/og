mod allow_all_origins;

use crate::allow_all_origins::AllowAllOrigins;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::convert::Infallible;
use std::error::Error;
use std::net::SocketAddr;
use std::str;
use tokio::net::TcpListener;

// Name the user agent after the app.
static APP_USER_AGENT: &str = concat!("OpenGraph Fetcher ", env!("CARGO_PKG_VERSION"));

fn parse_query(r: Request<hyper::body::Incoming>) -> Option<String> {
    let mut path = String::from(&r.uri().path()[1..]);
    if path == "" {
        return None;
    }

    if !path.starts_with("https://") && !path.starts_with("http://") {
        path = String::from("https://") + &path;
    }

    if let Some(q) = r.uri().query() {
        path = path + "?" + q;
    }

    Some(path)
}

async fn fetch_body(url: &str) -> Result<Bytes, reqwest::Error> {
    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()?;
    let resp = client.get(url).send().await?;
    Ok(resp.bytes().await?)
}

type OpenGraphData = HashMap<String, String>;

fn parse_open_graph(html: &str) -> OpenGraphData {
    let document = Html::parse_document(html);
    let selector = Selector::parse("meta[property^=\"og:\"]").unwrap();

    let mut ogd = HashMap::new();

    for element in document.select(&selector) {
        let elval = element.value();
        match (elval.attr("property"), elval.attr("content")) {
            (Some(p), Some(c)) => {
                ogd.insert(String::from(p), String::from(c));
            }
            _ => {}
        };
    }

    ogd
}

async fn handler(r: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let url = match parse_query(r) {
        Some(s) => s,
        None => {
            return Ok(Response::new(APP_USER_AGENT.into()));
        }
    };

    let body_bts = match fetch_body(&url).await {
        Ok(bts) => bts,
        Err(e) => {
            let mut resp = Response::new(e.to_string().into());
            if let Some(status) = e.status() {
                *resp.status_mut() = status;
            } else {
                *resp.status_mut() = hyper::StatusCode::BAD_REQUEST;
            }
            return Ok(resp);
        }
    };

    let body_str = match str::from_utf8(&body_bts) {
        Ok(s) => s,
        Err(e) => {
            let mut resp = Response::new(e.to_string().into());
            *resp.status_mut() = hyper::StatusCode::INTERNAL_SERVER_ERROR;
            return Ok(resp);
        }
    };

    let ogd = parse_open_graph(&body_str);

    let ogd_json = match serde_json::to_string(&ogd) {
        Ok(s) => s,
        Err(e) => {
            let mut resp = Response::new(e.to_string().into());
            *resp.status_mut() = hyper::StatusCode::INTERNAL_SERVER_ERROR;
            return Ok(resp);
        }
    };

    Ok(Response::new(ogd_json.into()))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let port: u16 = std::env::var("PORT")
        .unwrap_or_default()
        .parse()
        .unwrap_or(8000);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(stream, AllowAllOrigins::new(service_fn(handler)))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}
