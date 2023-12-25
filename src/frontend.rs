

use axum::{
    body::{Body},
    http::{header, HeaderValue, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use include_dir::{include_dir, Dir};


static FRONTEND_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/frontend/build");
pub async fn frontend(uri: Uri) -> impl IntoResponse {
    // Ugly "no path means index.html" hack
    let path = match uri.path().trim_start_matches("/") {
        "" => "index.html",
        x => x,
    };

    let mime_type = mime_guess::from_path(path).first_or_text_plain();
    let file = FRONTEND_DIR.get_file(path);
    match file {
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap(),
        Some(file) => Response::builder()
            .status(StatusCode::OK)
            .header(
                header::CONTENT_TYPE,
                HeaderValue::from_str(mime_type.as_ref()).unwrap(),
            )
            .body(Body::from(file.contents()))
            .unwrap(),
    }
}
