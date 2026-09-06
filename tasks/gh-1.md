# Задача
Нужно написать веб-приложение которое будет принимать на вход url, а на выходе давать информацию о теге head сайта.

# Стек
* python 3.12
* django 5.2 (LTS)
* pytest-django
* django modern rest (https://github.com/wemake-services/django-modern-rest) — использовать его Response/HTTPStatus
* requests + beautifulsoup4 (парсер lxml)
* Pillow (определение фактических размеров favicon)
* DaisyUI (UI)

# Интерфейсы
* html на основе Django-шаблонов + DaisyUI (без htmx)
* REST api

# Принцип работы
Приложение берёт url и извлекает тег head.
* Если url без схемы — автоматически добавляется `https://`.
* HTTP-запрос выполняется со следованием редиректам, таймаут 10 с.
Пользователю предоставляются:
* url
* расшифровка url: схема, хост, порт, путь, параметры и их значения
* отформатированное содержимое тега head с отступом 2 пробела
* заголовок (тег <title>)
* описание (meta description)
* seo-данные: canonical, robots, keywords
* opengraph-данные: og:title, og:description, og:image, og:url, og:type, og:site_name
* список favicon (размер, url)

Отсутствующие поля/теги отдаются как `null` (для favicons — пустой список `[]`).

# html
## UI
DaisyUI

## Страницы
### Страница ввода url
Одно поле ввода и кнопка по центру экрана. Обычная форма (GET), переход на страницу результата.

### Страница результата
Отдельная страница (переход по url). Состоит из следующих блоков:
* Строка с полным url
* Блок поделённый на две части в пропорции 1:3
    * Расшифровка url
    * Блок с двумя вкладками (обе отрендерены заранее; переключение — DaisyUI tabs без htmx)
        * Информация из тега head (title, description, seo, opengraph, favicons)
        * Отформатированный тег head (отступ 2 пробела)

# API
API должен отдавать всю информацию перечисленную в "Принцип работы".

## Endpoint
`GET /api/head?url=<url>`

## Ответ
```js
response = {
  url: "...",                      // полный url
  parsed_url: {                    // Расшифровка url
    scheme: "...",                 // схема
    host: "...",                   // хост
    port: 80,                      // порт (null, если не указан явно)
    path: "/",                     // путь
    query: { "key": "value" }      // параметры и их значения
  },
  title: "...",                    // заголовок (тег <title>)
  description: "...",              // описание (meta description)
  seo: {
    canonical: "...",              // <link rel="canonical">
    robots: "...",                 // <meta name="robots">
    keywords: "..."                // <meta name="keywords">
  },
  opengraph: {
    title: "...",                  // og:title
    description: "...",            // og:description
    image: "...",                  // og:image
    url: "...",                    // og:url
    type: "...",                   // og:type
    site_name: "..."               // og:site_name
  },
  favicons: [
    { url: "...", size: "32x32" }  // url и размер favicon
  ],
  head: "<head>...</head>"         // отформатированный тег head, отступ 2 пробела
}
```

## Favicon
* Включаются все `<link>`, где `rel` содержит `icon` или `apple-touch-icon`.
* Размер берётся из атрибута `sizes`; если `sizes` нет — скачать favicon и определить фактические размеры через Pillow. Для `.ico` с несколькими кадрами — максимальный размер.

## Ошибки
* `400` — невалидный url
* `502` — сайт недоступен (таймаут/сеть)

Тело ошибки: `{"error": "..."}`

# Критерии готовности
- [ ] API отдаёт все поля из "Принцип работы"
- [ ] Обработка ошибок: 400 (невалидный url) / 502 (недоступен)
- [ ] Юнит/интеграционные тесты
- [ ] Docker / инструкция по запуску
