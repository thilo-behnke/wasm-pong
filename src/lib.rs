mod utils;

use pong::collision::collision::{Collision, CollisionDetector};
use pong::game_field::{Field, Input, InputType};
use pong::game_object::game_object::GameObject;
use pong::geom::geom::Vector;
use pong::geom::shape::ShapeType;
use pong::utils::utils::Logger;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::cmp::{max, min};
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
#[repr(packed)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct GameObjectDTO {
    pub id: u16,
    pub x: u16,
    pub y: u16,
    pub shape_param_1: u16,
    pub shape_param_2: u16,
}

impl GameObjectDTO {
    pub fn from(obj: &Box<dyn GameObject>) -> GameObjectDTO {
        let pos = obj.pos();
        let shape = obj.shape();
        return GameObjectDTO {
            id: obj.id(),
            x: pos.x as u16,
            y: pos.y as u16,
            shape_param_1: match shape {
                ShapeType::Rect(_, width, _) => *width as u16,
                ShapeType::Circle(_, radius) => *radius as u16,
            },
            shape_param_2: match shape {
                ShapeType::Rect(_, _, height) => *height as u16,
                ShapeType::Circle(_, _) => 0,
            },
        };
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputTypeDTO {
    UP,
    DOWN,
}

impl InputTypeDTO {
    pub fn to_input_type(&self) -> InputType {
        match self {
            InputTypeDTO::UP => InputType::UP,
            InputTypeDTO::DOWN => InputType::DOWN,
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputDTO {
    pub input: InputTypeDTO,
    pub obj_id: u16,
}

impl InputDTO {
    pub fn to_input(&self) -> Input {
        return Input {
            input: self.input.to_input_type(),
            obj_id: self.obj_id,
        };
    }
}

#[wasm_bindgen]
pub struct FieldWrapper {
    field: Field,
}

#[wasm_bindgen]
impl FieldWrapper {
    pub fn new() -> FieldWrapper {
        let field = Field::new(Box::new(WasmLogger {}));
        FieldWrapper { field }
    }

    pub fn width(&self) -> u16 {
        self.field.width
    }

    pub fn height(&self) -> u16 {
        self.field.height
    }

    pub fn tick(&mut self, inputs_js: &JsValue) {
        let input_dtos: Vec<InputDTO> = inputs_js.into_serde().unwrap();
        let inputs = input_dtos
            .into_iter()
            .map(|i| i.to_input())
            .collect::<Vec<Input>>();
        // self.field.tick(inputs);
        // log!("{:?}", self.field.collisions);
    }

    pub fn objects(&self) -> *const GameObjectDTO {
        let mut objs = self.field.objs.borrow().iter().map(|o| GameObjectDTO::from(o)).collect::<Vec<GameObjectDTO>>();
        objs.as_ptr()
    }

    pub fn get_state(&self) -> String {
        let json = json!(GameObjectDTO {
            shape_param_1: 0,
            shape_param_2: 0,
            x: 10,
            y: 10,
            id: 1
        });
        serde_json::to_string(&json).unwrap()
    }
}

pub struct WasmLogger {}
impl Logger for WasmLogger {
    fn log(&self, msg: &str) {
        log!("{}", msg)
    }
}
