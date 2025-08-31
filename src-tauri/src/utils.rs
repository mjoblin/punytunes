use url::Url;

pub fn host_from_url(url: &str) -> Option<String> {
    if let Ok(url_info) = Url::parse(&url) {
        return Some(url_info.host_str().unwrap().to_owned());
    }

    None
}
