use axum::{
    Router,
    extract::{
        Path, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::{Html, Response},
    routing::get,
};
use futures_util::{SinkExt, StreamExt};
use tera::{Context, Tera};
use tower_http::services::ServeDir;

use tokio::sync::broadcast::{self, Receiver, Sender};

use serde::{Deserialize, Serialize};

use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "lowercase")]
pub enum SlideMessage {
    Text(String),
    Slide(u32),
}

pub struct Presentation {
    content: String,
    slide: u32,
    channel: (Sender<SlideMessage>, Receiver<SlideMessage>),
}

#[derive(Clone, Default)]
pub struct AppState {
    tera: Arc<Tera>,
    slides: Arc<Mutex<HashMap<String, Arc<Mutex<Presentation>>>>>,
}

async fn broadcast_to_all(
    ws: WebSocketUpgrade,
    Path(pid): Path<String>,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| ws_handle(socket, pid, state))
}

fn update_slide(pid: String, msg: SlideMessage, state: &mut AppState) {
    let mut slides = state.slides.lock().unwrap();
    let mut pres = slides.get_mut(&pid).unwrap().lock().unwrap();
    match msg {
        SlideMessage::Slide(sn) => {
            pres.slide = sn;
        }
        SlideMessage::Text(text) => {
            pres.content = text;
        }
    };
}

fn add_client_handler_channel(pid: String, state: &mut AppState) -> Arc<Mutex<Presentation>> {
    let mut slides = if let Ok(slides) = state.slides.lock() {
        slides
    } else {
        // TODO: no panics
        panic!("Unable to lock K/V store!");
    };
    let pres = slides
        .entry(pid)
        .or_insert(Arc::new(Mutex::new(Presentation {
            content: "".to_string(),
            slide: 0,
            channel: broadcast::channel(1024),
        })));
    return Arc::clone(&pres);
}

async fn handle_socket(
    msg: Result<Message, axum::Error>,
    pid: String,
    tx: &mut Sender<SlideMessage>,
    mut state: &mut AppState,
) -> Result<bool, &'static str> {
    let msg = if let Ok(msg) = msg {
        msg
    // disconnecteed
    } else {
        return Err("Disconnected");
    };
    if let Message::Close(_) = msg {
        return Err("Closed");
    }
    let msg: SlideMessage = match serde_json::from_str(msg.to_text().unwrap()) {
        Ok(msg) => msg,
        Err(e) => {
            // TODO: proper error handling
            return Err("Invalid message!");
        }
    };
    update_slide(pid.clone(), msg.clone(), &mut state);

    if tx.send(msg).is_err() {
        // disconnected
        return Err("Channel disconnected!");
    }
    return Ok(true);
}

async fn ws_handle(mut socket: WebSocket, pid: String, mut state: AppState) {
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
            if handle_socket(msg, pid.clone(), &mut tx, &mut state1)
                .await
                .is_err()
            {
                return;
            }
        }
    };
    let channel_handler = async {
        while let Ok(msg) = rx.recv().await {
            update_slide(pid.clone(), msg.clone(), &mut state);
            let text = serde_json::to_string(&msg).unwrap();
            sock_send.send(Message::from(text)).await.unwrap();
        }
    };
    use futures_lite::future::or;
    let _ = or(socket_handler, channel_handler).await;
}

async fn join(State(st): State<AppState>) -> Html<String> {
    let html = st.tera.render("join.html", &Context::new()).unwrap();
    Html(html)
}
async fn audience(State(st): State<AppState>) -> Html<String> {
    let html = st.tera.render("audience.html", &Context::new()).unwrap();
    Html(html)
}
async fn start(State(st): State<AppState>) -> Html<String> {
    let html = st.tera.render("start.html", &Context::new()).unwrap();
    Html(html)
}
async fn stage(State(st): State<AppState>) -> Html<String> {
    let html = st.tera.render("stage.html", &Context::new()).unwrap();
    Html(html)
}
async fn index(State(st): State<AppState>) -> Html<String> {
    let html = st.tera.render("index.html", &Context::new()).unwrap();
    Html(html)
}

/// Dynamic cleanup of still open presentations.
///
/// This takes a "stop-the-world" approach to finding and dynamically dropping the memory for
/// presentations without active listeners.
///
/// TODO: more effective solution via counting the number of _other_ connections each time a client
/// is disconnected, and freeing the memory then.
/// This would be vastly more efficent and scale better.
async fn cleanup(state: AppState) {
    let mut slides = state.slides.lock().unwrap();
    slides.retain(|_k, v| Arc::strong_count(v) == 1);
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // USR1 signal causes cleanup routine
    let sig_handle = Signals::new(&[SIGUSR1]).unwrap().handle();
    let tera = Tera::new("templates/**/*.html").unwrap();
    let state = AppState {
        tera: Arc::new(tera),
        slides: Arc::new(Mutex::new(HashMap::new())),
    };
    let app = Router::new()
        .route("/", get(index))
        .route("/audience", get(join))
        .route("/audience/{pid}", get(audience))
        .route("/stage", get(start))
        .route("/stage/{pid}", get(stage))
        .route("/ws/{pid}", get(broadcast_to_all))
        .nest_service("/css", ServeDir::new("../src/css/"))
        .nest_service("/js", ServeDir::new("../src/js/"))
        .nest_service("/demo", ServeDir::new("../src/demo/"))
        .with_state(state.clone());
    let listener = tokio::net::TcpListener::bind("0.0.0.0:5002").await.unwrap();
    let signal_task = tokio::spawn(cleanup(state));
    axum::serve(listener, app).await.unwrap();
    sig_handle.close();
    let _ = signal_task.await;
}
