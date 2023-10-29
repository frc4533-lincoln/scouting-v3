use axum::{
    body::Body,
    extract::{Path, Query},
    http::{header, Response, Uri},
    response::IntoResponse,
    routing::{get, post},
    Extension, Form, Json, Router, Server,
};
use hashbrown::HashMap;
use notify_debouncer_mini::{notify::RecursiveMode, DebounceEventResult};
use sled::Db;
use std::{
    error::Error,
    fs::{read_dir, File},
    io::Read,
    sync::Arc,
    time::Duration,
};
use tokio::sync::Mutex;

#[macro_use]
extern crate tokio;
extern crate axum;
extern crate hashbrown;
extern crate notify_debouncer_mini;
extern crate rust_embed;
extern crate scouting_v3;
extern crate sled;

use rust_embed::RustEmbed;
use scouting_v3::*;

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/../dist/"]
struct Assets;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // I'm using a hash map for the events
    let events: Arc<Mutex<HashMap<String, EventCfg>>> = Arc::new(Mutex::new(HashMap::new()));

    load_event_cfgs(events.clone()).await;

    let db = sled::open(".scouting-db")?;

    let rtr = Router::new()
        .nest(
            "/api",
            Router::new()
                .route("/status", get(status))
                .route("/events", get(list_events))
                .route("/scouters", get(list_scouters).put(register_scouter))
                .nest(
                    "/manage",
                    Router::new().route("/set", post(manager_set)), // .route("/assign/:scouter", post(assign_scouter)),
                ), // .route("/scoutees/:event", get(list_scoutees)), // .route("/scoutees/:id", post(submit_scouting_results)),
        )
        .layer(Extension(events))
        .layer(Extension(db))
        .fallback(static_handler);

    Server::bind(&"0.0.0.0:8000".parse()?)
        .serve(rtr.into_make_service())
        .await?;

    Ok(())
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    if path.is_empty() || path == "index.html" {
        return Response::new(Body::from(Assets::get("index.html").unwrap().data)).into_response();
    }

    match Assets::get(path) {
        Some(content) => Response::builder()
            .header(
                header::CONTENT_TYPE,
                if path.ends_with(".wasm") {
                    "application/wasm"
                } else {
                    "text/javascript"
                },
            )
            .body(Body::from(content.data))
            .unwrap()
            .into_response(),
        None => Response::builder()
            .status(418)
            .body(Body::empty())
            .unwrap()
            .into_response(),
    }
}

async fn load_event_cfgs(events: Arc<Mutex<HashMap<String, EventCfg>>>) {
    // Iterate over event cfgs and add them to the hashmap

    // Iterate over directory entries under ./events
    for f in read_dir("events").unwrap().into_iter() {
        // Unwrap directory entry bc std::fs readdir is retarded
        if let Ok(f) = f {
            // Filter out directories
            if f.file_type().map_or(false, |x| x.is_file()) {
                // Verify that the file name (OsString) can be converted to a Rust String and do so
                if let Ok(id) = f.file_name().into_string() {
                    // A buffer to load the contents of the event cfg into
                    let mut buf = String::new();

                    // Open the event cfg and read its contents into the buffer
                    File::open(f.path())
                        .unwrap()
                        .read_to_string(&mut buf)
                        .unwrap();

                    // Decode the TOML into Rust type
                    if let Ok(ev_cfg) = toml::de::from_str(buf.as_str()) {
                        // Insert the new/updated event cfg into the HashMap
                        (*events.lock().await)
                            .insert(id.strip_suffix(".toml").unwrap().to_string(), ev_cfg);
                    }
                }
            }
        }
    }

    // We want to allow for adding/updating while the server is running
    // To do this we must watch for filesystem changes

    let mut debouncer = notify_debouncer_mini::new_debouncer(
        Duration::from_secs(2),
        move |evs: DebounceEventResult| {
            if let Ok(evs) = evs {
                for ev in evs.iter() {
                    let mut buf = String::new();

                    File::open(ev.path.clone())
                        .unwrap()
                        .read_to_string(&mut buf)
                        .unwrap();

                    if let Ok(ev_cfg) = toml::de::from_str(buf.as_str()) {
                        (*events.blocking_lock()).insert(
                            ev.path.file_stem().unwrap().to_str().unwrap().to_string(),
                            ev_cfg,
                        );
                    }
                }
            }
        },
    )
    .unwrap();

    debouncer
        .watcher()
        .watch(std::path::Path::new("events"), RecursiveMode::Recursive)
        .unwrap();
}

async fn status(Extension(db): Extension<Db>) -> impl IntoResponse {
    let db = db.open_tree(b"manager").unwrap();

    let enabled = db
        .get(b"enabled")
        .unwrap()
        .map_or(false, |x| x.contains(&1));
    let managed = db
        .get(b"managed")
        .unwrap()
        .map_or(false, |x| x.contains(&1));
    let event = db.get(b"event").unwrap().map_or(String::default(), |x| {
        String::from_utf8(x.to_vec()).unwrap()
    });

    Json(Status {
        version: String::from(env!("CARGO_PKG_VERSION")),
        enabled,
        managed,
        event,
    })
}

// async fn assign_scouter(
//     Extension(db): Extension<Db>,
//     Path(scouter): Path<String>,
//     Form(data): Form<_>,
// ) -> impl IntoResponse {
//     let db = db.open_tree(b"scouters").unwrap();

//     db.insert(b"", b");
// }

async fn list_events(
    Extension(events): Extension<Arc<Mutex<HashMap<String, EventCfg>>>>,
) -> impl IntoResponse {
    let events = (*events.lock().await)
        .iter()
        .map(|(k, v)| Event {
            id: k.clone(),
            name: v.name.clone(),
        })
        .collect::<Vec<Event>>();

    Json(ListEvents {
        current: String::from("2023-scriw"),
        events,
    })
}

async fn list_scouters(Extension(db): Extension<Db>) -> impl IntoResponse {
    let scouters = db
        .open_tree(b"scouters")
        .unwrap()
        .iter()
        .map(|x| x.unwrap())
        .map(|(k, _)| String::from_utf8(k.to_vec()).unwrap())
        .collect::<Vec<String>>();

    Json(ListScouters { scouters })
}

async fn register_scouter(
    Extension(db): Extension<Db>,
    Query(scouter): Query<String>,
) -> impl IntoResponse {
    db.open_tree(b"scouters")
        .unwrap()
        .insert(scouter.as_bytes(), b"")
        .unwrap();

    Json(RegisterScouterRes {
        token: String::new(),
    })
}

async fn manager_set(
    Extension(db): Extension<Db>,
    Form(data): Form<ManagerSetReq>,
) -> impl IntoResponse {
    let db = db.open_tree(b"manager").unwrap();

    if let Some(e) = data.enabled {
        db.insert(b"enabled", if e { &[1] } else { &[0] }).unwrap();
    }

    if let Some(m) = data.managed {
        db.insert(b"managed", if m { &[1] } else { &[0] }).unwrap();
    }

    if let Some(ev) = data.event {
        db.insert(b"event", ev.as_bytes()).unwrap();
    }

    Response::new(Body::empty())
}
