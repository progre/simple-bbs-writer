use anyhow::Ok;
use tauri::async_runtime::spawn;
use url::Url;

use crate::bbs::{self, fetch_thread_url_encoding_name, parse_bbs_url};

type CommentObserver = Option<Box<dyn Fn(String) + Send + 'static>>;

pub struct PopoverViewModel {
    url: String,
    comment: String,
    comment_observer: CommentObserver,
    sage: bool,
}

impl PopoverViewModel {
    pub fn new() -> Self {
        Self {
            url: String::new(),
            comment: String::new(),
            comment_observer: None,
            sage: true, // デフォルトでsageを有効にする
        }
    }

    pub fn get_sage(&self) -> bool {
        self.sage
    }

    pub fn set_url(&mut self, url: String) {
        self.url = url;
    }

    pub fn set_comment(&mut self, comment: String) {
        self.comment = comment.clone();
        if let Some(observer) = self.comment_observer.as_ref() {
            observer(comment);
        }
    }

    pub fn set_sage(&mut self, sage: bool) {
        self.sage = sage;
    }

    pub fn subscribe_comment<F>(&mut self, observer: F)
    where
        F: Fn(String) + Send + 'static,
    {
        self.comment_observer = Some(Box::new(observer));
    }

    pub fn on_post_clicked(&mut self) {
        let url = self.url.clone();
        let comment = self.comment.clone();
        let sage = self.sage;
        self.set_comment(String::new());
        spawn(async move {
            let url: Url = url.parse()?;
            let bbs_url = parse_bbs_url(url).map_err(|_| anyhow::anyhow!("Invalid BBS URL"))?;
            let (thread_url, encoding, _title) = fetch_thread_url_encoding_name(&bbs_url).await?;
            let bbs = bbs::new(&thread_url).await?;

            let email = if sage { "sage" } else { "" };
            bbs.post(&encoding, "", email, &comment).await?;

            Ok(())
        });
    }
}
