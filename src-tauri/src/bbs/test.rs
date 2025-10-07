use url::Url;

use crate::bbs::BbsUrl;

struct EmptyThread;

#[async_trait::async_trait]
impl super::Thread for EmptyThread {
    async fn post(
        &self,
        _charset: &str,
        _name: &str,
        _email: &str,
        _msg: &str,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

#[test]
fn test_parse_bbs_url() {
    let data: [(&str, Option<BbsUrl>); _] = [
        (
            "https://jbbs.shitaraba.net/bbs/read.cgi/radio/22607/1484488601/l50",
            Some(BbsUrl::ProbablyShitarabaThread(
                Url::parse("https://jbbs.shitaraba.net/bbs/read.cgi/radio/22607/1484488601/l50")
                    .unwrap(),
                Box::new(EmptyThread),
            )),
        ),
        (
            "https://jbbs.shitaraba.net/radio/22607/",
            Some(BbsUrl::ProbablyShitarabaBoard(
                Url::parse("https://jbbs.shitaraba.net/radio/22607/").unwrap(),
                "radio".to_string(),
                22607,
            )),
        ),
        (
            "https://bbs.jpnkn.com/test/read.cgi/progre/1749359408/l50",
            Some(BbsUrl::ProbablyCompatibleThread(
                Url::parse("https://bbs.jpnkn.com/test/read.cgi/progre/1749359408/l50").unwrap(),
                Box::new(EmptyThread),
            )),
        ),
        (
            "https://bbs.jpnkn.com/progre/",
            Some(BbsUrl::MaybeCompatibleBoard(
                Url::parse("https://bbs.jpnkn.com/progre/").unwrap(),
                "progre".to_string(),
            )),
        ),
        ("https://example.com/", None),
    ];

    for (url_str, expected) in data {
        let url = Url::parse(url_str).unwrap();
        let result = super::parse_bbs_url(url);
        match (result, expected) {
            (
                Ok(BbsUrl::ProbablyShitarabaThread(u1, _)),
                Some(BbsUrl::ProbablyShitarabaThread(u2, _)),
            ) => {
                assert_eq!(u1, u2);
            }
            (
                Ok(BbsUrl::ProbablyShitarabaBoard(u1, d1, b1)),
                Some(BbsUrl::ProbablyShitarabaBoard(u2, d2, b2)),
            ) => {
                assert_eq!(u1, u2);
                assert_eq!(d1, d2);
                assert_eq!(b1, b2);
            }
            (
                Ok(BbsUrl::ProbablyCompatibleThread(u1, _)),
                Some(BbsUrl::ProbablyCompatibleThread(u2, _)),
            ) => {
                assert_eq!(u1, u2);
            }
            (
                Ok(BbsUrl::MaybeCompatibleBoard(u1, b1)),
                Some(BbsUrl::MaybeCompatibleBoard(u2, b2)),
            ) => {
                assert_eq!(u1, u2);
                assert_eq!(b1, b2);
            }
            (Err(_), None) => {}
            _ => panic!("Mismatched result for URL: {}", url_str),
        }
    }
}
