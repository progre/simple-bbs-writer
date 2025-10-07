use anyhow::{Result, anyhow, bail};
use encoding_rs::Encoding;
use percent_encoding::{NON_ALPHANUMERIC, percent_encode};
use regex::Regex;
use reqwest::header::{CONTENT_TYPE, REFERER, USER_AGENT};
use tracing::{debug, trace};
use url::Url;

use super::{Thread, UA};

pub fn parse_thread_url(thread_url: &Url) -> Option<Shitaraba> {
    let origin = thread_url.origin().ascii_serialization();
    let c = Regex::new(r"^/bbs/read.cgi/(.+?)/(.+?)/(.+?)(:?/.*)?$")
        .unwrap()
        .captures(thread_url.path())?;
    let dir = c.get(1).unwrap().as_str().to_string();
    let bbs = c.get(2).unwrap().as_str().parse().ok()?;
    let key = c.get(3).unwrap().as_str().parse().ok()?;
    Some(Shitaraba {
        origin,
        dir,
        bbs,
        key,
    })
}

pub fn parse_board_url(board_url: &Url) -> Option<(String, u64)> {
    let c = Regex::new(r"^https://jbbs.shitaraba.net/(.+?)/([0-9]+)/?$")
        .unwrap()
        .captures(board_url.as_str())?;
    Some((
        c.get(1).unwrap().as_str().to_string(),
        c.get(2).unwrap().as_str().parse().ok()?,
    ))
}

async fn fetch_subject_txt(origin: &str, dir: &str, bbs: u64) -> Result<String> {
    let subject_url = format!("{}/{}/{}/subject.txt", origin, dir, bbs);
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

async fn fetch_latest_thread(origin: &str, dir: &str, bbs: u64) -> Result<u64> {
    let subject_txt = fetch_subject_txt(origin, dir, bbs).await?;
    read_latest_thread(&subject_txt)
}

pub async fn fetch_latest_thread_url(origin: &str, dir: &str, bbs: u64) -> Result<Url> {
    let key = fetch_latest_thread(origin, dir, bbs)
        .await
        .inspect_err(|err| trace!("{:?}", err))?;
    let thread_url = format!("{}/bbs/read.cgi/{}/{}/{}/", origin, dir, bbs, key);
    Ok(Url::parse(&thread_url).unwrap())
}

fn charset_percent_encode(encoding: &'static Encoding, text: &str) -> String {
    let (text, _, _) = encoding.encode(text);
    percent_encode(&text, NON_ALPHANUMERIC).to_string()
}

pub struct Shitaraba {
    origin: String,
    dir: String,
    bbs: u64,
    key: u64,
}

impl Shitaraba {
    pub async fn new(url: &Url) -> Result<Self> {
        if let Some(site) = parse_thread_url(url) {
            return Ok(site);
        }
        bail!("Invalid URL: {}", url);
    }
}

#[async_trait::async_trait]
impl Thread for Shitaraba {
    async fn post(&self, charset: &str, name: &str, email: &str, msg: &str) -> Result<()> {
        let encoding = Encoding::for_label(charset.as_bytes()).unwrap();
        let name = charset_percent_encode(encoding, name);
        let email = charset_percent_encode(encoding, email);
        let msg = charset_percent_encode(encoding, msg);
        let resp = reqwest::Client::new()
            .post(format!(
                "{}/bbs/write.cgi/{}/{}/{}/",
                self.origin, self.dir, self.bbs, self.key
            ))
            .header(
                CONTENT_TYPE,
                format!("application/x-www-form-urlencoded; charset={}", charset),
            )
            .header(
                REFERER,
                format!(
                    "{}/bbs/read.cgi/{}/{}/{}/",
                    self.origin, self.dir, self.bbs, self.key
                ),
            )
            .header(USER_AGENT, UA)
            .body(format!(
                "BBS={}&KEY={}&DIR={}&NAME={}&MAIL={}&MESSAGE={}",
                self.bbs, self.key, self.dir, name, email, msg
            ))
            .send()
            .await?
            .error_for_status()?;
        let bytes = resp.bytes().await?;
        let text = Encoding::for_label(b"euc-jp").unwrap().decode(&bytes).0;
        debug!("post resp: {}", text.to_string());
        Ok(())
    }
}
