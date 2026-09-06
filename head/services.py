"""Fetching and parsing of a page's ``<head>`` tag."""

import re
from io import BytesIO
from urllib.parse import parse_qsl, urljoin, urlsplit

import requests
from bs4 import BeautifulSoup, Comment, NavigableString, Tag
from PIL import Image

FETCH_TIMEOUT = 10
FAVICON_TIMEOUT = 5
USER_AGENT = "head-of-site/0.1.0"

VOID_ELEMENTS = {
    "area",
    "base",
    "br",
    "col",
    "embed",
    "hr",
    "img",
    "input",
    "link",
    "meta",
    "param",
    "source",
    "track",
    "wbr",
}


class AppError(Exception):
    """Application error that maps onto an HTTP status code."""

    def __init__(self, status_code: int, message: str) -> None:
        self.status_code = status_code
        self.message = message
        super().__init__(message)


def parse_http_url(raw: str) -> str:
    """Normalize a user-supplied string into an absolute HTTP(S) URL."""
    trimmed = raw.strip()
    if not trimmed:
        raise AppError(400, "invalid url")
    if any(ch.isspace() for ch in trimmed):
        raise AppError(400, "invalid url")

    candidate = trimmed if "://" in trimmed else f"https://{trimmed}"
    parsed = urlsplit(candidate)
    if parsed.scheme not in ("http", "https"):
        raise AppError(400, f"unsupported url scheme: {parsed.scheme}")
    if not parsed.hostname:
        raise AppError(400, "invalid url")
    return candidate


def parse_url_parts(url: str) -> dict:
    """Break a URL into scheme / host / port / path / query."""
    parsed = urlsplit(url)
    query: dict[str, str] = {}
    for key, value in parse_qsl(parsed.query, keep_blank_values=True):
        if key not in query:
            query[key] = value
    return {
        "scheme": parsed.scheme,
        "host": parsed.hostname or "",
        "port": parsed.port,
        "path": parsed.path or "/",
        "query": query,
    }


def _meta_content(head: Tag | None, names: list[str]) -> str | None:
    if head is None:
        return None
    for name in names:
        for tag in head.find_all("meta", property=name):
            content = tag.get("content")
            if content is not None:
                return content
        for tag in head.find_all("meta", attrs={"name": name}):
            content = tag.get("content")
            if content is not None:
                return content
    return None


def _canonical(head: Tag | None) -> str | None:
    if head is None:
        return None
    link = head.find("link", rel=lambda rel: bool(rel) and "canonical" in rel)
    return link.get("href") if link else None


def _extract_favicons(head: Tag | None, page_url: str) -> list[dict]:
    favicons: list[dict] = []
    if head is None:
        return favicons
    for link in head.find_all("link"):
        rel = link.get("rel") or []
        if isinstance(rel, str):
            rel = [rel]
        tokens = [token.lower() for token in rel]
        if "icon" not in tokens and "apple-touch-icon" not in tokens:
            continue
        href = link.get("href")
        if not href:
            continue
        favicons.append(
            {
                "url": urljoin(page_url, href),
                "size": link.get("sizes"),
            },
        )
    return favicons


def _attr_value(value: object) -> str:
    if isinstance(value, (list, tuple)):
        return " ".join(str(item) for item in value)
    return str(value)


def _format_element(tag: Tag, indent: int) -> str:
    pad = "  " * indent
    name = tag.name
    attrs = "".join(
        f' {key}="{_attr_value(value)}"' for key, value in tag.attrs.items()
    )
    out = f"{pad}<{name}{attrs}"
    if name in VOID_ELEMENTS:
        return out + ">"

    out += ">"
    element_children = [child for child in tag.children if isinstance(child, Tag)]
    if not element_children:
        text = "".join(
            str(child) for child in tag.children if isinstance(child, NavigableString)
        )
        return out + text + f"</{name}>"

    out += "\n"
    for child in tag.children:
        if isinstance(child, Tag):
            out += _format_element(child, indent + 1) + "\n"
        elif isinstance(child, NavigableString):
            text = str(child).strip()
            if text:
                out += "  " * (indent + 1) + text + "\n"
        elif isinstance(child, Comment):
            out += "  " * (indent + 1) + f"<!--{child}-->" + "\n"
    return out + f"{pad}</{name}>"


