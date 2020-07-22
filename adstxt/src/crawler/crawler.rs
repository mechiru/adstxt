use hyper::{body, header, Client, Method, Request, StatusCode};
use tokio::time;

use std::{fs, path, pin::Pin};

use crate::crawler;

pub struct Config {
    pub chunk_size: usize,
    pub out_dir: path::PathBuf,
    pub timeout: time::Duration,
}

pub async fn crawle(config: Config, domains: Vec<String>) -> crawler::Result<()> {
    let start = std::time::Instant::now();
    log::info!("start crawle ...");

    let c = new_hyper_client();
    let len = domains.len();

    let mut counter = 0usize;

    let mut handles = Vec::new();
    let mut handles = Pin::new(&mut handles);

    macro_rules! current {
        () => {
            log::info!(
                "current: {:>7} / {}, elapsed: {:?}",
                counter,
                len,
                start.elapsed(),
            );
        };
    }

    for chunk in domains.chunks(config.chunk_size) {
        for domain in chunk {
            handles.push(tokio::spawn({
                let c = c.clone();
                let domain = domain.to_owned();
                let timeout = config.timeout;
                let mut path = config.out_dir.clone();

                async move {
                    match crawle_until_find(c, domain.as_ref(), timeout).await {
                        Ok(Some(txt)) => {
                            path.push(domain.as_str());
                            if let Err(e) = fs::write(path, txt) {
                                log::error!("{}: {:?}", domain, e);
                            }
                        }
                        Ok(None) => {}
                        // TODO:
                        // Crawle(hyper::Error(Io, Os { code: 54, kind: ConnectionReset, message: "Connection reset by peer" }))
                        // Crawle(hyper::Error(ChannelClosed))
                        // Crawle(hyper::Error(IncompleteMessage))
                        Err(crawler::Error::BodyEncoding(_))
                        | Err(crawler::Error::HeaderEncoding(_)) => {}
                        Err(e) => log::error!("{}: {:?}", domain, e),
                    }
                }
            }));
        }

        let mut chunk = Vec::new();
        chunk.append(&mut handles);
        for handle in chunk {
            handle.await?;
            counter += 1;
            if counter % 10000usize == 0 {
                current!();
            }
        }
    }

    current!();
    log::info!("done!");

    Ok(())
}

const USER_AGENT: &str = concat!(
    "ads.txt crawler/1.0.2; +github.com/mechiru/",
    env!("CARGO_PKG_NAME"),
    " v",
    env!("CARGO_PKG_VERSION")
);

type HyperClient = Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>, body::Body>;

fn new_hyper_client() -> HyperClient {
    Client::builder()
        .pool_idle_timeout(None)
        .pool_max_idle_per_host(0)
        .http2_keep_alive_interval(None)
        .build(hyper_tls::HttpsConnector::new())
}

async fn crawle_until_find(
    c: HyperClient,
    domain: &str,
    timeout: time::Duration,
) -> crawler::Result<Option<String>> {
    let uri = format!("http://{}/ads.txt", domain);

    let uri = match fetch_with_timeout(c.clone(), uri, timeout).await? {
        Response::NotFound => format!("https://{}/ads.txt", domain),
        Response::Found { location } => {
            if !location.contains(domain) || !location.contains("ads.txt") {
                return Ok(None);
            }
            location
        }
        Response::Success { data } => return Ok(Some(data)),
    };

    match fetch_with_timeout(c, uri, timeout).await? {
        Response::NotFound | Response::Found { .. } => Ok(None),
        Response::Success { data } => Ok(Some(data)),
    }
}

enum Response {
    NotFound,
    Found { location: String },
    Success { data: String },
}

async fn fetch(c: HyperClient, uri: String) -> crawler::Result<Response> {
    let req = Request::builder()
        .header(header::USER_AGENT, USER_AGENT)
        .header(header::ACCEPT, "text/plain")
        .method(Method::GET)
        .uri(uri)
        .body(body::Body::empty())?;

    let resp = match c.request(req).await {
        Err(e) if e.is_connect() => return Ok(Response::NotFound),
        result => result,
    }?;

    if matches!(
        resp.status(),
        StatusCode::MOVED_PERMANENTLY
            | StatusCode::FOUND
            | StatusCode::TEMPORARY_REDIRECT
            | StatusCode::PERMANENT_REDIRECT
    ) {
        let ret = if let Some(location) = resp.headers().get(header::LOCATION) {
            Response::Found {
                location: location.to_str()?.to_owned(),
            }
        } else {
            Response::NotFound
        };
        return Ok(ret);
    };

    let ret = if resp.status().is_success() {
        if let Some(ctype) = resp.headers().get(header::CONTENT_TYPE) {
            if !ctype.as_ref().starts_with(b"text/plain") {
                return Ok(Response::NotFound);
            }
        }
        let data = body::to_bytes(resp.into_body()).await?;
        let data = String::from_utf8(data.to_vec())?;
        Response::Success { data }
    } else {
        Response::NotFound
    };
    Ok(ret)
}

async fn fetch_with_timeout(
    c: HyperClient,
    uri: String,
    dur: time::Duration,
) -> crawler::Result<Response> {
    tokio::select! {
        _ = time::delay_for(dur) => Ok(Response::NotFound),
        resp = fetch(c, uri) => resp
    }
}
