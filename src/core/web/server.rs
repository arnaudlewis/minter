use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
use rust_embed::Embed;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};

use super::state::{Action, UiState};

#[derive(Embed)]
#[folder = "src/core/web/assets/"]
struct Assets;

/// Shared application state for the web server.
pub struct AppState {
    pub ui_state: Mutex<UiState>,
    pub tx: broadcast::Sender<String>,
}

/// JSON body for action requests that need a spec path.
#[derive(serde::Deserialize)]
pub struct ActionBody {
    pub spec_path: Option<PathBuf>,
}

/// Run the web dashboard server.
///
/// Returns an exit code (0 = clean shutdown, 1 = error).
pub async fn run_server(working_dir: PathBuf, port: u16, no_open: bool) -> i32 {
    // 1. Load initial state
    let ui_state = UiState::load(&working_dir);
    let (tx, _rx) = broadcast::channel::<String>(64);

    let app_state = Arc::new(AppState {
        ui_state: Mutex::new(ui_state),
        tx: tx.clone(),
    });

    // 2. Setup file watcher
    let watcher_state = Arc::clone(&app_state);
    let watcher_dir = working_dir.clone();
    let watcher_tx = tx.clone();
    let rt = tokio::runtime::Handle::current();
    std::thread::spawn(move || {
        let (debounce_tx, debounce_rx) = std::sync::mpsc::channel();
        let mut debouncer = match new_debouncer(std::time::Duration::from_millis(300), debounce_tx)
        {
            Ok(d) => d,
            Err(e) => {
                eprintln!("warning: could not start file watcher: {}", e);
                return;
            }
        };

        // Watch the working directory recursively
        if let Err(e) = debouncer
            .watcher()
            .watch(watcher_dir.as_ref(), RecursiveMode::Recursive)
        {
            eprintln!("warning: could not watch directory: {}", e);
            return;
        }

        loop {
            match debounce_rx.recv() {
                Ok(Ok(events)) => {
                    // Only react to spec/test file changes
                    let dominated = events.iter().any(|e| {
                        let p = e.path.to_string_lossy();
                        p.ends_with(".spec")
                            || p.ends_with(".nfr")
                            || p.ends_with(".rs")
                            || p.ends_with(".ts")
                            || p.ends_with(".js")
                            || p.ends_with(".py")
                            || p.ends_with(".go")
                            || p.ends_with(".java")
                            || p.ends_with(".lock")
                    });

                    if !dominated {
                        continue;
                    }

                    // Refresh state and broadcast
                    let state_clone = Arc::clone(&watcher_state);
                    let tx_clone = watcher_tx.clone();
                    rt.spawn(async move {
                        let json = {
                            let mut state = state_clone
                                .ui_state
                                .lock()
                                .unwrap_or_else(|e| e.into_inner());
                            state.refresh();
                            state.validate_all();
                            serde_json::to_string(&*state).unwrap_or_else(|_| "{}".to_string())
                        };
                        let _ = tx_clone.send(json);
                    });
                }
                Ok(Err(errors)) => {
                    eprintln!("warning: file watcher errors: {:?}", errors);
                }
                Err(_) => break, // channel closed
            }
        }
    });

    // 3. Build router
    // SECURITY: localhost-only server, CORS is permissive by design.
    // The server binds to 127.0.0.1 and is not exposed on the network.
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/api/state", get(get_state))
        .route("/api/action/{name}", post(run_action))
        .route("/ws", get(ws_handler))
        .route("/assets/{*path}", get(serve_asset))
        .route("/{file}", get(serve_root_file))
        .layer(cors)
        .with_state(app_state);

    // 4. Bind to port, trying port+1 if in use
    let addr = match try_bind(port).await {
        Some(a) => a,
        None => match try_bind(port + 1).await {
            Some(a) => a,
            None => {
                eprintln!("error: could not bind to port {} or {}", port, port + 1);
                return 1;
            }
        },
    };

    let bound_port = addr.port();
    let url = format!("http://127.0.0.1:{}", bound_port);
    eprintln!("Minter dashboard: {}", url);

    // 5. Open browser unless --no-open
    if !no_open {
        if let Err(e) = open::that(&url) {
            eprintln!("warning: could not open browser: {}", e);
        }
    }

    // 6. Start server
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("error: could not start server: {}", e);
            return 1;
        }
    };

    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("error: server failed: {}", e);
        return 1;
    }

    0
}

/// Try to check if a port is available by briefly binding to it.
async fn try_bind(port: u16) -> Option<SocketAddr> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            let local_addr = listener.local_addr().ok();
            drop(listener);
            local_addr
        }
        Err(_) => None,
    }
}

// ── Route handlers ──────────────────────────────────────

/// GET / — serve the index.html from embedded assets.
async fn serve_index() -> impl IntoResponse {
    match Assets::get("index.html") {
        Some(content) => {
            Html(String::from_utf8_lossy(content.data.as_ref()).to_string()).into_response()
        }
        None => (StatusCode::NOT_FOUND, "index.html not found").into_response(),
    }
}

