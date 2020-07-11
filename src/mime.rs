use std::fmt;

const HTML: &str = "text/html; charset=utf-8";
const CSS: &str = "text/css; charset=utf-8";
const JS: &str = "text/javascript; charset=utf-8";

#[derive(Debug, Clone, PartialEq)]
pub enum Mime {
    Html,
    Css,
    Js,
}

impl fmt::Display for Mime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match *self {
            Mime::Html => HTML,
            Mime::Css => CSS,
            Mime::Js => JS,
        };

        write!(f, "{}", string)
    }
}

pub trait MimeAware {
    fn content_type(self, mime: Mime) -> Self;
}

impl MimeAware for http::response::Builder {
    fn content_type(self, mime: Mime) -> Self {
        self.header("content-type", mime.to_string())
    }
}
