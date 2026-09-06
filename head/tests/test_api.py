"""Integration tests for the REST API (Django test client, mocked network)."""

from unittest import mock

import pytest
from django.test import Client

from head import services

SAMPLE_HTML = """<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>Example Site</title>
  <meta name="description" content="An example site">
  <meta name="robots" content="index, follow">
  <link rel="icon" href="/favicon.ico" sizes="32x32">
</head>
<body><h1>Hello</h1></body>
</html>"""


def _fake_response(html: str) -> mock.Mock:
    response = mock.Mock()
    response.headers = {"Content-Type": "text/html; charset=utf-8"}
    response.text = html
    response.encoding = "utf-8"
    response.apparent_encoding = "utf-8"
    response.content = html.encode("utf-8")
    return response


@pytest.fixture
def client() -> Client:
    return Client()


def test_api_head_returns_all_fields(client):
    with mock.patch.object(
        services.requests, "get", return_value=_fake_response(SAMPLE_HTML)
    ):
        response = client.get("/api/head", {"url": "https://example.com"})

    assert response.status_code == 200
    data = response.json()
    assert data["url"] == "https://example.com"
    assert data["parsed_url"]["scheme"] == "https"
    assert data["parsed_url"]["host"] == "example.com"
    assert data["parsed_url"]["port"] is None
    assert data["title"] == "Example Site"
    assert data["description"] == "An example site"
    assert data["seo"]["robots"] == "index, follow"
    assert data["favicons"][0]["url"] == "https://example.com/favicon.ico"
    assert data["favicons"][0]["size"] == "32x32"
    assert data["head"].startswith("<head>")


def test_api_head_missing_url_400(client):
    response = client.get("/api/head")
    assert response.status_code == 400
    assert response.json() == {"error": "invalid url"}


def test_api_head_invalid_url_400(client):
    response = client.get("/api/head", {"url": "not a url"})
    assert response.status_code == 400
    assert response.json() == {"error": "invalid url"}


def test_api_head_unsupported_scheme_400(client):
    response = client.get("/api/head", {"url": "ftp://example.com"})
    assert response.status_code == 400
    assert "error" in response.json()


def test_api_head_unreachable_502(client):
    with mock.patch.object(
        services.requests,
        "get",
        side_effect=services.requests.exceptions.ConnectionError("boom"),
    ):
        response = client.get("/api/head", {"url": "https://example.com"})

    assert response.status_code == 502
    assert "error" in response.json()


def test_html_index_page(client):
    response = client.get("/")
    assert response.status_code == 200
    assert "Анализировать" in response.content.decode()


def test_html_result_page(client):
    with mock.patch.object(
        services.requests, "get", return_value=_fake_response(SAMPLE_HTML)
    ):
        response = client.get("/head/", {"url": "https://example.com"})

    assert response.status_code == 200
    content = response.content.decode()
    assert "Example Site" in content
    assert "Расшифровка URL" in content
