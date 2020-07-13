use serde_derive::{Serialize, Deserialize};
use warp::Filter;
use warp::reply::Reply;

use dice::dice::DiceRoller;
use dice::parse::parse_str;

mod mime;
mod template;

// use crate::template::{compile_templates, serve_template, State};

use std::error::Error;
use std::net::SocketAddr;
// use std::sync::Arc;

const STATIC_DIR_PATH: &str = "./static/";
const APP_JS: &str = "./frontend/static/main.js";
const APP_WASM: &str = "./frontend/static/main_bg.wasm";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();

    // let templates = compile_templates(&[
    //     "./templates/index.html.liquid",
    //     "./templates/style.css.liquid",
    //     // "./templates/main.js.liquid",
    // ])
    // .await?;
    // log::info!("{} templates compiled.", templates.len());
    //
    // let state = Arc::new(State::new(templates));
    //
    // let _with_state = {
    //     let filter = warp::filters::any::any().map(move || state.clone());
    //     move || filter.clone()
    // };

    let statics = warp::filters::method::get()
        .and(warp::fs::dir(STATIC_DIR_PATH))
        .and(warp::path::end());

    let js = warp::filters::method::get()
        .and(warp::path("main.js"))
        .and(warp::path::end())
        .and(warp::fs::file(APP_JS));

    let wasm = warp::filters::method::get()
        .and(warp::path("main_bg.wasm"))
        .and(warp::path::end())
        .and(warp::fs::file(APP_WASM));

    let dice = warp::filters::method::post()
        .and(warp::path("dice"))
        // 16kb
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json())
        .map(|req: DiceRequest| {
            log::info!("Received a request: {:?}", req.roll);

            let mut roller = DiceRoller::new();

            if let Ok((times, dice)) = parse_str(req.roll.as_str()) {
                let roll = roller.roll_dice_times(&dice, times);

                warp::reply::json(&DiceResponse { roll }).into_response()
            } else {
                http::Response::builder()
                    .status(422)
                    .body("Invalid roll.").into_response()
            }
    });

    let addr = "0.0.0.0:3000";
    log::info!("Serving server on {}", addr);
    warp::serve(statics.or(js).or(wasm).or(dice))
        .run(addr.parse::<SocketAddr>()?)
        .await;

    Ok(())
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct DiceRequest {
    pub roll: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct DiceResponse {
    pub roll: Vec<i64>,
}

trait ForWarp {
    type Reply;

    fn for_warp(self) -> Result<Self::Reply, warp::Rejection>;
}

impl<T> ForWarp for Result<T, Box<dyn Error>> 
where
    T: warp::Reply + 'static
{
    type Reply = Box<dyn warp::Reply>;

    fn for_warp(self) -> Result<Self::Reply, warp::Rejection> {
        let b: Box<dyn warp::Reply> = match self {
            Ok(reply) => Box::new(reply),
            Err(e) => {
                log::error!("Error: {}", e);
                let res = http::Response::builder()
                    .status(500)
                    .body("Something went wrong, apologies.");
                Box::new(res)
            }
        };
        Ok(b)
    }
}
