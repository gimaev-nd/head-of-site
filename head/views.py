"""HTML views and the REST API controller."""

from http import HTTPStatus

from django.shortcuts import render
from dmr import APIError, Controller, Query, ResponseSpec, modify
from dmr.plugins.pydantic import PydanticSerializer

from . import services
from .schemas import ErrorResponse, HeadInfo, HeadQuery


def index(request):
    """Input page: a single URL field and a button, centered."""
    return render(request, "index.html")


def result(request):
    """Result page: full URL + 1:3 block (parsed URL | tabbed head info)."""
    url = request.GET.get("url", "").strip()
    if not url:
        return render(
            request,
            "result.html",
            {"error": "Введите URL для анализа."},
        )

    try:
        info = services.fetch_and_parse(url)
    except services.AppError as exc:
        return render(
            request,
            "result.html",
            {"error": exc.message},
        )

    return render(request, "result.html", {"info": info})


class HeadController(Controller[PydanticSerializer]):
    """``GET /api/head?url=<url>``."""

    @modify(
        extra_responses=[
            ResponseSpec(ErrorResponse, status_code=HTTPStatus.BAD_REQUEST),
            ResponseSpec(ErrorResponse, status_code=HTTPStatus.BAD_GATEWAY),
        ],
    )
    def get(self, parsed_query: Query[HeadQuery]) -> HeadInfo:
        url = (parsed_query.url or "").strip()
        if not url:
            raise APIError(
                {"error": "invalid url"},
                status_code=HTTPStatus.BAD_REQUEST,
            )

        try:
            data = services.fetch_and_parse(url)
        except services.AppError as exc:
            raise APIError(
                {"error": exc.message},
                status_code=HTTPStatus(exc.status_code),
            ) from exc

        return HeadInfo.model_validate(data)
