use anyhow::Context;
use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::any,
    Router,
};
use std::{
    net::SocketAddr,
    path::{Component, Path as FsPath, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::{fs, net::TcpListener, task::JoinHandle};

#[derive(Clone)]
struct WebdavState {
    root: PathBuf,
    token: String,
}

pub async fn spawn(
    root: PathBuf,
    addr: SocketAddr,
    token: String,
) -> anyhow::Result<(String, JoinHandle<anyhow::Result<()>>)> {
    let root = root
        .canonicalize()
        .with_context(|| format!("failed to canonicalize {}", root.display()))?;
    let state = WebdavState { root, token };
    let app = Router::new()
        .route("/webdav/{*path}", any(dispatch_path))
        .route("/webdav", any(dispatch_root))
        .with_state(state);

    let listener = TcpListener::bind(addr).await?;
    let local_addr = listener.local_addr()?;
    let url = format!("http://{local_addr}/webdav");
    let task = tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .context("webdav server failed")
    });

    Ok((url, task))
}

async fn dispatch_root(
    State(state): State<WebdavState>,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    dispatch(&state, method, headers, "", body).await
}

async fn dispatch_path(
    State(state): State<WebdavState>,
    Path(path): Path<String>,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    dispatch(&state, method, headers, &path, body).await
}

async fn dispatch(
    state: &WebdavState,
    method: Method,
    headers: HeaderMap,
    path: &str,
    body: Bytes,
) -> Response {
    match method.as_str() {
        "GET" => read_response(state, &headers, path).await.into_response(),
        "PUT" => write_response(state, &headers, path, body)
            .await
            .into_response(),
        "PROPFIND" => propfind_response(state, &headers, path)
            .await
            .into_response(),
        _ => StatusCode::METHOD_NOT_ALLOWED.into_response(),
    }
}

async fn read_response(
    state: &WebdavState,
    headers: &HeaderMap,
    path: &str,
) -> Result<(StatusCode, Vec<u8>), StatusCode> {
    authorize(headers, &state.token)?;
    let path = resolve_workspace_path(&state.root, path)?;
    fs::read(path)
        .await
        .map(|bytes| (StatusCode::OK, bytes))
        .map_err(|_| StatusCode::NOT_FOUND)
}

async fn write_response(
    state: &WebdavState,
    headers: &HeaderMap,
    path: &str,
    body: Bytes,
) -> Result<StatusCode, StatusCode> {
    authorize(headers, &state.token)?;
    let path = resolve_workspace_path(&state.root, path)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    fs::write(path, body)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn propfind_response(
    state: &WebdavState,
    headers: &HeaderMap,
    path: &str,
) -> Result<(StatusCode, String), StatusCode> {
    authorize(headers, &state.token)?;
    let path = resolve_workspace_path(&state.root, path)?;
    let mut entries = fs::read_dir(path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    let mut names = Vec::new();

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        names.push(entry.file_name().to_string_lossy().to_string());
    }
    names.sort();

    Ok((StatusCode::MULTI_STATUS, build_propfind_xml(&names)))
}

fn authorize(headers: &HeaderMap, expected_token: &str) -> Result<(), StatusCode> {
    let Some(value) = headers.get(axum::http::header::AUTHORIZATION) else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    let Ok(value) = value.to_str() else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    let expected = format!("Bearer {expected_token}");
    if value == expected {
        Ok(())
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

pub fn resolve_workspace_path(root: &FsPath, request_path: &str) -> Result<PathBuf, StatusCode> {
    let mut resolved = root.to_path_buf();
    for component in FsPath::new(request_path.trim_start_matches('/')).components() {
        match component {
            Component::Normal(part) => resolved.push(part),
            Component::CurDir => {}
            _ => return Err(StatusCode::BAD_REQUEST),
        }
    }
    Ok(resolved)
}

pub fn build_propfind_xml(entries: &[String]) -> String {
    use std::fmt::Write as _;

    let mut responses = String::new();
    for entry in entries {
        let _ = write!(
            responses,
            "<d:response><d:href>{}</d:href><d:propstat><d:status>HTTP/1.1 200 OK</d:status></d:propstat></d:response>",
            escape_xml(entry)
        );
    }

    format!(
        r#"<?xml version="1.0" encoding="utf-8"?><d:multistatus xmlns:d="DAV:">{responses}</d:multistatus>"#
    )
}

pub fn generate_token() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("secret_{:x}_{:x}", std::process::id(), nanos)
}

fn escape_xml(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_parent_directory_traversal() {
        assert!(resolve_workspace_path(FsPath::new("/tmp/root"), "../secret").is_err());
    }

    #[test]
    fn joins_safe_workspace_path() {
        let resolved = resolve_workspace_path(FsPath::new("/tmp/root"), "src/main.rs").unwrap();

        assert!(resolved.ends_with("src/main.rs"));
    }

    #[test]
    fn propfind_xml_escapes_entries() {
        let xml = build_propfind_xml(&["a&b.rs".to_string()]);

        assert!(xml.contains("a&amp;b.rs"));
        assert!(xml.contains("multistatus"));
    }
}
