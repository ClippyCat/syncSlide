//! `syncslide-websocket`
//!
//! Runs the backend of the `SyncSlide` project.
//!
//! Handles live web-sockets (updating of slides live), as well as templated-HTML for most pages.
//!
#![deny(clippy::all, clippy::pedantic, rustdoc::all, unsafe_code, missing_docs)]

use axum::{
    Form, Router,
    extract::{
        FromRef, Path, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use axum_login::AuthManagerLayerBuilder;
use futures_lite::future::or;
use futures_util::{SinkExt, StreamExt};
use sqlx::SqlitePool;
use tera::{Context, Tera};
use time::Duration;
use tower_http::services::ServeDir;
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::SqliteStore;

use tokio::sync::broadcast::{self, Receiver, Sender};

use serde::{Deserialize, Serialize};

use signal_hook::consts::signal::SIGUSR1;
use signal_hook_tokio::Signals;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

mod db;
use db::{AuthSession, Backend, LoginForm, Presentation as DbPresentation, User};

/// A message indicating a _change_ in [`Presentation`] state.
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "lowercase")]
pub enum SlideMessage {
    /// Change the `content` field of the presentation.
    Text(String),
    /// Change the `slide` index.
    Slide(u32),
}

/// A specific presetation.
///
/// There is no key here, as presentations are stored in a hashmap with its associated keys.
pub struct Presentation {
    /// The full content of _all_ slides in the presentation.
    content: String,
    /// The slide index that is currently active.
    slide: u32,
    /// A set of channels for reading and writing to the sockets.
    channel: (Sender<SlideMessage>, Receiver<SlideMessage>),
}

/// The state of the entire application.
#[derive(Clone)]
pub struct AppState {
    /// Used to render HTML templates.
    tera: Arc<Tera>,
    /// Used to store all the ongoing presentation.
    /// They Key here is a user-defined string, and the value is a [`Presentation`] struct.
    slides: Arc<Mutex<HashMap<String, Arc<Mutex<Presentation>>>>>,
    db_pool: SqlitePool,
}

impl FromRef<AppState> for SqlitePool {
    fn from_ref(state: &AppState) -> Self {
        state.db_pool.clone()
    }
}
impl FromRef<AppState> for Arc<Tera> {
    fn from_ref(state: &AppState) -> Self {
        Arc::clone(&state.tera)
    }
}

async fn broadcast_to_all(
    ws: WebSocketUpgrade,
    Path(pid): Path<String>,
    State(state): State<AppState>,
    auth_session: AuthSession,
) -> Response {
    if auth_session.user.is_some() {
        ws.on_upgrade(|socket| ws_handle(socket, pid, state, true))
    } else {
        ws.on_upgrade(|socket| ws_handle(socket, pid, state, false))
    }
}

fn update_slide(pid: &str, msg: SlideMessage, state: &mut AppState) {
    let mut slides = state.slides.lock().unwrap();
    let mut pres = slides.get_mut(pid).unwrap().lock().unwrap();
    match msg {
        SlideMessage::Slide(sn) => {
            pres.slide = sn;
        }
        SlideMessage::Text(text) => {
            pres.content = text;
        }
    }
}

fn add_client_handler_channel(pid: String, state: &mut AppState) -> Arc<Mutex<Presentation>> {
    let Ok(mut slides) = state.slides.lock() else {
        // TODO: no panics
        panic!("Unable to lock K/V store!");
    };
    let pres = slides
        .entry(pid)
        .or_insert(Arc::new(Mutex::new(Presentation {
            content: String::new(),
            slide: 0,
            channel: broadcast::channel(1024),
        })));
    Arc::clone(pres)
}

fn handle_socket(
    msg: Result<Message, axum::Error>,
    pid: &str,
    tx: &mut Sender<SlideMessage>,
    state: &mut AppState,
) -> Result<bool, &'static str> {
    let Ok(msg) = msg else {
        cleanup(state);
        return Err("Disconnected");
    };
    if let Message::Close(_) = msg {
        cleanup(state);
        return Err("Closed");
    }
    let msg: SlideMessage = match serde_json::from_str(msg.to_text().unwrap()) {
        Ok(msg) => msg,
        Err(_e) => {
            // TODO: proper error handling
            return Err("Invalid message!");
        }
    };
    update_slide(pid, msg.clone(), state);

    if tx.send(msg).is_err() {
        cleanup(state);
        // disconnected
        return Err("Channel disconnected!");
    }
    Ok(true)
}

