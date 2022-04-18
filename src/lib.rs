mod utils;

use std::cmp::{max, min};
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

#[derive(Debug, Clone, PartialEq)]
pub struct Vector {
    pub x: i32,
    pub y: i32
}

impl Vector {
    pub fn zero() -> Vector {
        Vector {x: 0, y: 0}
    }

    pub fn normalize(&mut self) {
        let length = self.len();
        self.x /= length;
        self.y /= length;
    }

    pub fn len(&self) -> i32 {
        let distance = self.x.pow(2) + self.y.pow(2);
        return (distance as f32).sqrt() as i32;
    }
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Debug, PartialEq)]
pub enum Shape {
    Rect = 0,
    Circle = 1,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GameObject {
    pub id: u16,
    pub x: u16,
    pub y: u16,
    pub shape: Shape,
    pub shape_params: Vec<u16>,
    pub vel: Vector
}

impl GameObject {
    pub fn update_pos(&mut self, field_width: u16, field_height: u16) {
        let updated_x = self.x.wrapping_add(self.vel.x as u16);
        let updated_y = self.y.wrapping_add(self.vel.y as u16);

        let updated_bounding_box = self.bounding_box(updated_x, updated_y);
        if updated_bounding_box.points.iter().any(|p| p.x < 0 || p.x > field_width as i16 || p.y < 0 || p.y > field_height as i16) {
            return;
        }
        self.x = updated_x;
        self.y = updated_y;
    }

    pub fn set_vel_x(&mut self, x: i32) {
        self.vel.x = x
    }

    pub fn set_vel_y(&mut self, y: i32) {
        self.vel.y = y
    }

    fn bounding_box(&self, x: u16, y: u16) -> BoundingBox {
        match self.shape {
            Shape::Rect => {
                BoundingBox::create(x, y, self.shape_params[0], self.shape_params[1])
            },
            Shape::Circle => {
                BoundingBox::create(x, y, self.shape_params[0] * 2, self.shape_params[0] * 2)
            }
        }
    }
}

struct BoundingBox {
    pub points: Vec<Pos>
}

impl BoundingBox {
    pub fn create(center_x: u16, center_y: u16, width: u16, height: u16) -> BoundingBox {
        let points = vec![
            Pos {x: center_x as i16 - (width / 2) as i16, y: center_y as i16 + (height / 2) as i16}, // top left
            Pos {x: center_x as i16 + (width / 2) as i16, y: center_y as i16 + (height / 2) as i16}, // top right
            Pos {x: center_x as i16 - (width / 2) as i16, y: center_y as i16 - (height / 2) as i16}, // bottom left
            Pos {x: center_x as i16 + (width / 2) as i16, y: center_y as i16 - (height / 2) as i16}, // bottom right
        ];
        BoundingBox {
            points
        }
    }
}

struct Pos {
    x: i16,
    y: i16
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

#[derive(Clone, Debug, PartialEq)]
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
                vel: Vector::zero()
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
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
                shape_params: vec![field.width / 80],
                vel: Vector::zero()
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
        let ball = Ball::new(id, x, y, &self);
        self.balls.push(ball);
    }

    pub fn tick_inner(&mut self, inputs: Vec<Input>) {
        for ball in self.balls.iter_mut() {
            if ball.obj.vel == Vector::zero() {
                ball.obj.set_vel_x(-1)
            }
        }

        for player in self.players.iter_mut() {
            let input_opt = inputs.iter().find(|input| player.obj.id == input.obj_id);
            if let None = input_opt {
                player.obj.set_vel_y(0);
                continue;
            }
            let input = input_opt.unwrap();
            match input.input {
                InputType::UP => {
                    player.obj.vel.y = min(player.obj.vel.y + 1, 5);
                }
                InputType::DOWN => {
                    player.obj.vel.y = max(player.obj.vel.y - 1, -5);
                }
            };
        }

        for player in self.players.iter_mut() {
            player.obj.update_pos(self.width, self.height)
        }
        for ball in self.balls.iter_mut() {
            ball.obj.update_pos(self.width, self.height)
        }
    }

    pub fn players(&self) -> &Vec<Player> {
        &self.players
    }
}
