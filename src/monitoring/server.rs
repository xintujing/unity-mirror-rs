use crate::commons::{RevelArc, RevelWeak};
use crate::mirror::{NetworkServer, NetworkServerStatic};
use crate::unity_engine;
use crate::unity_engine::GameObject;
use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::any::{type_name, Any, TypeId};

pub fn start(addr: String) {
    tokio::spawn(async move {
        // initialize tracing
        // tracing_subscriber::fmt::init();

        // build our application with a route
        let app = Router::new()
            // `GET /` goes to `root`
            .route("/", get(root));

        // run our app with hyper, listening globally on port 3000
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });
}

async fn root() -> &'static str {
    let mut output = String::default();
    if let Some(world) = unity_engine::WorldManager::active_world().get() {
        output.push_str(world.get_scene_path().as_str());
        output.push_str("\n");
    }

    for gm in unity_engine::WorldManager::root_game_objects().iter() {
        if let Some(gm) = gm.upgrade() {
            dg(0, &mut output, vec![&gm]);
            output.push_str("\n");
        }
    }

    Box::leak(output.into_boxed_str())
}

pub(crate) fn dg(level: u8, output: &mut String, gms: Vec<&RevelArc<GameObject>>) {
    for gm in gms.iter() {
        let t = " ".repeat(level as usize);
        output.push_str(format!("{}{}:{}\n", t, gm.id, gm.name).as_str());
        // output.push_str(format!("{}- {}:{}", " ".repeat(level as usize), gm.id, gm.name).as_str())

        for comp in gm.components.iter() {
            if let Some(comp) = comp.last() {
                output.push_str(format!("- {}{}\n", t, comp.type_name()).as_str())
            }
        }

        if !gm.children.is_empty() {
            dg(level + 1, output, gm.children.values().collect::<Vec<_>>())
        }
    }
}
