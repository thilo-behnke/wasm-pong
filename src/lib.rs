mod utils;
mod geom;
mod game_object;
mod collision;

use std::borrow::BorrowMut;
use std::cmp::{max, min};
use serde::{Deserialize, Serialize};
use serde_json::json;
use wasm_bindgen::prelude::*;
use crate::game_object::{GameObject, Shape};
use crate::geom::geom::{BoundingBox, Vector};

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
pub struct Field {
    pub width: u16,
    pub height: u16,
    players: Vec<Player>,
    balls: Vec<Ball>,
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

        let mut collisions = self.detect_collisions();
        for mut collision in collisions.iter_mut() {
            let players = &self.players;
            let balls = &self.balls;
            // TODO: Find obj by id.
            // if !collision.obj_a.is_static {
            //     collision.obj_a.vel.invert();
            // }
            // if !collision.obj_b.is_static {
            //     collision.obj_b.vel.invert();
            // }
        }
    }

    fn detect_collisions(&mut self) -> Vec<Collision> {
        let balls = self.balls();
        let players = self.players();

        let mut collisions = vec![];
        // for ball in balls.into_iter() {
        //     let collision_opt = players.into_iter().find(|p| p.obj.bounding_box().overlaps(&ball.obj.bounding_box()));
        //     if let None = collision_opt {
        //         continue;
        //     }
        //     let player = collision_opt.unwrap();
        //     // TODO: This can cause multiple mutable refs of the same player/ball object and therefore does not compile.
        //     collisions.push(Collision {obj_a: player.obj.id, obj_b: ball.obj.id});
        // }
        collisions
    }

    pub fn players(&self) -> Vec<&Player> {
        self.players.iter().collect()
    }

    pub fn balls(&self) -> Vec<&Ball> {
        self.balls.iter().collect()
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
                vel: Vector::zero(),
                is_static: true
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
                vel: Vector::zero(),
                is_static: false
            },
        }
    }
}

#[derive(Debug)]
pub struct Collision {
    obj_a: u16,
    obj_b: u16
}
