"""Root URL configuration for the head-of-site project."""

from django.urls import include, path

urlpatterns = [
    path("", include("head.urls")),
]