def _format_head(head: Tag | None) -> str:
    if head is None:
        return ""
    return _format_element(head, 0)


def _parse_dimension(value: str) -> int | None:
    value = value.strip().removesuffix("px").strip()
    try:
        return int(float(value))
    except ValueError:
        return None


def _detect_svg_size(data: bytes) -> str | None:
    try:
        text = data.decode("utf-8", errors="ignore")
    except Exception:
        return None
    if "<svg" not in text:
        return None

    match_width = re.search(r'width=["\']([^"\']+)["\']', text)
    match_height = re.search(r'height=["\']([^"\']+)["\']', text)
    if match_width and match_height:
        width = _parse_dimension(match_width.group(1))
        height = _parse_dimension(match_height.group(1))
        if width is not None and height is not None:
            return f"{width}x{height}"

    match_viewbox = re.search(r'viewBox=["\']([^"\']+)["\']', text)
    if match_viewbox:
        parts = match_viewbox.group(1).split()
        if len(parts) == 4:
            width = _parse_dimension(parts[2])
            height = _parse_dimension(parts[3])
            if width is not None and height is not None:
                return f"{width}x{height}"
    return None


def _detect_image_size(data: bytes) -> str | None:
    try:
        with Image.open(BytesIO(data)) as image:
            width, height = image.size
        return f"{width}x{height}"
    except Exception:
        return _detect_svg_size(data)


def _fill_favicon_sizes(favicons: list[dict]) -> None:
    for favicon in favicons:
        if favicon["size"]:
            continue
        if not favicon["url"].startswith(("http://", "https://")):
            continue
        try:
            response = requests.get(
                favicon["url"],
                timeout=FAVICON_TIMEOUT,
                headers={"User-Agent": USER_AGENT},
            )
            response.raise_for_status()
        except requests.RequestException:
            continue
        size = _detect_image_size(response.content)
        if size:
            favicon["size"] = size


def _extract_head(html: str, page_url: str) -> dict:
    soup = BeautifulSoup(html, "lxml")
    head = soup.find("head")

    title = None
    if head is not None:
        title_tag = head.find("title")
        if title_tag is not None:
            title = title_tag.get_text(strip=True) or None

    return {
        "url": page_url,
        "parsed_url": parse_url_parts(page_url),
        "title": title,
        "description": _meta_content(head, ["description"]),
        "seo": {
            "canonical": _canonical(head),
            "robots": _meta_content(head, ["robots"]),
            "keywords": _meta_content(head, ["keywords"]),
        },
        "opengraph": {
            "title": _meta_content(head, ["og:title"]),
            "description": _meta_content(head, ["og:description"]),
            "image": _meta_content(head, ["og:image"]),
            "url": _meta_content(head, ["og:url"]),
            "type": _meta_content(head, ["og:type"]),
            "site_name": _meta_content(head, ["og:site_name"]),
        },
        "favicons": _extract_favicons(head, page_url),
        "head": _format_head(head),
    }


def fetch_and_parse(raw_url: str) -> dict:
    """Fetch a page and extract everything the task asks for."""
    url = parse_http_url(raw_url)

    try:
        response = requests.get(
            url,
            timeout=FETCH_TIMEOUT,
            headers={"User-Agent": USER_AGENT},
        )
    except requests.exceptions.InvalidURL as exc:
        raise AppError(400, "invalid url") from exc
    except requests.exceptions.Timeout as exc:
        raise AppError(502, "failed to fetch url: timeout") from exc
    except requests.exceptions.TooManyRedirects as exc:
        raise AppError(502, "failed to fetch url: too many redirects") from exc
    except requests.exceptions.ConnectionError as exc:
        raise AppError(502, "failed to fetch url: connection failed") from exc
    except requests.exceptions.RequestException as exc:
        raise AppError(502, "failed to fetch url: request failed") from exc

    content_type = response.headers.get("Content-Type", "")
    if "charset" not in content_type.lower():
        response.encoding = response.apparent_encoding
    html = response.text

    info = _extract_head(html, url)
    _fill_favicon_sizes(info["favicons"])
    return info