async fn ws_handle(mut socket: WebSocket, pid: String, mut state: AppState, auth: bool) {
    let pres = add_client_handler_channel(pid.clone(), &mut state);
    let (mut tx, mut rx, text, slide) = {
        let p = pres.lock().unwrap();
        let text = serde_json::to_string(&SlideMessage::Text(p.content.clone())).unwrap();
        let slide = serde_json::to_string(&SlideMessage::Slide(p.slide)).unwrap();
        let (tx, rx) = (p.channel.0.clone(), p.channel.0.subscribe());
        (tx, rx, text, slide)
    };
    socket.send(Message::from(text)).await.unwrap();
    socket.send(Message::from(slide)).await.unwrap();

    let mut state1 = state.clone();
    let (mut sock_send, mut sock_recv) = socket.split();
    let socket_handler = async {
        while let Some(msg) = sock_recv.next().await {
            if !auth {
                continue;
            }
            if handle_socket(msg, &pid, &mut tx, &mut state1).is_err() {
                return;
            }
        }
    };
    let channel_handler = async {
        while let Ok(msg) = rx.recv().await {
            update_slide(&pid, msg.clone(), &mut state);
            let text = serde_json::to_string(&msg).unwrap();
            sock_send.send(Message::from(text)).await.unwrap();
            let id = pid.parse().unwrap();
            if let SlideMessage::Text(text) = msg {
                let _ = DbPresentation::update_content(id, text, &state.db_pool).await;
            }
        }
    };
    let () = or(socket_handler, channel_handler).await;
    drop(pres);
}

