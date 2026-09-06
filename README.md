# head-of-site

Веб-приложение, которое принимает на вход URL и возвращает информацию о теге
`<head>` сайта.

## Стек

- **Python 3.12**, **Django 5.2 (LTS)**
- **django-modern-rest** (REST API: `Controller` + `APIError` + `HTTPStatus`)
- **requests** + **beautifulsoup4** (парсер lxml) — загрузка и разбор HTML
- **Pillow** — определение фактических размеров favicon
- **DaisyUI** (CDN) — интерфейс
- **pytest** + **pytest-django** — тесты
- **ruff** + **pre-commit** — линтинг и форматирование

## Запуск (локально)

```sh
python -m venv .venv && source .venv/bin/activate
pip install -r requirements-dev.txt   # рантайм + ruff, pre-commit, pytest
python manage.py runserver
```

Приложение поднимется на `http://127.0.0.1:8000`.

## Линтинг и форматирование

```sh
ruff check .    # линтер
ruff format .   # форматтер
pre-commit install        # установить git-хуки
pre-commit run --all-files
```

Настройки — в `pyproject.toml` (`[tool.ruff]`), хуки — в `.pre-commit-config.yaml`.

## Docker

```sh
docker build -t head-of-site .
docker run --rm -p 8000:8000 head-of-site
```

## API

### `GET /api/head?url=<url>`

Возвращает всю информацию о теге `<head>` в формате JSON:

```json
{
  "url": "https://example.com/",
  "parsed_url": {
    "scheme": "https",
    "host": "example.com",
    "port": null,
    "path": "/",
    "query": { "key": "value" }
  },
  "title": "Example",
  "description": "An example site",
  "seo": { "canonical": "...", "robots": "...", "keywords": "..." },
  "opengraph": {
    "title": "...", "description": "...", "image": "...",
    "url": "...", "type": "...", "site_name": "..."
  },
  "favicons": [ { "url": "...", "size": "32x32" } ],
  "head": "<head>...</head>"
}
```

- URL без схемы автоматически дополняется до `https://`.
- Редиректы обрабатываются, таймаут запроса — 10 с.
- Отсутствующие поля отдаются как `null` (favicons — `[]`).
- `port` — `null`, если порт не указан явно.

### Ошибки

| Код | Условие                          | Тело               |
|-----|----------------------------------|--------------------|
| 400 | невалидный url / неподдерживаемая схема | `{"error": "..."}` |
| 502 | сайт недоступен (сеть/таймаут)   | `{"error": "..."}` |

## Страницы

- `/` — форма ввода URL (одно поле + кнопка, по центру).
- `/head/?url=<url>` — страница результата: строка с полным URL, блок в
  пропорции 1:3 (расшифровка URL + вкладки «Информация» и «Тег <head>»).
  Вкладки отрендерены заранее и переключаются без htmx (DaisyUI tabs).

## Тесты

```sh
pytest
```
