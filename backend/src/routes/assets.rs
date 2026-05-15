use axum::{
    body::Body,
    http::{Response, StatusCode, Uri, header},
    response::IntoResponse,
};
use rust_embed::RustEmbed;

// Path is relative to CARGO_MANIFEST_DIR (backend/), resolved to an absolute
// path at compile time. In debug builds rust-embed reads from disk at that
// absolute path; in release it embeds the files into the binary. Either way
// the current working directory of the running process is irrelevant.
#[derive(RustEmbed)]
#[folder = "../frontend/dist"]
struct AdminUi;

pub async fn handler(uri: Uri) -> Response<Body> {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    serve(path).unwrap_or_else(|| serve("index.html").unwrap_or_else(not_found))
}

fn serve(path: &str) -> Option<Response<Body>> {
    let asset = AdminUi::get(path)?;
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    Some(([(header::CONTENT_TYPE, mime.as_ref())], asset.data).into_response())
}

fn not_found() -> Response<Body> {
    StatusCode::NOT_FOUND.into_response()
}
