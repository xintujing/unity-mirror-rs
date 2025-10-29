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

pub fn start() {
    // tokio::spawn(async move {
    //     // initialize tracing
    //     // tracing_subscriber::fmt::init();
    //
    //     // build our application with a route
    //     let app = Router::new()
    //         // `GET /` goes to `root`
    //         .route("/", get(root));
    //
    //     // run our app with hyper, listening globally on port 3000
    //     let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    //     axum::serve(listener, app).await.unwrap();
    // });
}

async fn root() -> &'static str {
    let mut output = String::default();
    if let Some(world) = unity_engine::WorldManager::active_world().get() {
        output.push_str(world.get_scene_path().as_str());
        output.push_str("\n");

        for gm in unity_engine::WorldManager::root_game_objects().iter() {
            if let Some(gm) = gm.upgrade() {
                dg(0, &mut output, vec![&gm])
            }
        }
    }
    Box::leak(output.into_boxed_str())
}

pub(crate) fn dg(level: u8, output: &mut String, gms: Vec<&RevelArc<GameObject>>) {
    for gm in gms.iter() {
        output.push_str(format!("{}{}:{}", " ".repeat(level as usize), gm.id, gm.name).as_str());
        // output.push_str(format!("{}- {}:{}", " ".repeat(level as usize), gm.id, gm.name).as_str())
        if !gm.children.is_empty() {
            dg(level + 1, output, gm.children.values().collect::<Vec<_>>())
        }
    }
}

//
// fn comp(level: u8, output: &mut String, gms: Vec<RevelWeak<GameObject>>) {
//     for gm in gms.iter() {
//         if let Some(gm) = gm.upgrade() {
//             output.push_str(format!("{}{}:{}", " ".repeat(level as usize), gm.id, gm.name).as_str())
//             output.push_str(format!("{}- {}:{}", " ".repeat(level as usize), gm.id, gm.name).as_str())
//         }
//     }
// }

async fn create_user(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    // insert your application logic here
    let user = User {
        id: 1337,
        username: payload.username,
    };

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(user))
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}