async fn join(State(tera): State<Arc<Tera>>) -> Html<String> {
    let html = tera.render("join.html", &Context::new()).unwrap();
    Html(html)
}
async fn audience(tera: Arc<Tera>) -> Html<String> {
    let html = tera.render("audience.html", &Context::new()).unwrap();
    Html(html)
}
async fn start(State(tera): State<Arc<Tera>>, auth_session: AuthSession) -> impl IntoResponse {
    if auth_session.user.is_none() {
        return Redirect::to("/auth/login").into_response();
    }
    let html = tera.render("start.html", &Context::new()).unwrap();
    Html(html).into_response()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct NameForm {
    name: String,
}

async fn start_pres(
    State(db): State<SqlitePool>,
    auth_session: AuthSession,
    Form(name_form): Form<NameForm>,
) -> impl IntoResponse {
    let Some(user) = auth_session.user else {
        return Redirect::to("/auth/login").into_response();
    };
    if name_form.name.is_empty() {
        return Redirect::to("/start").into_response();
    }
    let pres = DbPresentation::new(&user, name_form.name, &db).await;
    if let Err(ref e) = pres {
        println!("{e:?}");
    }
    let Ok(pres) = pres else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    Redirect::to(&format!("/{}/{}", user.name, pres.id)).into_response()
}

async fn present(
    State(tera): State<Arc<Tera>>,
    State(db): State<SqlitePool>,
    auth_session: AuthSession,
    Path((uname, pid)): Path<(String, i64)>,
) -> impl IntoResponse {
    let audience_page = audience(Arc::clone(&tera)).await.into_response();
    let pres_user = User::get_by_name(uname, &db).await;
    let pres_user = match pres_user {
        Ok(Some(u)) => u,
        Ok(None) => return audience_page,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let pres = DbPresentation::get_by_id(pid, &db).await;
    let _pres = match pres {
        Ok(Some(p)) => p,
        Ok(None) => return audience_page,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let Some(ref user) = auth_session.user else {
        return audience_page;
    };
    if user.id != pres_user.id {
        return audience_page;
    }
    stage(tera, db, auth_session, pid).await.into_response()
}

async fn stage(
    tera: Arc<Tera>,
    db: SqlitePool,
    auth_session: AuthSession,
    pid: i64,
) -> impl IntoResponse {
    if auth_session.user.is_none() {
        return Redirect::to("/auth/login").into_response();
    }
    let pres = DbPresentation::get_by_id(pid, &db).await.unwrap();
    let mut ctx = Context::new();
    ctx.insert("pres", &pres);
    let html = tera.render("stage.html", &ctx).unwrap();
    Html(html).into_response()
}
async fn presentations(
    State(tera): State<Arc<Tera>>,
    State(db): State<SqlitePool>,
    auth_session: AuthSession,
) -> impl IntoResponse {
    let Some(user) = auth_session.user else {
        return Redirect::to("/auth/login").into_response();
    };
    let press = DbPresentation::get_for_user(&user, &db).await;
    let Ok(press) = press else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    let mut ctx = Context::new();
    ctx.insert("press", &press);
    ctx.insert("user", &user);
    let html = tera.render("presentations.html", &ctx).unwrap();
    Html(html).into_response()
}
async fn login(State(tera): State<Arc<Tera>>) -> Html<String> {
    let html = tera.render("login.html", &Context::new()).unwrap();
    Html(html)
}

async fn login_process(
    State(tera): State<Arc<Tera>>,
    mut auth_session: AuthSession,
    Form(login): Form<LoginForm>,
) -> impl IntoResponse {
    let user = match auth_session.authenticate(login).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return Html(tera.render("login.html", &Context::new()).unwrap()).into_response();
        }
        Err(_) => {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    if auth_session.login(&user).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    Redirect::to("/").into_response()
}
async fn logout(mut auth_session: AuthSession) -> impl IntoResponse {
    match auth_session.logout().await {
        Ok(_) => Redirect::to("/").into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn index(
    State(tera): State<Arc<Tera>>,
    auth_session: AuthSession,
    State(db): State<SqlitePool>,
) -> impl IntoResponse {
    let mut ctx = Context::new();
    ctx.insert("user", &auth_session.user);
    if let Some(user) = auth_session.user {
        let pn = DbPresentation::num_for_user(&user, &db).await.unwrap();
        ctx.insert("pres_num", &pn);
    }
    let html = tera.render("index.html", &ctx).unwrap();
    Html(html).into_response()
}

/// Dynamic cleanup of still open presentations.
fn cleanup(state: &mut AppState) {
    let mut slides = state.slides.lock().unwrap();
    slides.retain(|_k, v| Arc::strong_count(v) > 1);
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // USR1 signal causes cleanup routine
    let sig_handle = Signals::new([SIGUSR1]).unwrap().handle();
    let tera = Tera::new("templates/**/*.html").unwrap();
    let db_pool = SqlitePool::connect("sqlite://db.sqlite3").await.unwrap();
    let session_store = SqliteStore::new(db_pool.clone());
    session_store.migrate().await.unwrap();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(true)
        .with_expiry(Expiry::OnInactivity(Duration::days(1)));
    let backend = Backend::new(db_pool.clone());
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    let mut state = AppState {
        tera: Arc::new(tera),
        slides: Arc::new(Mutex::new(HashMap::new())),
        db_pool,
    };
    let app = Router::new()
        .route("/", get(index))
        .route("/auth/login", get(login))
        .route("/auth/login", post(login_process))
        .route("/auth/logout", get(logout))
        .route("/user/presentations", get(presentations))
        .route("/join", get(join))
        .route("/stage", get(start))
        .route("/stage", post(start_pres))
        .route("/{uname}/{pid}", get(present))
        .route("/ws/{pid}", get(broadcast_to_all))
        .nest_service("/css", ServeDir::new("../src/css/"))
        .nest_service("/js", ServeDir::new("../src/js/"))
        .nest_service("/demo", ServeDir::new("../src/demo/"))
        .with_state(state.clone())
        .layer(auth_layer);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:5002").await.unwrap();
    let signal_task = tokio::spawn(async move { cleanup(&mut state) });
    axum::serve(listener, app).await.unwrap();
    sig_handle.close();
    let _ = signal_task.await;
}
