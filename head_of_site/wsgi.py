"""WSGI config for the head-of-site project."""

import os

from django.core.wsgi import get_wsgi_application

os.environ.setdefault("DJANGO_SETTINGS_MODULE", "head_of_site.settings")

application = get_wsgi_application()
