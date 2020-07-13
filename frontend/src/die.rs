use anyhow::Error;
use http::request::Request;
use http::response::Response;
use serde_derive::Deserialize;
use serde_json::json;
use yew::format::Json;
use yew::prelude::*;
use yew::services::fetch::{FetchService, FetchTask};

use crate::app::DieData;

#[derive(Debug, Deserialize)]
struct Data {
    pub roll: Vec<i64>,
}

fn send_roll_request(die: &mut Die) {
    let json = &json!({"roll": die.roll});
    let post_request = Request::post("/dice")
        .header("Content-Type", "application/json")
        .body(Json(json))
        .expect("Failed to build post request.");

    let task = FetchService::fetch(
        post_request,
        die.link
            .callback(|response: Response<Json<Result<Data, Error>>>| {
                log::info!(
                    "headers: {:?}, status: {:?}, body: {:?}",
                    response.headers(),
                    response.status(),
                    response.body()
                );
                if let (meta, Json(Ok(body))) = response.into_parts() {
                    if meta.status.is_success() {
                        return Msg::Output(format!("{:?}", body.roll));
                    }
                }
                Msg::FetchFailed
            }),
    );

    if let Ok(t) = task {
        die.fetch_task = Some(t)
    }
}

#[derive(Debug)]
pub struct Die {
    pub link: ComponentLink<Self>,
    pub name: String,
    pub roll: String,
    pub output: String,
    pub fetch_task: Option<FetchTask>,
    pub onsignal: Callback<(String, DieData)>,
}

pub enum Msg {
    InputName(String),
    InputRoll(String),
    Output(String),
    FetchFailed,
    Roll,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub name: String,
    #[prop_or_default]
    pub roll: String,
    #[prop_or_default]
    pub output: String,
    pub onsignal: Callback<(String, DieData)>,
}

impl Component for Die {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Die {
            link,
            name: props.name,
            roll: props.roll,
            output: props.output,
            fetch_task: None,
            onsignal: props.onsignal,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::InputName(s) => {
                let former_name = self.name.clone();

                self.name = s;

                self.onsignal
                    .emit((former_name, DieData::new(&self.name, &self.roll)));
            }
            Msg::InputRoll(s) => {
                self.roll = s;

                self.onsignal
                    .emit((self.name.clone(), DieData::new(&self.name, &self.roll)));
            }
            Msg::Output(s) => {
                self.fetch_task = None;
                self.output = s
            }
            Msg::FetchFailed => {
                self.fetch_task = None;
                self.output = "Invalid input".to_string()
            }
            Msg::Roll => send_roll_request(self),
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.name = props.name;
        self.roll = props.roll;
        self.output = props.output;
        true
    }

    fn view(&self) -> Html {
        html! {
            <div class="die">
                <label>
                    <input
                    type="text",
                    class="dice-input-name",
                    value=&self.name,
                    placeholder="Name"
                    oninput=self.link.callback(|e: InputData| Msg::InputName(e.value)) />
                    <input
                    type="text",
                    class="dice-input-roll",
                    value=&self.roll,
                    placeholder="Roll"
                    oninput=self.link.callback(|e: InputData| Msg::InputRoll(e.value)) />
                    <button class="pure-button" onclick=self.link.callback(|_| Msg::Roll)>{ "Roll" }</button>
                    <p class="dice-output">{ &self.output }</p>
                </label>
            </div>
        }
    }
}
