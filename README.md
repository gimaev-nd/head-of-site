# head-of-site

Веб-приложение, которое принимает на вход URL и возвращает информацию о теге
`<head>` сайта.

## Стек

- **Rust + Topcoat** (fullstack-фреймворк на базе Axum, experimental)
- Серверный рендеринг через `view!`-макрос, интерфейс на **htmx**
- **Topcoat UI** (feature `ui`), стилизация через Tailwind

## Запуск

Требуется Rust **1.95+** (проект собирается на edition 2024).

```sh
cargo run
```

Приложение поднимется на `http://127.0.0.1:3000` (адрес можно переопределить
через переменные окружения `HOST` и `PORT`).

При первом запуске build-скрипт скачает автономный Tailwind CLI и сгенерирует
стили; итоговый CSS встраивается в бинарник.

## Docker

```sh
docker build -t head-of-site .
docker run --rm -p 3000:3000 head-of-site
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
    "port": 443,
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

### Ошибки

| Код | Условие                      | Тело               |
|-----|------------------------------|--------------------|
| 400 | невалидный url / схема       | `{"error": "..."}` |
| 502 | сайт недоступен (сеть/таймаут) | `{"error": "..."}` |

## Страницы

- `/` — форма ввода URL (одно поле + кнопка, по центру).
- `/head?url=<url>&tab=info|head` — страница результата: строка с полным URL,
  блок в пропорции 1:3 (расшифровка URL + вкладки «Информация» и «Тег <head>»).

## Тесты

```sh
cargo test
```
