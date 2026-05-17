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
        // This handler now decides: is it a directory, a Markdown page, or a static asset?
        .route("/{*path}", get(handle_request))
        .fallback_service(static_service);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Martian Research Hub running on http://0.0.0.0:8080");
    axum::serve(listener, app).await.unwrap();
}

async fn serve_index(req: Request<Body>) -> Response {
    handle_path(String::new(), req).await
}

async fn handle_request(AxPath(path): AxPath<String>, req: Request<Body>) -> Response {
    handle_path(path, req).await
}

/// Unified path handler to manage directories, static assets, and markdown files
async fn handle_path(path: String, req: Request<Body>) -> Response {
    let full_path = PathBuf::from("./content").join(&path);

    // 1. If the path is a directory, handle index checks and auto-generation
    if full_path.is_dir() {
        // Enforce a trailing slash for directory URLs so relative links work reliably
        if !path.is_empty() && !path.ends_with('/') {
            return Response::builder()
                .status(StatusCode::MOVED_PERMANENTLY)
                .header(header::LOCATION, format!("/{}/", path))
                .body(Body::empty())
                .unwrap();
        }

        // If index.html exists, let ServeDir handle it
        if full_path.join("index.html").exists() {
            match ServeDir::new("./content").oneshot(req).await {
                Ok(res) => return res.into_response(),
                Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        }

        // If index.md exists instead, render it as the directory's homepage
        if full_path.join("index.md").exists() {
            let index_md_target = if path.is_empty() {
                "index".to_string()
            } else {
                format!("{}index", path)
            };
            return render_markdown(index_md_target).await.into_response();
        }

        // Auto-generate a directory index page if no index file exists
        return match generate_directory_index(&path, &full_path) {
            Ok(html_content) => Html(html_content).into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };
    }

    // 2. If it has an extension (e.g., .pdf, .js, .webp), treat it as a static file
    let path_buf = PathBuf::from(&path);
    if path_buf.extension().is_some() {
        match ServeDir::new("./content").oneshot(req).await {
            Ok(res) => return res.into_response(),
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }

    // 3. If no extension and not a directory, try to render it as a Markdown file
    render_markdown(path).await.into_response()
}

async fn render_markdown(name: String) -> impl IntoResponse {
    let mut path = PathBuf::from("./content/");
    path.push(format!("{}.md", name));

    match fs::read_to_string(path) {
        Ok(markdown_input) => {
            let html_output = markdown_to_html(&markdown_input);
            Html(wrap_in_template(&html_output)).into_response()
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

/// Dynamically builds a clean Markdown directory listing and compiles it to HTML
fn generate_directory_index(display_path: &str, full_path: &std::path::Path) -> Result<String, std::io::Error> {
    let title_path = if display_path.is_empty() { "/" } else { display_path };
    let mut markdown_input = format!("# Index of `{}`\n\n", title_path);

    // Provide a back/up link if we're inside a subdirectory
    if !display_path.is_empty() {
        markdown_input.push_str("* [📁 ..](../)\n");
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(full_path)? {
        let entry = entry?;
        let file_name = entry.file_name().to_string_lossy().into_owned();

        // Ignore hidden files and directories
        if file_name.starts_with('.') {
            continue;
        }

        let file_type = entry.file_type()?;
        entries.push((file_name, file_type.is_dir()));
    }

    // Sort entries: directories first, followed by files alphabetically
    entries.sort_by(|a, b| {
        if a.1 != b.1 {
            b.1.cmp(&a.1) 
        } else {
            a.0.cmp(&b.0)
        }
    });

    for (name, is_dir) in entries {
        if is_dir {
            markdown_input.push_str(&format!("* [📁 {}/](./{}/)\n", name, name));
        } else if name.ends_with(".md") {
            // Strip the extension for markdown files so clicking them triggers the HTML renderer
            let base_name = &name[..name.len() - 3];
            markdown_input.push_str(&format!("* [📄 {}](./{})\n", base_name, base_name));
        } else {
            markdown_input.push_str(&format!("* [📄 {}](./{})\n", name, name));
        }
    }

    let html_output = markdown_to_html(&markdown_input);
    Ok(wrap_in_template(&html_output))
}

fn markdown_to_html(markdown_input: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(markdown_input, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
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
