use std::collections::BTreeMap;

use scraper::{Html, Selector, node::Node};
use serde::Serialize;
use url::Url;

/// The parts of a URL, as the task's "Расшифровка url" block shows them.
#[derive(Debug, Clone, Serialize)]
pub struct ParsedUrl {
    pub scheme: String,
    pub host: String,
    pub port: Option<u16>,
    pub path: String,
    pub query: BTreeMap<String, String>,
}

impl ParsedUrl {
    pub fn from_url(url: &Url) -> Self {
        let mut query = BTreeMap::new();
        for (k, v) in url.query_pairs() {
            query.entry(k.into_owned()).or_insert(v.into_owned());
        }
        ParsedUrl {
            scheme: url.scheme().to_string(),
            host: url.host_str().unwrap_or_default().to_string(),
            port: url.port_or_known_default(),
            path: url.path().to_string(),
            query,
        }
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Seo {
    pub canonical: Option<String>,
    pub robots: Option<String>,
    pub keywords: Option<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct OpenGraph {
    pub title: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
    pub url: Option<String>,
    #[serde(rename = "type")]
    pub og_type: Option<String>,
    pub site_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Favicon {
    pub url: String,
    pub size: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HeadInfo {
    pub url: String,
    pub parsed_url: ParsedUrl,
    pub title: Option<String>,
    pub description: Option<String>,
    pub seo: Seo,
    pub opengraph: OpenGraph,
    pub favicons: Vec<Favicon>,
    pub head: String,
}

/// An application error, mapped onto the HTTP status codes the task specifies
/// (400 for a bad URL, 502 for a site that cannot be reached).
#[derive(Debug, Clone)]
pub enum AppError {
    InvalidUrl,
    UnsupportedScheme(String),
    FetchFailed(String),
}

impl AppError {
    pub fn status(&self) -> u16 {
        match self {
            Self::InvalidUrl | Self::UnsupportedScheme(_) => 400,
            Self::FetchFailed(_) => 502,
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::InvalidUrl => "invalid url".to_string(),
            Self::UnsupportedScheme(scheme) => format!("unsupported url scheme: {scheme}"),
            Self::FetchFailed(detail) => format!("failed to fetch url: {detail}"),
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message())
    }
}

impl std::error::Error for AppError {}

/// Parses a raw user-supplied string into an absolute HTTP(S) URL.
pub fn parse_http_url(raw: &str) -> Result<Url, AppError> {
    let trimmed = raw.trim();
    let url = Url::parse(trimmed).map_err(|_| AppError::InvalidUrl)?;
    match url.scheme() {
        "http" | "https" => Ok(url),
        scheme => Err(AppError::UnsupportedScheme(scheme.to_string())),
    }
}

fn select_one<'a>(document: &'a Html, selector: &str) -> Option<scraper::ElementRef<'a>> {
    let selector = Selector::parse(selector).ok()?;
    document.select(&selector).next()
}

fn first_text(document: &Html, selector: &str) -> Option<String> {
    select_one(document, selector).map(|el| el.text().collect::<String>())
}

fn first_attr(document: &Html, selector: &str, attr: &str) -> Option<String> {
    select_one(document, selector).and_then(|el| el.attr(attr).map(str::to_string))
}

/// Reads a meta tag's `content`, checking both `property` and `name` keys.
fn meta_content(document: &Html, names: &[&str]) -> Option<String> {
    for name in names {
        let property = format!("head meta[property=\"{name}\"]");
        if let Some(v) = first_attr(document, &property, "content") {
            return Some(v);
        }
        let name_attr = format!("head meta[name=\"{name}\"]");
        if let Some(v) = first_attr(document, &name_attr, "content") {
            return Some(v);
        }
    }
    None
}

fn extract_favicons(document: &Html, page_url: &Url) -> Vec<Favicon> {
    let Ok(selector) =
        Selector::parse("head link[rel~=\"icon\"], head link[rel~=\"apple-touch-icon\"]")
    else {
        return Vec::new();
    };

    let mut favicons = Vec::new();
    for el in document.select(&selector) {
        let Some(href) = el.attr("href") else {
            continue;
        };
        let url = page_url
            .join(href)
            .map(|u| u.to_string())
            .unwrap_or_else(|_| href.to_string());
        favicons.push(Favicon {
            url,
            size: el.attr("sizes").map(str::to_string),
        });
    }
    favicons
}

/// Extracts everything the task asks for from a page's HTML.
pub fn extract_head(html: &str, page_url: &Url) -> HeadInfo {
    let document = Html::parse_document(html);

    HeadInfo {
        url: page_url.to_string(),
        parsed_url: ParsedUrl::from_url(page_url),
        title: first_text(&document, "head title"),
        description: meta_content(&document, &["description"]),
        seo: Seo {
            canonical: first_attr(&document, "head link[rel~=\"canonical\"]", "href"),
            robots: meta_content(&document, &["robots"]),
            keywords: meta_content(&document, &["keywords"]),
        },
        opengraph: OpenGraph {
            title: meta_content(&document, &["og:title"]),
            description: meta_content(&document, &["og:description"]),
            image: meta_content(&document, &["og:image"]),
            url: meta_content(&document, &["og:url"]),
            og_type: meta_content(&document, &["og:type"]),
            site_name: meta_content(&document, &["og:site_name"]),
        },
        favicons: extract_favicons(&document, page_url),
        head: format_head(&document),
    }
}

// -- Formatted <head> serialization (2-space indent) --

const VOID: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

fn format_head(document: &Html) -> String {
    let Some(head) = select_one(document, "head") else {
        return String::new();
    };
    format_element(&head, 0)
}

fn format_element(el: &scraper::ElementRef, indent: usize) -> String {
    let name = el.value().name();
    let pad = "  ".repeat(indent);
    let mut out = format!("{pad}<{name}");
    for (key, value) in el.value().attrs() {
        out.push_str(&format!(" {key}=\"{value}\""));
    }

    if VOID.contains(&name) {
        out.push('>');
        return out;
    }

    out.push('>');

    let children: Vec<_> = el.children().collect();
    let element_children: Vec<_> = el.child_elements().collect();

    if element_children.is_empty() {
        let text: String = children
            .iter()
            .filter_map(|child| child.value().as_text())
            .map(|t| &**t)
            .collect();
        out.push_str(&text);
        out.push_str(&format!("</{name}>"));
    } else {
        out.push('\n');
        for child in &children {
            match child.value() {
                Node::Element(_) => {
                    let child_el = scraper::ElementRef::wrap(*child).unwrap();
                    out.push_str(&format_element(&child_el, indent + 1));
                    out.push('\n');
                }
                Node::Text(t) => {
                    let s = t.trim();
                    if !s.is_empty() {
                        out.push_str(&format!("{}{s}\n", "  ".repeat(indent + 1)));
                    }
                }
                Node::Comment(c) => {
                    out.push_str(&format!("{}<!--{}-->\n", "  ".repeat(indent + 1), &**c));
                }
                _ => {}
            }
        }
        out.push_str(&format!("{pad}</{name}>"));
    }

    out
}

// -- Favicon size detection from raw image bytes --

/// Detects the pixel dimensions of an image from its header bytes.
pub fn detect_image_size(bytes: &[u8]) -> Option<(u32, u32)> {
    // PNG: signature (8 bytes) + IHDR, width/height are big-endian at 16..24.
    if bytes.len() >= 24 && bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        let w = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let h = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
        return Some((w, h));
    }

    // GIF: little-endian width/height at bytes 6..10.
    if bytes.len() >= 10 && (bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a")) {
        let w = u16::from_le_bytes([bytes[6], bytes[7]]) as u32;
        let h = u16::from_le_bytes([bytes[8], bytes[9]]) as u32;
        return Some((w, h));
    }

    // ICO: 6-byte header, then 16-byte directory entries (0 means 256).
    if bytes.len() >= 22 && bytes[0] == 0 && bytes[1] == 0 && bytes[2] == 1 && bytes[3] == 0 {
        let w = bytes[6] as u32;
        let h = bytes[7] as u32;
        return Some((if w == 0 { 256 } else { w }, if h == 0 { 256 } else { h }));
    }

    // BMP: little-endian width/height in the BITMAPINFOHEADER.
    if bytes.len() >= 26 && bytes.starts_with(b"BM") {
        let w = i32::from_le_bytes([bytes[18], bytes[19], bytes[20], bytes[21]]) as u32;
        let h = i32::from_le_bytes([bytes[22], bytes[23], bytes[24], bytes[25]]);
        return Some((w, h.unsigned_abs()));
    }

    // SVG: read width/height (with optional px) or the viewBox.
    if let Ok(text) = std::str::from_utf8(bytes) {
        if text.contains("<svg") {
            return parse_svg_size(text);
        }
    }

    None
}

fn parse_svg_size(text: &str) -> Option<(u32, u32)> {
    if let (Some(w), Some(h)) = (extract_attr(text, "width"), extract_attr(text, "height")) {
        if let (Ok(w), Ok(h)) = (parse_px(&w), parse_px(&h)) {
            return Some((w, h));
        }
    }
    if let Some(view_box) = extract_attr(text, "viewBox") {
        let parts: Vec<&str> = view_box.split_whitespace().collect();
        if parts.len() == 4 {
            if let (Ok(w), Ok(h)) = (parse_px(parts[2]), parse_px(parts[3])) {
                return Some((w, h));
            }
        }
    }
    None
}

fn extract_attr(text: &str, name: &str) -> Option<String> {
    let needle = format!("{name}=");
    let idx = text.find(&needle)?;
    let after = &text[idx + needle.len()..];
    let quote = after.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let inner = &after[1..];
    let close = inner.find(quote)?;
    Some(inner[..close].to_string())
}

fn parse_px(s: &str) -> Result<u32, ()> {
    let s = s.trim().trim_end_matches("px").trim();
    if s.is_empty() {
        return Err(());
    }
    s.parse::<f64>().map(|v| v as u32).map_err(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_HTML: &str = r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>Example Site</title>
  <meta name="description" content="An example site">
  <meta name="robots" content="index, follow">
  <meta name="keywords" content="example, test">
  <link rel="canonical" href="https://example.com/canonical">
  <meta property="og:title" content="OG Title">
  <meta property="og:description" content="OG Description">
  <meta property="og:image" content="https://example.com/og.png">
  <meta property="og:url" content="https://example.com/">
  <meta property="og:type" content="website">
  <meta property="og:site_name" content="Example">
  <link rel="icon" href="/favicon.ico" sizes="32x32">
  <link rel="apple-touch-icon" href="/apple.png" sizes="180x180">
  <link rel="stylesheet" href="/style.css">
</head>
<body><h1>Hello</h1></body>
</html>"#;

    fn page_url() -> Url {
        Url::parse("https://example.com/some/page?x=1&y=2").unwrap()
    }

    #[test]
    fn parses_valid_http_urls() {
        assert_eq!(
            parse_http_url("https://example.com").unwrap().scheme(),
            "https"
        );
        assert_eq!(
            parse_http_url("http://example.com:8080/p").unwrap().port(),
            Some(8080)
        );
    }

    #[test]
    fn rejects_invalid_and_unsupported_urls() {
        assert!(matches!(parse_http_url("not a url"), Err(AppError::InvalidUrl)));
        assert!(matches!(parse_http_url("example.com"), Err(AppError::InvalidUrl)));
        assert!(matches!(
            parse_http_url("ftp://example.com"),
            Err(AppError::UnsupportedScheme(_))
        ));
    }

    #[test]
    fn parses_url_parts() {
        let parsed = ParsedUrl::from_url(&page_url());
        assert_eq!(parsed.scheme, "https");
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.port, Some(443));
        assert_eq!(parsed.path, "/some/page");
        assert_eq!(parsed.query.get("x").map(String::as_str), Some("1"));
        assert_eq!(parsed.query.get("y").map(String::as_str), Some("2"));
    }

    #[test]
    fn extracts_head_information() {
        let info = extract_head(SAMPLE_HTML, &page_url());
        assert_eq!(info.title.as_deref(), Some("Example Site"));
        assert_eq!(info.description.as_deref(), Some("An example site"));
        assert_eq!(
            info.seo.canonical.as_deref(),
            Some("https://example.com/canonical")
        );
        assert_eq!(info.seo.robots.as_deref(), Some("index, follow"));
        assert_eq!(info.seo.keywords.as_deref(), Some("example, test"));
        assert_eq!(info.opengraph.title.as_deref(), Some("OG Title"));
        assert_eq!(info.opengraph.site_name.as_deref(), Some("Example"));
        assert_eq!(info.opengraph.og_type.as_deref(), Some("website"));
        assert_eq!(info.favicons.len(), 2);
        assert_eq!(info.favicons[0].url, "https://example.com/favicon.ico");
        assert_eq!(info.favicons[0].size.as_deref(), Some("32x32"));
        assert_eq!(info.favicons[1].url, "https://example.com/apple.png");
        assert_eq!(info.favicons[1].size.as_deref(), Some("180x180"));
    }

    #[test]
    fn formats_head_with_two_space_indent() {
        let info = extract_head(SAMPLE_HTML, &page_url());
        let head = info.head;
        assert!(head.starts_with("<head>"));
        assert!(head.contains("\n  <title>Example Site</title>"));
        assert!(head.contains("\n  <meta"));
        assert!(head.trim_end().ends_with("</head>"));
    }

    #[test]
    fn detects_png_ico_and_svg_sizes() {
        let png = [
            0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, b'I', b'H',
            b'D', b'R', 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x10, 0x08, 0x06, 0x00, 0x00,
            0x00,
        ];
        assert_eq!(detect_image_size(&png), Some((16, 16)));

        let ico = [
            0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 32, 32, 0x00, 0x00, 0x01, 0x00, 0x20, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x16, 0x00, 0x00, 0x00,
        ];
        assert_eq!(detect_image_size(&ico), Some((32, 32)));

        let svg =
            br#"<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 48 48"></svg>"#;
        assert_eq!(detect_image_size(svg), Some((48, 48)));
    }

    #[test]
    fn error_status_codes_match_spec() {
        assert_eq!(AppError::InvalidUrl.status(), 400);
        assert_eq!(AppError::UnsupportedScheme("ftp".into()).status(), 400);
        assert_eq!(AppError::FetchFailed("boom".into()).status(), 502);
    }
}
