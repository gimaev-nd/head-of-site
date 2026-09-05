use topcoat::{
    Result,
    context::Cx,
    htmx::hx_request,
    router::{
        Body, HeaderValue, StatusCode, content::Json, header, layout, page, parse_query_params,
        response::{IntoResponse, Response}, route,
    },
    view::{attributes, view},
};

use crate::components::{
    badge::{BadgeVariant, badge},
    button::button,
    card::{card, card_content, card_description, card_header, card_title},
    input::input,
    tabs::{tabs, tabs_content, tabs_list, tabs_trigger},
};
use crate::fetch::fetch_and_parse;

const HTMX_SRC: &str = "https://cdn.jsdelivr.net/npm/htmx.org@2.0.10/dist/htmx.min.js";

// -- Layout ---------------------------------------------------------------

#[layout("/")]
async fn root(cx: &Cx, slot: Result) -> Result {
    view! {
        if hx_request(cx) {
            (slot?)
        } else {
            <!DOCTYPE html>
            <html lang="ru">
                <head>
                    <meta charset="utf-8">
                    <meta name="viewport" content="width=device-width, initial-scale=1">
                    <title>"Head of Site"</title>
                    <link rel="stylesheet" href="/styles.css">
                    <script src=(HTMX_SRC)></script>
                </head>
                <body>(slot?)</body>
            </html>
        }
    }
}

// -- Pages ----------------------------------------------------------------

#[page("/")]
async fn home() -> Result {
    view! {
        <main class="flex min-h-screen items-center justify-center px-4">
            <div class="w-full max-w-xl">
                <h1 class="mb-2 text-center text-2xl font-semibold">"Head of Site"</h1>
                <p class="mb-6 text-center text-sm text-muted-foreground">
                    "Введите URL, чтобы посмотреть содержимое его тега <head>."
                </p>
                <form hx-get="/head" hx-target="#result" hx-swap="innerHTML" hx-push-url="true">
                    <div class="flex gap-2">
                        input(
                            attrs: attributes! {
                                type="url" name="url" placeholder="https://example.com"
                                required="" class="flex-1"
                            }
                        )
                        button(attrs: attributes! { type="submit" }, "Анализировать")
                    </div>
                </form>
                <div id="result" class="mt-8"></div>
            </div>
        </main>
    }
}

#[derive(serde::Deserialize)]
struct HeadQuery {
    url: Option<String>,
    tab: Option<String>,
}

