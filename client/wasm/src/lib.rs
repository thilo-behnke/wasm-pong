mod utils;

use pong::game_field::{Field, Input, InputType};
use pong::game_object::game_object::{GameObject};
use pong::geom::shape::ShapeType;
use pong::pong::pong_events::{NoopPongEventWriter};
use pong::utils::utils::{DefaultLoggerFactory, Logger};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

extern crate serde_json;
extern crate web_sys;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into())
    }
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct GameObjectDTO {
    pub id: u16,
    pub x: f64,
    pub y: f64,
    pub orientation_x: f64,
    pub orientation_y: f64,
    pub vel_x: f64,
    pub vel_y: f64,
    pub shape_param_1: u16,
    pub shape_param_2: u16,
}

impl GameObjectDTO {
    pub fn from(obj: &Rc<RefCell<Box<dyn GameObject>>>) -> GameObjectDTO {
        let obj = RefCell::borrow(obj);

        let pos = obj.pos();
        let orientation = obj.orientation();
        let vel = obj.vel();
        let shape = obj.shape();
        return GameObjectDTO {
            id: obj.id(),
            x: pos.x,
            y: pos.y,
            orientation_x: orientation.x,
            orientation_y: orientation.y,
            vel_x: vel.x,
            vel_y: vel.y,
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
    pub player: u16,
}

impl InputDTO {
    pub fn to_input(&self) -> Input {
        return Input {
            input: self.input.to_input_type(),
            obj_id: self.obj_id,
            player: self.player,
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
        let field = Field::new(
            DefaultLoggerFactory::new(Box::new(WasmLogger::root())),
            NoopPongEventWriter::new(),
        );
        FieldWrapper { field }
    }

    pub fn width(&self) -> u16 {
        self.field.width
    }

    pub fn height(&self) -> u16 {
        self.field.height
    }

    pub fn tick(&mut self, inputs_js: &JsValue, ms_diff_js: JsValue) {
        let input_dtos: Vec<InputDTO> = inputs_js.into_serde().unwrap();
        let inputs = input_dtos
            .into_iter()
            .map(|i| i.to_input())
            .collect::<Vec<Input>>();
        let ms_diff = ms_diff_js.as_f64();
        self.field.tick(inputs, ms_diff.unwrap());
        // log!("{:?}", self.field.collisions);
    }

    pub fn objects(&self) -> String {
        let objs = self
            .field
            .objs()
            .into_iter()
            .map(|o| GameObjectDTO::from(o))
            .collect::<Vec<GameObjectDTO>>();
        let json = json!(objs);
        serde_json::to_string(&json).unwrap()
    }
}

#[derive(Clone)]
pub struct WasmLogger {
    name: String,
}

impl WasmLogger {
    pub fn root() -> WasmLogger {
        WasmLogger {
            name: String::from("root"),
        }
    }
}

impl Logger for WasmLogger {
    fn box_clone(&self) -> Box<dyn Logger> {
        Box::new(self.clone())
    }

    fn set_name(&mut self, name: &str) {
        self.name = String::from(name);
    }

    fn log(&self, msg: &str) {
        log!("[{}] {}", self.name, msg)
    }
}
