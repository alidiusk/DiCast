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
    pub current: DieData,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum Msg {
    UpdateDie(String, DieData),
    SelectDie(String),
    NewDie,
    DeleteDie,
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

                let current = dice[0].clone();

                State { dice, current }
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

                if *die == self.state.current {
                    self.state.current = data.clone();
                }

                *die = data;
            }
            Msg::SelectDie(s) => {
                // we know that there has to be a die with that name here
                // the event only generates names of existing die
                let die = self
                    .state
                    .dice
                    .iter()
                    .find(|d| d.name == s)
                    .unwrap()
                    .clone();
                self.state.current = die;
            }
            Msg::NewDie => {
                let die = DieData::new("", "");
                self.state.dice.push(die.clone());
                self.state.current = die;
            }
            Msg::DeleteDie => {
                let current = self.state.current.clone();
                self.state.dice.retain(|d| d != &current);
                self.state.current = self
                    .state
                    .dice
                    .last()
                    .unwrap_or(&DieData::new("", ""))
                    .clone();
            }
        }

        self.storage.store(KEY, Json(&self.state));

        true
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
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
                    <select id="dice-select" name="dice" value=&self.state.current.name
                    onchange=self.link.callback(|cd: ChangeData| {
                        match cd {
                            ChangeData::Select(s) => Msg::SelectDie(s.value()),
                            _ => unreachable!(),
                        }
                    }) >
                    {
                        self.state.dice.iter().map(|d| html! {
                            <option value=&d.name>{ &d.name }</option>
                        })
                        .collect::<Html>()
                    }
                    </select>
                    <button id="new-die-button" class="pure-button pure-button-primary"
                    onclick=self.link.callback(|_| Msg::NewDie)>{ "New die" }</button>
                    <button id="delete-die-button" class="pure-button pure-button-primary"
                    onclick=self.link.callback(|_| Msg::DeleteDie)>{ "Delete current die" }</button>
                    <br/><br/>
                    <div id="dice">
                        <Die
                        name=&self.state.current.name
                        roll=&self.state.current.roll
                        output={ "".to_string() },
                        onsignal=self.link.callback(|(name, new_die)| Msg::UpdateDie(name, new_die)) />
                    </div>
                </div>
            </div>
        }
    }
}