/// GET /assets/*path — serve static assets from embedded files.
async fn serve_asset(Path(path): Path<String>) -> impl IntoResponse {
    let embed_path = format!("assets/{}", path);
    match Assets::get(&embed_path) {
        Some(content) => {
            let mime = mime_from_path(&path);
            Response::builder()
                .header(header::CONTENT_TYPE, mime)
                .body(axum::body::Body::from(content.data.to_vec()))
                .unwrap_or_else(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to build response",
                    )
                        .into_response()
                })
        }
        None => (StatusCode::NOT_FOUND, "asset not found").into_response(),
    }
}

/// GET /:file — serve root-level static files (favicon, logos).
async fn serve_root_file(Path(file): Path<String>) -> impl IntoResponse {
    match Assets::get(&file) {
        Some(content) => {
            let mime = mime_from_path(&file);
            Response::builder()
                .header(header::CONTENT_TYPE, mime)
                .body(axum::body::Body::from(content.data.to_vec()))
                .unwrap_or_else(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to build response",
                    )
                        .into_response()
                })
        }
        None => (StatusCode::NOT_FOUND, "file not found").into_response(),
    }
}

/// GET /api/state — return the full UiState as JSON.
async fn get_state(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let ui_state = state.ui_state.lock().unwrap_or_else(|e| e.into_inner());
    let json = serde_json::to_string(&*ui_state).unwrap_or_else(|_| "{}".to_string());
    Response::builder()
        .header(header::CONTENT_TYPE, "application/json")
        .body(axum::body::Body::from(json))
        .unwrap_or_else(|_| {
            (StatusCode::INTERNAL_SERVER_ERROR, "serialization error").into_response()
        })
}

/// POST /api/action/:name — execute an action and return the result.
async fn run_action(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    body: Option<Json<ActionBody>>,
) -> impl IntoResponse {
    let action = match name.as_str() {
        "validate" => Action::Validate,
        "deep-validate" => Action::DeepValidate,
        "coverage" => Action::Coverage,
        "lock" => Action::Lock,
        "graph" => Action::Graph,
        "inspect" => Action::Inspect,
        "format" => Action::Format,
        "scaffold" => Action::Scaffold,
        "guide" => Action::Guide,
        _ => {
            return (StatusCode::BAD_REQUEST, format!("unknown action: {}", name)).into_response();
        }
    };

    let spec_path = body.and_then(|b| b.spec_path.clone());

    let (result_json, is_lock) = {
        let ui_state = state.ui_state.lock().unwrap_or_else(|e| e.into_inner());
        let result = ui_state.run_action(action, spec_path.as_deref());
        let json = serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string());
        (json, matches!(name.as_str(), "lock"))
    };

    // For Lock action: refresh state and broadcast to WS clients
    if is_lock {
        let broadcast_json = {
            let mut ui_state = state.ui_state.lock().unwrap_or_else(|e| e.into_inner());
            ui_state.refresh();
            ui_state.validate_all();
            serde_json::to_string(&*ui_state).unwrap_or_else(|_| "{}".to_string())
        };
        let _ = state.tx.send(broadcast_json);
    }

    Response::builder()
        .header(header::CONTENT_TYPE, "application/json")
        .body(axum::body::Body::from(result_json))
        .unwrap_or_else(|_| {
            (StatusCode::INTERNAL_SERVER_ERROR, "serialization error").into_response()
        })
}

/// GET /ws — WebSocket upgrade for live state updates.
async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

async fn handle_ws(mut socket: WebSocket, state: Arc<AppState>) {
    // Send initial state on connect
    let initial_json = {
        let ui_state = state.ui_state.lock().unwrap_or_else(|e| e.into_inner());
        serde_json::to_string(&*ui_state).unwrap_or_else(|_| "{}".to_string())
    };
    if socket
        .send(Message::Text(initial_json.into()))
        .await
        .is_err()
    {
        return;
    }

    // Subscribe to broadcast updates
    let mut rx = state.tx.subscribe();

    loop {
        tokio::select! {
            msg = rx.recv() => {
                match msg {
                    Ok(json) => {
                        if socket.send(Message::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            ws_msg = socket.recv() => {
                match ws_msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {} // ignore other client messages
                }
            }
        }
    }
}

/// Guess MIME type from file extension.
fn mime_from_path(path: &str) -> &'static str {
    if path.ends_with(".html") {
        "text/html; charset=utf-8"
    } else if path.ends_with(".css") {
        "text/css; charset=utf-8"
    } else if path.ends_with(".js") {
        "application/javascript; charset=utf-8"
    } else if path.ends_with(".json") {
        "application/json"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".ico") {
        "image/x-icon"
    } else if path.ends_with(".woff2") {
        "font/woff2"
    } else if path.ends_with(".woff") {
        "font/woff"
    } else {
        "application/octet-stream"
    }
}
