use axum::{
    body::Body,
    extract::Path,
    http::{header, Response, StatusCode},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use pulldown_cmark::{html, Options, Parser};
use std::fs;
use std::path::PathBuf;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let app = Router::new()
        // Serve the root "About Me" page
        .route("/", get(serve_index))
        // Dynamic route for Markdown files
        .route("/{file}", get(render_markdown))
        // Serve PDFs and other assets from the "content" folder
        .fallback_service(ServeDir::new("./content"));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Martian Research Server running on http://localhost:8080");
    axum::serve(listener, app).await.unwrap();
}

// Renders index.md from the content folder
async fn serve_index() -> impl IntoResponse {
    render_markdown(Path("index".to_string())).await
}

async fn render_markdown(Path(name): Path<String>) -> impl IntoResponse {
    let path = PathBuf::from(format!("./content/{}.md", name));

    match fs::read_to_string(path) {
        Ok(markdown_input) => {
            let mut options = Options::empty();
            options.insert(Options::ENABLE_TABLES);
            options.insert(Options::ENABLE_FOOTNOTES);
            options.insert(Options::ENABLE_STRIKETHROUGH);
            options.insert(Options::ENABLE_TASKLISTS);

            let parser = Parser::new_ext(&markdown_input, options);
            let mut html_output = String::new();
            html::push_html(&mut html_output, parser);

            Html(wrap_in_template(&html_output)).into_response()
        }
        Err(_) => {
            // If the .md file doesn't exist, we return a 404
            (StatusCode::NOT_FOUND, "Research paper not found").into_response()
        }
    }
}

fn wrap_in_template(content: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
        <html>
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1">
            <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/github-markdown-css/5.5.1/github-markdown.min.css">
            <style>
                .markdown-body {{ box-sizing: border-box; min-width: 200px; max-width: 980px; margin: 0 auto; padding: 45px; }}
                @media (max-width: 767px) {{ .markdown-body {{ padding: 15px; }} }}
                body {{ background-color: #0d1117; color: #c9d1d9; }}
            </style>
        </head>
        <body class="markdown-body">
            {}
        </body>
        </html>"#,
        content
    )
}
