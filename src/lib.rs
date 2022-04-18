mod utils;

use serde::{Deserialize, Serialize};
use serde_json::json;
use wasm_bindgen::prelude::*;

extern crate serde_json;
extern crate web_sys;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub struct Field {
    pub width: u16,
    pub height: u16,
    players: Vec<Player>,
    balls: Vec<Ball>,
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    Rect = 0,
    Circle = 1,
}

#[derive(Debug, PartialEq, Eq)]
pub struct GameObject {
    pub id: u16,
    pub x: u16,
    pub y: u16,
    pub shape: Shape,
    pub shape_params: Vec<u16>,
}

#[wasm_bindgen]
#[repr(packed)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct GameObjectDTO {
    pub id: u16,
    pub x: u16,
    pub y: u16,
    pub shape: u16,
}

impl GameObjectDTO {
    pub fn from(obj: &GameObject) -> GameObjectDTO {
        return GameObjectDTO {
            id: obj.id,
            x: obj.x,
            y: obj.y,
            shape: match obj.shape_params[..] {
                [p1] => p1 << 8,
                [p1, p2] | [p1, p2, ..] => p1 << 8 | p2,
                _ => 0
            },
        };
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Player {
    pub obj: GameObject,
}

impl Player {
    pub fn new(id: u16, x: u16, y: u16, field: &Field) -> Player {
        Player {
            obj: GameObject {
                id,
                x,
                y,
                shape: Shape::Rect,
                shape_params: vec![field.width / 25, field.height / 5],
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Ball {
    pub obj: GameObject,
}

impl Ball {
    pub fn new(id: u16, x: u16, y: u16, field: &Field) -> Ball {
        Ball {
            obj: GameObject {
                id,
                x,
                y,
                shape: Shape::Circle,
                shape_params: vec![field.width / 50],
            },
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputType {
    UP,
    DOWN,
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Input {
    pub input: InputType,
    pub obj_id: u16,
}

#[wasm_bindgen]
impl Field {
    pub fn new() -> Field {
        let width = 800;
        let height = 600;

        let mut field = Field {
            width,
            height,
            players: vec![],
            balls: vec![]
        };

        field.add_player(0, 0 + width / 20, height / 2);
        field.add_player(1, width - width / 20, height / 2);
        field.add_ball(2, width / 2, height / 2);

        return field;
    }

    pub fn tick(&mut self, inputs_js: &JsValue) {
        let inputs: Vec<Input> = inputs_js.into_serde().unwrap();
        self.tick_inner(inputs);
    }

    pub fn objects(&self) -> *const GameObjectDTO {
        let mut objs = vec![];
        objs.append(
            &mut self
                .balls
                .iter()
                .map(|ball| GameObjectDTO::from(&ball.obj))
                .collect::<Vec<GameObjectDTO>>(),
        );
        objs.append(
            &mut self
                .players
                .iter()
                .map(|player| GameObjectDTO::from(&player.obj))
                .collect::<Vec<GameObjectDTO>>(),
        );
        objs.as_ptr()
    }

    pub fn get_state(&self) -> String {
        let json = json!(GameObjectDTO {
            shape: 0,
            x: 10,
            y: 10,
            id: 1
        });
        serde_json::to_string(&json).unwrap()
    }
}

impl Field {
    pub fn mock(width: u16, height: u16) -> Field {
        Field {
            width,
            height,
            players: vec![],
            balls: vec![]
        }
    }

    pub fn add_player(&mut self, id: u16, x: u16, y: u16) {
        self.players.push(Player::new(id, x, y, &self));
    }

    pub fn add_ball(&mut self, id: u16, x: u16, y: u16) {
        self.balls.push(Ball::new(id, x, y, &self));
    }

    pub fn tick_inner(&mut self, inputs: Vec<Input>) {
        for input in inputs.iter() {
            let obj_opt = self.players.iter_mut().find(|p| p.obj.id == input.obj_id);
            if let None = obj_opt {
                log!(
                    "Could not find player with id {} with players: {:?}",
                    input.obj_id,
                    self.players
                );
                continue;
            }
            let player = obj_opt.unwrap();
            let mut player_obj = &mut player.obj;
            let half_box = player_obj.shape_params[1] / 2;
            match input.input {
                InputType::UP => {
                    let shape_vert_bound = player_obj.y + half_box;
                    let out_of_bounds = shape_vert_bound + 5 > self.height;
                    if out_of_bounds {
                        player_obj.y = self.height - half_box
                    } else {
                        player_obj.y = player_obj.y + 5
                    }
                }
                InputType::DOWN => {
                    let shape_vert_bound = player_obj.y.checked_sub(half_box).map_or(0, |val| val);
                    let next_iter = shape_vert_bound.checked_sub(5);
                    if let Some(res) = next_iter {
                        player_obj.y = res + half_box;
                    } else {
                        player_obj.y = half_box;
                    }
                }
            };
        }
    }

    pub fn players(&self) -> &Vec<Player> {
        &self.players
    }
}
