use crate::collision::collision::{Collision, CollisionDetector, CollisionHandler, CollisionRegistry, Collisions};
use crate::game_object::game_object::{GameObject, ShapeType};
use crate::geom::geom::Vector;
use crate::utils::utils::{Logger, NoopLogger};

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
    pub logger: Box<dyn Logger>,
    pub width: u16,
    pub height: u16,
    pub players: Vec<Player>,
    pub balls: Vec<Ball>,
    pub bounds: Bounds,
    pub collisions: Box<dyn CollisionRegistry>
}

impl Field {
    pub fn new(logger: Box<dyn Logger>) -> Field {
        let width = 800;
        let height = 600;

        let mut field = Field {
            logger,
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
            logger: Box::new(NoopLogger {}),
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
            if ball.obj.physics.vel() == Vector::zero() {
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
                    let updated_vel_y = (player.obj.vel.y + 1.).min(5.);
                    player.obj.set_vel_y(updated_vel_y);
                }
                InputType::DOWN => {
                    let updated_vel_y = (player.obj.vel.y - 1.).max(-5.);
                    player.obj.set_vel_y(updated_vel_y);
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
        let collision_handler = CollisionHandler::new();
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


            self.logger.log("### BEFORE COLLISION ###");
            self.logger.log(&*format!("{:?}", ball.obj));
            self.logger.log(&*format!("{:?}", other));
            collision_handler.handle(&mut ball.obj, other);
            self.logger.log("### AFTER COLLISION ###");
            self.logger.log(&*format!("{:?}", ball.obj));
            self.logger.log(&*format!("{:?}", other));
            self.logger.log("### DONE ###");
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
                orientation: Vector::new(0., 1.),
                shape: ShapeType::Rect,
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
                orientation: Vector::zero(),
                shape: ShapeType::Circle,
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
                // top
                GameObject {
                    id: 90,
                    pos: Vector {x: (width / 2) as f64, y: 0 as f64},
                    orientation: Vector::new(1., 0.),
                    shape: ShapeType::Rect,
                    shape_params: vec![width, 2],
                    is_static: true,
                    vel: Vector::zero(),
                },
                // bottom
                GameObject {
                    id: 91,
                    pos: Vector {x: (width / 2) as f64, y: height as f64},
                    orientation: Vector::new(-1., 0.),
                    shape: ShapeType::Rect,
                    shape_params: vec![width, 2],
                    is_static: true,
                    vel: Vector::zero(),
                },
                // left
                GameObject {
                    id: 92,
                    pos: Vector {x: 0 as f64, y: (height / 2) as f64},
                    orientation: Vector::new(0., 1.),
                    shape: ShapeType::Rect,
                    shape_params: vec![2, height],
                    is_static: true,
                    vel: Vector::zero(),
                },
                // right
                GameObject {
                    id: 93,
                    pos: Vector {x: width as f64, y: (height / 2) as f64},
                    orientation: Vector::new(0., -1.),
                    shape: ShapeType::Rect,
                    shape_params: vec![2, height],
                    is_static: true,
                    vel: Vector::zero(),
                },
            ],
        }
    }
}
