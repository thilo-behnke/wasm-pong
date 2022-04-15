mod utils;

use std::fmt;
use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

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
    width: u32,
    height: u32,
    players: Vec<Player>,
    balls: Vec<Ball>,
}

#[wasm_bindgen]
#[repr(packed)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameObject {
    pub id: u8,
    pub x: u16,
    pub y: u16,
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Player {
    pub obj: GameObject,
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ball {
    pub obj: GameObject,
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
    pub obj_id: u8,
}

#[wasm_bindgen]
impl Field {
    pub fn new() -> Field {
        Field {
            width: 0,
            height: 0,
            players: vec![Player {
                obj: GameObject { id: 1, x: 0, y: 0 },
            }],
            balls: vec![],
        }
    }
    pub fn tick(&self, inputs_js: &JsValue) {
        let inputs: Vec<Input> = inputs_js.into_serde().unwrap();
        log!("### tick start ###");
        for input in inputs.iter() {
            let mut obj_opt = self.players.iter().find(|p| p.obj.id == input.obj_id);
            if let None = obj_opt {
                log!("Could not find player with id {}", input.obj_id);
                continue;
            }
            let obj = obj_opt.unwrap();
            match input.input {
                InputType::UP => obj.obj.y + 1,
                InputType::DOWN => obj.obj.y - 1,
            };
        }
        log!("### tick end ###");
    }

    pub fn objects(&self) -> *const GameObject {
        let mut objs = vec![];
        objs.append(
            &mut self
                .balls
                .iter()
                .map(|ball| ball.obj)
                .collect::<Vec<GameObject>>(),
        );
        objs.append(
            &mut self
                .players
                .iter()
                .map(|player| player.obj)
                .collect::<Vec<GameObject>>(),
        );
        objs.as_ptr()
    }
}
