//! Serves the embedded Svelte dashboard as static files with SPA fallback.

use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "dashboard-dist/"]
struct DashboardAssets;

pub async fn serve_dashboard(uri: axum::http::Uri) -> Response {
    let path = uri.path().trim_start_matches('/');

    // Try exact file match first.
    if let Some(file) = DashboardAssets::get(path) {
        return file_response(path, &file);
    }

    // SPA fallback: serve index.html for unmatched routes.
    match DashboardAssets::get("index.html") {
        Some(file) => file_response("index.html", &file),
        None => (StatusCode::NOT_FOUND, "Dashboard not available").into_response(),
    }
}

fn file_response(path: &str, file: &rust_embed::EmbeddedFile) -> Response {
    let mime = mime_guess::from_path(path).first_or_octet_stream();

    let cache = if path.contains("_app/immutable") {
        "public, max-age=31536000, immutable"
    } else if path == "index.html" {
        "no-cache"
    } else {
        "public, max-age=3600"
    };

    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, mime.as_ref()),
            (header::CACHE_CONTROL, cache),
        ],
        file.data.clone(),
    )
        .into_response()
}
