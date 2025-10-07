use anyhow::{Result, anyhow, bail};
use encoding_rs::Encoding;
use percent_encoding::{NON_ALPHANUMERIC, percent_encode};
use regex::Regex;
use reqwest::header::USER_AGENT;
use tracing::debug;
use url::Url;

use super::{Thread, UA};

pub fn parse_thread_url(thread_url: &Url) -> Option<Compatible> {
    let origin = thread_url.origin().ascii_serialization();
    let c = Regex::new(r"^/test/read.cgi/(.+?)/(.+?)(:?/.*)?$")
        .unwrap()
        .captures(thread_url.path())?;
    let bbs = c.get(1).unwrap().as_str().to_string();
    let key = c.get(2).unwrap().as_str().parse().ok()?;
    Some(Compatible { origin, bbs, key })
}

pub fn parse_board_url(board_url: &Url) -> Option<String> {
    let c = Regex::new(r"^/(.+?)/?$")
        .unwrap()
        .captures(board_url.path())?;
    Some(c.get(1).unwrap().as_str().to_string())
}

async fn fetch_subject_txt(origin: &str, bbs: &str) -> Result<String> {
    let subject_url = format!("{}/{}/subject.txt", origin, bbs);
    Ok(reqwest::get(subject_url).await?.text().await?)
}

fn read_latest_thread(subject_txt: &str) -> Result<u64> {
    Ok(subject_txt
        .lines()
        .next()
        .ok_or_else(|| anyhow!("Empty subject.txt"))?
        .split(".")
        .next()
        .ok_or_else(|| anyhow!("Empty subject.txt"))?
        .parse()?)
}

async fn fetch_latest_thread(origin: &str, bbs: &str) -> Result<u64> {
    let subject_txt = fetch_subject_txt(origin, bbs).await?;
    read_latest_thread(&subject_txt)
}

pub async fn fetch_latest_thread_url(origin: &str, bbs: &str) -> Result<Url> {
    let key = fetch_latest_thread(origin, bbs).await?;
    let thread_url = format!("{}/test/read.cgi/{}/{}", origin, bbs, key);
    Ok(Url::parse(&thread_url).unwrap())
}

fn charset_percent_encode(encoding: &'static Encoding, text: &str) -> String {
    let (text, _, _) = encoding.encode(text);
    percent_encode(&text, NON_ALPHANUMERIC).to_string()
}

pub struct Compatible {
    origin: String,
    bbs: String,
    key: u64,
}

impl Compatible {
    pub async fn new(url: &Url) -> Result<Self> {
        if let Some(site) = parse_thread_url(url) {
            return Ok(site);
        }
        bail!("Invalid URL: {}", url);
    }
}

#[async_trait::async_trait]
impl Thread for Compatible {
    async fn post(&self, charset: &str, name: &str, email: &str, msg: &str) -> Result<()> {
        let encoding = Encoding::for_label(charset.as_bytes()).unwrap();
        let name = charset_percent_encode(encoding, name);
        let email = charset_percent_encode(encoding, email);
        let msg = charset_percent_encode(encoding, msg);
        let resp = reqwest::Client::new()
            .post(format!("{}/test/bbs.cgi", self.origin))
            .header(USER_AGENT, UA)
            .header(
                "Content-Type",
                format!("application/x-www-form-urlencoded; charset={}", charset),
            )
            .header("Cookie", format!(r#"NAME="{}";MAIL="{}""#, name, email))
            .body(format!(
                "FROM={}&mail={}&MESSAGE={}&key={}&bbs={}",
                name, email, msg, self.key, self.bbs
            ))
            .send()
            .await?
            .error_for_status()?;
        let bytes = resp.bytes().await?;
        let text = Encoding::for_label(b"shift_jis").unwrap().decode(&bytes).0;
        debug!("post resp: {}", text.to_string());
        Ok(())
    }
}
