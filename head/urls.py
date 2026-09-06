"""URL configuration for the head app."""

from django.urls import path
from dmr.routing import Router
from dmr.routing import path as dmr_path

from . import views

api_router = Router(
    "api/",
    [dmr_path("head", views.HeadController.as_view(), name="head-api")],
)

urlpatterns = [
    path("", views.index, name="index"),
    path("head/", views.result, name="result"),
    api_router.to_urlpatterns(namespace="api"),
]
