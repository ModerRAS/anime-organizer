use axum::response::Html;

const INDEX_HTML: &[u8] = include_bytes!("../../webui/dist/index.html");

pub(crate) async fn index() -> Html<&'static str> {
    Html(std::str::from_utf8(INDEX_HTML).expect("embedded WebUI must be UTF-8"))
}
