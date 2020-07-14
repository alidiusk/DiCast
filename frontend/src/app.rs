use serde_derive::{Deserialize, Serialize};
use yew::format::Json;
use yew::prelude::*;
use yew::services::storage::{Area, StorageService};

use crate::die::Die;

const KEY: &str = "state";

pub struct App {
    link: ComponentLink<Self>,
    storage: StorageService,
    state: State,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DieData {
    pub name: String,
    pub roll: String,
}

impl DieData {
    pub fn new(name: &str, roll: &str) -> Self {
        DieData {
            name: name.to_string(),
            roll: roll.to_string(),
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct State {
    pub dice: Vec<DieData>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum Msg {
    UpdateDie(String, DieData),
    NewDie,
    DeleteDie(String),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        // make this optional in the future
        let storage = StorageService::new(Area::Local).expect("storage was disabled by the user.");
        let state = {
            if let Json(Ok(state)) = storage.restore(KEY) {
                state
            } else {
                let dice = vec![DieData::new("default", "3x 3d20 *2 +1 s2")];

                State { dice }
            }
        };

        App {
            link,
            storage,
            state,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        // refactor. very ugly
        match msg {
            Msg::UpdateDie(name, data) => {
                let die = self.state.dice.iter_mut().find(|d| d.name == name).unwrap();

                *die = data;
            }
            Msg::NewDie => {
                let die = DieData::new("", "");
                self.state.dice.push(die);
            }
            Msg::DeleteDie(s) => {
                self.state.dice.retain(|d| d.name != s);
            }
        }

        self.storage.store(KEY, Json(&self.state));

        true
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        // refactor this concurrent code at some point
        html! {
        <div class="pure-g">
            <div id="container" class="pure-u-1">
                <h1>{ "Dice Roller" }</h1>
                <p>
                <b><u>{ "Syntax:"}</u></b>{ "{#x}{#}d{#}{*//#}{+/-#}{s#}" }<br/><br/>
                {
                "[Number of rolls, number of dice, number of sides, multiplier,
                modifier, number of dice to drop.]"
                }
                </p>
                <button id="new-die-button" class="pure-button"
                onclick=self.link.callback(|_| Msg::NewDie)>{ "New die" }</button>
                <br/><br/>
                <div id="dice">
                {
                    (0..self.state.dice.len()).map(|index| {
                        let dice_name = self.state.dice[index].name.clone();

                        html! {
                            <div class="die-row">
                            <button
                            class="delete-die-button pure-button button-red"
                            onclick=self.link.callback(move |_|
                                Msg::DeleteDie((&dice_name).to_string()))>
                            { "Delete" }</button>

                            <Die
                            name=&self.state.dice[index].name
                            roll=&self.state.dice[index].roll
                            output={ "".to_string() },
                            onsignal=self.link.callback(|(name, new_die)|
                                Msg::UpdateDie(name, new_die)) />
                            </div>
                    }}).collect::<Html>()
                }
                </div>
            </div>
        </div>
        }
    }
}
