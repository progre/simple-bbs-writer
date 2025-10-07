mod compatible;
mod shitaraba;
#[cfg(test)]
mod test;

use core::str;

use anyhow::{Result, anyhow};
use encoding_rs::{Encoding, UTF_8};
use futures::StreamExt;
use regex::Regex;
use url::Url;

use self::compatible::Compatible;
use self::shitaraba::Shitaraba;

pub const UA: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[async_trait::async_trait]
pub trait Thread: Send + Sync {
    async fn post(&self, charset: &str, name: &str, email: &str, msg: &str) -> Result<()>;
}

pub async fn new(url: &Url) -> Result<Box<dyn Thread>> {
    let host = url.host_str().ok_or_else(|| anyhow!("No host"))?;
    let path = url.path();
    if is_shitaraba_bbs(host, path) {
        Ok(Box::new(Shitaraba::new(url).await?))
    } else {
        Ok(Box::new(Compatible::new(url).await?))
    }
}

fn is_shitaraba_bbs(host: &str, path: &str) -> bool {
    if host == "jbbs.shitaraba.net" {
        return true;
    }
    if path.starts_with("/bbs/read.cgi/") {
        return true;
    }
    false
}

pub enum BbsUrl {
    ProbablyShitarabaThread(Url, Box<dyn Thread>),
    ProbablyShitarabaBoard(Url, String, u64),
    ProbablyCompatibleThread(Url, Box<dyn Thread>),
    MaybeCompatibleBoard(Url, String),
}

pub fn parse_bbs_url(url: Url) -> Result<BbsUrl, Url> {
    if let Some(thread) = shitaraba::parse_thread_url(&url) {
        return Ok(BbsUrl::ProbablyShitarabaThread(url, Box::new(thread)));
    }
    if let Some((dir, board)) = shitaraba::parse_board_url(&url) {
        return Ok(BbsUrl::ProbablyShitarabaBoard(url, dir, board));
    }
    if let Some(thread) = compatible::parse_thread_url(&url) {
        return Ok(BbsUrl::ProbablyCompatibleThread(url, Box::new(thread)));
    }
    if let Some(board) = compatible::parse_board_url(&url) {
        return Ok(BbsUrl::MaybeCompatibleBoard(url, board));
    }
    Err(url)
}

async fn fetch_charset_title_pair(url: &Url) -> Result<(String, String)> {
    let resp = reqwest::Client::new()
        .get(url.clone())
        .header("User-Agent", UA)
        .send()
        .await?
        .error_for_status()?;
    let mut bytes_stream = resp.bytes_stream();
    let mut buf = Vec::new();
    while let Some(chunk) = bytes_stream.next().await {
        let chunk = chunk?;
        buf.append(&mut chunk.to_vec());
        if buf.len() > 1024 {
            break;
        }
    }
    let (charset, text) = 'block: {
        let Some(charset) = regex::bytes::Regex::new(r#"charset=([^;"]+)|charset="(.+)""#)
            .unwrap()
            .captures(&buf)
        else {
            break 'block (UTF_8.name().to_owned(), String::from_utf8_lossy(&buf));
        };
        let charset = charset
            .get(1)
            .or_else(|| charset.get(2))
            .unwrap()
            .as_bytes();
        let Some(encoding) = Encoding::for_label(charset) else {
            break 'block (UTF_8.name().to_owned(), String::from_utf8_lossy(&buf));
        };
        let (text, _, _) = encoding.decode(&buf);
        (String::from_utf8_lossy(charset).to_string(), text)
    };
    Ok((
        charset,
        Regex::new(r"(?i)<title>(.*?)</title>")
            .unwrap()
            .captures(text.as_ref())
            .map(|x| x[1].to_owned())
            .unwrap_or_default(),
    ))
}

pub async fn fetch_thread_url_encoding_name(bbs_url: &BbsUrl) -> Result<(Url, String, String)> {
    match bbs_url {
        BbsUrl::ProbablyShitarabaThread(url, _thread) => {
            let (encoding, title) = fetch_charset_title_pair(url).await?;
            Ok((url.clone(), encoding, title))
        }
        BbsUrl::ProbablyShitarabaBoard(url, dir, board) => {
            let origin = url.origin().ascii_serialization();
            let url = shitaraba::fetch_latest_thread_url(&origin, dir, *board).await?;
            let (encoding, title) = fetch_charset_title_pair(&url).await?;
            Ok((url, encoding, title))
        }
        BbsUrl::ProbablyCompatibleThread(url, _thread) => {
            let (encoding, title) = fetch_charset_title_pair(url).await?;
            Ok((url.clone(), encoding, title))
        }
        BbsUrl::MaybeCompatibleBoard(url, board) => {
            let origin = url.origin().ascii_serialization();
            let url = compatible::fetch_latest_thread_url(&origin, board).await?;
            let (encoding, title) = fetch_charset_title_pair(&url).await?;
            Ok((url, encoding, title))
        }
    }
}
