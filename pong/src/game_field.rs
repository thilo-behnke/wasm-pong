use std::f64::consts::{FRAC_PI_2, FRAC_PI_4};
use crate::collision::collision::{Collision, CollisionDetector, CollisionRegistry, Collisions};
use crate::game_object::game_object::{GameObject, Shape};
use crate::geom::geom::Vector;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputType {
    UP,
    DOWN,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Input {
    pub input: InputType,
    pub obj_id: u16,
}

pub struct Field {
    pub width: u16,
    pub height: u16,
    pub players: Vec<Player>,
    pub balls: Vec<Ball>,
    pub bounds: Bounds,
    pub collisions: Box<dyn CollisionRegistry>
}

impl Field {
    pub fn new() -> Field {
        let width = 800;
        let height = 600;

        let mut field = Field {
            width,
            height,
            players: vec![],
            balls: vec![],
            bounds: Bounds::new(width, height),
            collisions: Box::new(Collisions::new(vec![]))
        };

        field.add_player(0, 0 + width / 20, height / 2);
        field.add_player(1, width - width / 20, height / 2);
        field.add_ball(2, width / 2, height / 2);

        return field;
    }

    pub fn mock(width: u16, height: u16) -> Field {
        Field {
            width,
            height,
            players: vec![],
            balls: vec![],
            bounds: Bounds::new(width, height),
            collisions: Box::new(Collisions::new(vec![]))
        }
    }

    pub fn add_player(&mut self, id: u16, x: u16, y: u16) {
        self.players.push(Player::new(id, x, y, &self));
    }

    pub fn add_ball(&mut self, id: u16, x: u16, y: u16) {
        let ball = Ball::new(id, x, y, &self);
        self.balls.push(ball);
    }

    pub fn tick(&mut self, inputs: Vec<Input>) {
        for ball in self.balls.iter_mut() {
            if ball.obj.vel == Vector::zero() {
                ball.obj.set_vel_x(-2.)
            }
        }

        for player in self.players.iter_mut() {
            let input_opt = inputs.iter().find(|input| player.obj.id == input.obj_id);
            if let None = input_opt {
                player.obj.set_vel_y(0.);
                continue;
            }
            let input = input_opt.unwrap();
            match input.input {
                InputType::UP => {
                    player.obj.vel.y = (player.obj.vel.y + 1.).min(5.);
                }
                InputType::DOWN => {
                    player.obj.vel.y = (player.obj.vel.y - 1.).max(-5.);
                }
            };
        }

        for player in self.players.iter_mut() {
            player.obj.update_pos()
        }
        for ball in self.balls.iter_mut() {
            ball.obj.update_pos()
        }

        let mut objs: Vec<GameObject> = vec![];
        objs.extend(
            self.players
                .clone()
                .into_iter()
                .map(|p| p.obj)
                .collect::<Vec<GameObject>>(),
        );
        objs.extend(
            self.balls
                .clone()
                .into_iter()
                .map(|b| b.obj)
                .collect::<Vec<GameObject>>(),
        );
        objs.extend(
            self.bounds.objs
                .clone()
                .into_iter()
                .collect::<Vec<GameObject>>()
        );
        let collision_detector = CollisionDetector::new();
        self.collisions = collision_detector.detect_collisions(objs.iter().collect());

        for ball in self.balls.iter_mut() {
            let collisions = self.collisions.get_collisions_by_id(ball.obj.id);
            if collisions.is_empty() {
                continue;
            }
            let other = match collisions[0] {
                Collision(obj_a_id, obj_b_id) if *obj_a_id == ball.obj.id => {
                    objs.iter().find(|o| o.id == *obj_b_id).unwrap()
                }
                collision => objs.iter().find(|o| o.id == collision.0).unwrap(),
            };
            ball.obj.vel.add(&other.vel);
            ball.obj.vel.invert();

            // let dot = ball.obj.vel.dot(&other.vel);
            // if dot == 0. {
            //     ball.obj.vel.invert();
            // } else {
            //     let angle = ball.obj.vel.angle(&other.vel);
            //     ball.obj.vel.rotate(FRAC_PI_2 - angle);
            //     ball.obj.vel.invert();
            // }
        }
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
                pos: Vector {x: x as f64, y: y as f64},
                shape: Shape::Rect,
                shape_params: vec![field.width / 25, field.height / 5],
                vel: Vector::zero(),
                is_static: true,
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
                pos: Vector {x: x as f64, y: y as f64},
                shape: Shape::Circle,
                shape_params: vec![field.width / 80],
                vel: Vector::zero(),
                is_static: false,
            },
        }
    }
}

#[derive(Debug)]
pub struct Bounds {
    pub objs: Vec<GameObject>,
}

impl Bounds {
    pub fn new(width: u16, height: u16) -> Bounds {
        Bounds {
            objs: vec![
                GameObject {
                    id: 90,
                    pos: Vector {x: (width / 2) as f64, y: 0 as f64},
                    shape: Shape::Rect,
                    shape_params: vec![width, 2],
                    is_static: true,
                    vel: Vector::zero(),
                },
                // bottom
                GameObject {
                    id: 91,
                    pos: Vector {x: (width / 2) as f64, y: height as f64},
                    shape: Shape::Rect,
                    shape_params: vec![width, 2],
                    is_static: true,
                    vel: Vector::zero(),
                },
                // left
                GameObject {
                    id: 92,
                    pos: Vector {x: 0 as f64, y: (height / 2) as f64},
                    shape: Shape::Rect,
                    shape_params: vec![2, height],
                    is_static: true,
                    vel: Vector::zero(),
                },
                // right
                GameObject {
                    id: 93,
                    pos: Vector {x: width as f64, y: (height / 2) as f64},
                    shape: Shape::Rect,
                    shape_params: vec![2, height],
                    is_static: true,
                    vel: Vector::zero(),
                },
            ],
        }
    }
}