#[page("/head")]
async fn head_page(cx: &Cx) -> Result {
    let query = parse_query_params::<HeadQuery>(cx)?;
    let url = query.url.clone().filter(|s| !s.trim().is_empty());
    let tab: &'static str = match query.tab.as_deref() {
        Some("head") => "head",
        _ => "info",
    };

    match url {
        None => view! {
            <div class="rounded-lg border border-border bg-background p-6 text-center">
                <p class="text-sm font-medium text-destructive">"Введите URL для анализа."</p>
            </div>
        },
        Some(url) => match fetch_and_parse(&url).await {
            Ok(info) => {
                let url = info.url.clone();
                let scheme = info.parsed_url.scheme.clone();
                let host = info.parsed_url.host.clone();
                let port = info
                    .parsed_url
                    .port
                    .map(|p| p.to_string())
                    .unwrap_or_else(|| "—".to_string());
                let path = info.parsed_url.path.clone();
                let query_pairs: Vec<(String, String)> = info
                    .parsed_url
                    .query
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                let seo_pairs: Vec<(String, String)> = vec![
                    ("canonical".to_string(), show(&info.seo.canonical)),
                    ("robots".to_string(), show(&info.seo.robots)),
                    ("keywords".to_string(), show(&info.seo.keywords)),
                ];
                let og_pairs: Vec<(String, String)> = vec![
                    ("title".to_string(), show(&info.opengraph.title)),
                    ("description".to_string(), show(&info.opengraph.description)),
                    ("image".to_string(), show(&info.opengraph.image)),
                    ("url".to_string(), show(&info.opengraph.url)),
                    ("type".to_string(), show(&info.opengraph.og_type)),
                    ("site_name".to_string(), show(&info.opengraph.site_name)),
                ];
                let favicons = info.favicons.clone();
                let title = show(&info.title);
                let description = show(&info.description);
                let head = info.head.clone();
                let info_href = tab_href(&url, "info");
                let head_href = tab_href(&url, "head");

                view! {
                    <div class="space-y-4">
                        <a href="/" class="inline-flex items-center text-sm text-muted-foreground hover:text-foreground">
                            "← Новый поиск"
                        </a>
                        <p class="break-all text-sm font-medium">(url)</p>
                        <div class="grid grid-cols-4 gap-4">
                            <div class="col-span-1">
                                card(
                                    card_header(
                                        card_title("Расшифровка URL")
                                        card_description("Схема, хост, порт, путь и параметры.")
                                    )
                                    card_content(
                                        <dl class="flex flex-col gap-2 text-sm">
                                            <div class="flex items-start justify-between gap-3">
                                                <dt class="shrink-0 text-xs text-muted-foreground">"scheme"</dt>
                                                <dd class="break-all font-medium">(scheme)</dd>
                                            </div>
                                            <div class="flex items-start justify-between gap-3">
                                                <dt class="shrink-0 text-xs text-muted-foreground">"host"</dt>
                                                <dd class="break-all font-medium">(host)</dd>
                                            </div>
                                            <div class="flex items-start justify-between gap-3">
                                                <dt class="shrink-0 text-xs text-muted-foreground">"port"</dt>
                                                <dd class="font-medium">(port)</dd>
                                            </div>
                                            <div class="flex items-start justify-between gap-3">
                                                <dt class="shrink-0 text-xs text-muted-foreground">"path"</dt>
                                                <dd class="break-all font-medium">(path)</dd>
                                            </div>
                                            <div class="flex flex-col gap-1 border-t border-border pt-2">
                                                <dt class="text-xs font-medium uppercase tracking-wide text-muted-foreground">"query"</dt>
                                                if query_pairs.is_empty() {
                                                    <dd class="text-sm text-muted-foreground">"—"</dd>
                                                } else {
                                                    for (k, v) in query_pairs {
                                                        <div class="flex items-start justify-between gap-3">
                                                            <span class="shrink-0 text-xs text-muted-foreground">(k)</span>
                                                            <span class="break-all text-right text-sm">(v)</span>
                                                        </div>
                                                    }
                                                }
                                            </div>
                                        </dl>
                                    )
                                )
                            </div>
                            <div class="col-span-3">
                                card(
                                    card_content(
                                        tabs(
                                            tabs_list(
                                                tabs_trigger(
                                                    active: tab == "info",
                                                    attrs: attributes! { href=(info_href) },
                                                    "Информация"
                                                )
                                                tabs_trigger(
                                                    active: tab == "head",
                                                    attrs: attributes! { href=(head_href) },
                                                    "Тег <head>"
                                                )
                                            )
                                            tabs_content(
                                                if tab == "info" {
                                                    <div class="flex flex-col gap-5">
                                                        <div class="flex flex-col gap-1">
                                                            <p class="text-xs font-medium uppercase tracking-wide text-muted-foreground">"Заголовок (title)"</p>
                                                            <p class="text-sm">(title)</p>
                                                        </div>
                                                        <div class="flex flex-col gap-1">
                                                            <p class="text-xs font-medium uppercase tracking-wide text-muted-foreground">"Описание (meta description)"</p>
                                                            <p class="text-sm">(description)</p>
                                                        </div>
                                                        <div class="flex flex-col gap-2">
                                                            <p class="text-xs font-medium uppercase tracking-wide text-muted-foreground">"SEO"</p>
                                                            for (k, v) in seo_pairs {
                                                                <div class="flex items-start justify-between gap-3 border-b border-border py-1.5">
                                                                    <span class="shrink-0 text-xs text-muted-foreground">(k)</span>
                                                                    <span class="break-all text-right text-sm">(v)</span>
                                                                </div>
                                                            }
                                                        </div>
                                                        <div class="flex flex-col gap-2">
                                                            <p class="text-xs font-medium uppercase tracking-wide text-muted-foreground">"Open Graph"</p>
                                                            for (k, v) in og_pairs {
                                                                <div class="flex items-start justify-between gap-3 border-b border-border py-1.5">
                                                                    <span class="shrink-0 text-xs text-muted-foreground">(k)</span>
                                                                    <span class="break-all text-right text-sm">(v)</span>
                                                                </div>
                                                            }
                                                        </div>
                                                        <div class="flex flex-col gap-2">
                                                            <p class="text-xs font-medium uppercase tracking-wide text-muted-foreground">"Favicon"</p>
                                                            if favicons.is_empty() {
                                                                <p class="text-sm text-muted-foreground">"Не найдено"</p>
                                                            } else {
                                                                for favicon in favicons {
                                                                    <div class="flex items-start justify-between gap-3 border-b border-border py-1.5">
                                                                        <span class="break-all text-sm">(favicon.url)</span>
                                                                        badge(
                                                                            variant: BadgeVariant::Secondary,
                                                                            (favicon.size.unwrap_or_else(|| "—".to_string()))
                                                                        )
                                                                    </div>
                                                                }
                                                            }
                                                        </div>
                                                    </div>
                                                } else {
                                                    <pre class="overflow-x-auto rounded-md border border-border bg-background p-4 text-xs leading-relaxed">
                                                        <code>(head)</code>
                                                    </pre>
                                                }
                                            )
                                        )
                                    )
                                )
                            </div>
                        </div>
                    </div>
                }
            }
            Err(error) => {
                let message = error.message();
                view! {
                    <div class="rounded-lg border border-border bg-background p-6 text-center">
                        <p class="text-sm font-medium text-destructive">(message)</p>
                    </div>
                }
            }
        },
    }
}

// -- API ------------------------------------------------------------------

#[derive(serde::Deserialize)]
struct ApiQuery {
    url: Option<String>,
}

#[route(GET "/api/head")]
async fn api_head(cx: &Cx) -> Result<Response> {
    let query = parse_query_params::<ApiQuery>(cx)?;
    let Some(url) = query.url.as_deref().filter(|s| !s.trim().is_empty()) else {
        return error_response(cx, 400, "invalid url");
    };

    match fetch_and_parse(url).await {
        Ok(info) => Json(info).into_response(cx),
        Err(error) => error_response(cx, error.status(), &error.message()),
    }
}

#[route(GET "/styles.css")]
async fn styles() -> Result<Response> {
    let css = include_str!("../static/styles.css");
    let mut response = Response::new(Body::from(css));
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static("text/css"));
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=3600"),
    );
    Ok(response)
}

// -- Helpers --------------------------------------------------------------

fn error_response(cx: &Cx, status: u16, message: &str) -> Result<Response> {
    let status = StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let body = Json(serde_json::json!({ "error": message }));
    (status, body).into_response(cx)
}

fn show(value: &Option<String>) -> String {
    value
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| "—".to_string())
}

fn tab_href(url: &str, tab: &str) -> String {
    let mut serializer = url::form_urlencoded::Serializer::new(String::new());
    serializer.append_pair("url", url);
    serializer.append_pair("tab", tab);
    format!("/head?{}", serializer.finish())
}
