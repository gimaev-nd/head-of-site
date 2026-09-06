"""Response and query schemas for the REST API (pydantic)."""

import pydantic


class ParsedUrl(pydantic.BaseModel):
    """The parts of a URL, as the task's "Расшифровка url" block shows them."""

    scheme: str
    host: str
    port: int | None = None
    path: str
    query: dict[str, str] = pydantic.Field(default_factory=dict)


class Seo(pydantic.BaseModel):
    """SEO-related head metadata."""

    canonical: str | None = None
    robots: str | None = None
    keywords: str | None = None


class OpenGraph(pydantic.BaseModel):
    """Open Graph metadata."""

    title: str | None = None
    description: str | None = None
    image: str | None = None
    url: str | None = None
    type: str | None = None
    site_name: str | None = None


class Favicon(pydantic.BaseModel):
    """A single favicon entry."""

    url: str
    size: str | None = None


class HeadInfo(pydantic.BaseModel):
    """Everything the API reports about a page's ``<head>`` tag."""

    url: str
    parsed_url: ParsedUrl
    title: str | None = None
    description: str | None = None
    seo: Seo
    opengraph: OpenGraph
    favicons: list[Favicon] = pydantic.Field(default_factory=list)
    head: str


class HeadQuery(pydantic.BaseModel):
    """Query parameters for ``GET /api/head``."""

    url: str | None = None


class ErrorResponse(pydantic.BaseModel):
    """Error body for ``400`` / ``502`` responses."""

    error: str
