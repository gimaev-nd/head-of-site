"""ASGI config for the head-of-site project."""

import os

from django.core.asgi import get_asgi_application

os.environ.setdefault("DJANGO_SETTINGS_MODULE", "head_of_site.settings")

application = get_asgi_application()
