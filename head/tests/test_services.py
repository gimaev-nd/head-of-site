"""Unit tests for the parsing/fetching logic (no network, no Django)."""

import io

import pytest
from PIL import Image

from head import services

SAMPLE_HTML = """<!DOCTYPE html>
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
</html>"""


class TestParseHttpUrl:
    def test_parses_valid_http_urls(self):
        assert services.parse_http_url("https://example.com") == "https://example.com"
        assert (
            services.parse_http_url("http://example.com:8080/p")
            == "http://example.com:8080/p"
        )

    def test_adds_https_when_scheme_missing(self):
        assert services.parse_http_url("example.com") == "https://example.com"

    def test_rejects_empty_url(self):
        with pytest.raises(services.AppError) as excinfo:
            services.parse_http_url("   ")
        assert excinfo.value.status_code == 400

    def test_rejects_url_with_spaces(self):
        with pytest.raises(services.AppError) as excinfo:
            services.parse_http_url("not a url")
        assert excinfo.value.status_code == 400

    def test_rejects_unsupported_scheme(self):
        with pytest.raises(services.AppError) as excinfo:
            services.parse_http_url("ftp://example.com")
        assert excinfo.value.status_code == 400


class TestParseUrlParts:
    def test_parses_parts_without_explicit_port(self):
        parts = services.parse_url_parts("https://example.com/some/page?x=1&y=2")
        assert parts["scheme"] == "https"
        assert parts["host"] == "example.com"
        assert parts["port"] is None
        assert parts["path"] == "/some/page"
        assert parts["query"] == {"x": "1", "y": "2"}

    def test_explicit_port_is_kept(self):
        parts = services.parse_url_parts("http://example.com:8080/")
        assert parts["port"] == 8080

    def test_empty_path_becomes_root(self):
        parts = services.parse_url_parts("https://example.com")
        assert parts["path"] == "/"


class TestExtractHead:
    def test_extracts_all_fields(self):
        info = services._extract_head(
            SAMPLE_HTML, "https://example.com/some/page?x=1&y=2"
        )
        assert info["title"] == "Example Site"
        assert info["description"] == "An example site"
        assert info["seo"]["canonical"] == "https://example.com/canonical"
        assert info["seo"]["robots"] == "index, follow"
        assert info["seo"]["keywords"] == "example, test"
        assert info["opengraph"]["title"] == "OG Title"
        assert info["opengraph"]["site_name"] == "Example"
        assert info["opengraph"]["type"] == "website"
        assert len(info["favicons"]) == 2
        assert info["favicons"][0]["url"] == "https://example.com/favicon.ico"
        assert info["favicons"][0]["size"] == "32x32"
        assert info["favicons"][1]["url"] == "https://example.com/apple.png"
        assert info["favicons"][1]["size"] == "180x180"

    def test_missing_fields_are_none(self):
        info = services._extract_head(
            "<html><head></head></html>", "https://example.com/"
        )
        assert info["title"] is None
        assert info["description"] is None
        assert info["seo"]["canonical"] is None
        assert info["favicons"] == []

    def test_formats_head_with_two_space_indent(self):
        info = services._extract_head(SAMPLE_HTML, "https://example.com/")
        head = info["head"]
        assert head.startswith("<head>")
        assert "\n  <title>Example Site</title>" in head
        assert "\n  <meta" in head
        assert head.rstrip().endswith("</head>")


class TestDetectImageSize:
    def test_detects_png_size(self):
        buffer = io.BytesIO()
        Image.new("RGB", (16, 16)).save(buffer, "PNG")
        assert services._detect_image_size(buffer.getvalue()) == "16x16"

    def test_detects_svg_size(self):
        svg = (
            b'<svg xmlns="http://www.w3.org/2000/svg" '
            b'width="48" height="48" viewBox="0 0 48 48"></svg>'
        )
        assert services._detect_image_size(svg) == "48x48"

    def test_returns_none_for_garbage(self):
        assert services._detect_image_size(b"not an image") is None
