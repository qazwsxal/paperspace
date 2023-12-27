use axum::{
    body::Body,
    http::{header, HeaderValue, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use include_dir::{include_dir, Dir};

static FRONTEND_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/frontend/build");

// Hack to turn /$uri into checks for /$uri, /$uri.html, /$uri/index.html
static FORMAT_SUFFIXES: &[&str] = &["", ".html", "/index.html"];
pub async fn frontend(uri: Uri) -> impl IntoResponse {
    let path = uri.path().strip_prefix("/").unwrap();
    for suffix in FORMAT_SUFFIXES.iter() {
        let newpath = format!("{}{}", path, suffix);
        let file = FRONTEND_DIR.get_file(&newpath);
        match file {
            None => continue,
            Some(file) => {
                let mime_type = mime_guess::from_path(newpath).first_or_text_plain();
                return Response::builder()
                    .status(StatusCode::OK)
                    .header(
                        header::CONTENT_TYPE,
                        HeaderValue::from_str(mime_type.as_ref()).unwrap(),
                    )
                    .body(Body::from(file.contents()))
                    .unwrap();
            }
        }
    }
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty())
        .unwrap()
}
