// use ax_extract::Path;
use axum::{
    body::Body,
    extract::Path as AxPath,
    http::{header, Request, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use pulldown_cmark::{html, Options, Parser};
use std::{fs, path::PathBuf};
use tower_http::services::ServeDir;
use tower::util::ServiceExt;

#[tokio::main]
async fn main() {
    // We create the static file service first
    let static_service = ServeDir::new("./content");

    let app = Router::new()
        .route("/", get(serve_index))
        // This handler now decides: is it a Markdown page or a static asset?
        .route("/{*path}", get(handle_request))
        .fallback_service(static_service);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Martian Research Hub running on http://localhost:8080");
    axum::serve(listener, app).await.unwrap();
}

async fn serve_index() -> impl IntoResponse {
    render_markdown("index".to_string()).await
}

async fn handle_request(AxPath(path): AxPath<String>, req: Request<Body>) -> Response {
    let path_buf = PathBuf::from(&path);
    
    // 1. If it has an extension (e.g., .pdf, .js, .webp), treat it as a static file
    if path_buf.extension().is_some() {
        // We manually call ServeDir for this specific path
        match ServeDir::new("./content").oneshot(req).await {
            Ok(res) => return res.into_response(),
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }

    // 2. If no extension, try to render it as a Markdown file
    render_markdown(path).await.into_response()
}

async fn render_markdown(name: String) -> impl IntoResponse {
    let mut path = PathBuf::from("./content/");
    path.push(format!("{}.md", name));

    match fs::read_to_string(path) {
        Ok(markdown_input) => {
            let mut options = Options::empty();
            options.insert(Options::ENABLE_TABLES);
            options.insert(Options::ENABLE_FOOTNOTES);
            options.insert(Options::ENABLE_STRIKETHROUGH);
            options.insert(Options::ENABLE_TASKLISTS);
            // gemini inserted this but doesn't work
            // options.insert(Options::ENABLE_MATHJAX); // Bonus: for your LaTeX formulas!

            let parser = Parser::new_ext(&markdown_input, options);
            let mut html_output = String::new();
            html::push_html(&mut html_output, parser);

            Html(wrap_in_template(&html_output)).into_response()
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

fn wrap_in_template(content: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1">
            <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/github-markdown-css/5.5.1/github-markdown.min.css">
            <style>
                .markdown-body {{ box-sizing: border-box; min-width: 200px; max-width: 980px; margin: 0 auto; padding: 45px; color: #c9d1d9; }}
                @media (max-width: 767px) {{ .markdown-body {{ padding: 15px; }} }}
                body {{ background-color: #0d1117; }}
            </style>
            <title>Research | Martian17</title>
        </head>
        <body class="markdown-body">
            {}
        </body>
        </html>"#,
        content
    )
}
