use crate::collision::collision::{Collision, CollisionDetector, CollisionHandler, CollisionRegistry, Collisions};
use crate::game_object::components::{DefaultGeomComp, DefaultPhysicsComp};
use crate::game_object::game_object::{DefaultGameObject, GameObject};
use crate::geom::geom::Vector;
use crate::geom::shape::{Circle, Rect};
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
            if *ball.obj.vel() == Vector::zero() {
                ball.obj.vel_mut().add(&Vector::new(-2.0, 0.))
            }
        }

        for player in self.players.iter_mut() {
            let input_opt = inputs.iter().find(|input| player.obj.id() == input.obj_id);
            if let None = input_opt {
                player.obj.vel_mut().y = 0.;
                continue;
            }
            let input = input_opt.unwrap();
            match input.input {
                InputType::UP => {
                    let updated_vel_y = (player.obj.vel().y + 1.).min(5.);
                    player.obj.vel_mut().y = updated_vel_y;
                }
                InputType::DOWN => {
                    let updated_vel_y = (player.obj.vel().y - 1.).max(-5.);
                    player.obj.vel_mut().y = updated_vel_y;
                }
            };
        }

        for player in self.players.iter_mut() {
            player.obj.update_pos()
        }
        for ball in self.balls.iter_mut() {
            ball.obj.update_pos()
        }

        let mut objs: Vec<&Box<dyn GameObject>> = vec![];
        objs.extend(
            self.players
                .iter()
                .map(|p| &p.obj)
                .collect::<Vec<&Box<dyn GameObject>>>(),
        );
        // objs.extend(
        //     self.balls
        //         .iter()
        //         .map(|b| &b.obj)
        //         .collect::<Vec<&Box<dyn GameObject>>>(),
        // );
        objs.extend(
            self.bounds.objs
                .iter()
                .collect::<Vec<&Box<dyn GameObject>>>()
        );
        let collision_detector = CollisionDetector::new();
        let collision_handler = CollisionHandler::new();
        self.collisions = collision_detector.detect_collisions(&objs);

        for ball in self.balls.iter_mut() {
            let collisions = self.collisions.get_collisions_by_id(ball.obj.id());
            if collisions.is_empty() {
                continue;
            }
            let other = match collisions[0] {
                Collision(obj_a_id, obj_b_id) if *obj_a_id == ball.obj.id() => {
                    objs.iter().find(|o| o.id() == *obj_b_id).unwrap()
                }
                collision => objs.iter().find(|o| o.id() == collision.0).unwrap(),
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

// #[derive(Clone, Debug, PartialEq)]
#[derive(Debug)]
pub struct Player {
    pub obj: Box<dyn GameObject>,
}

impl Player {
    pub fn new(id: u16, x: u16, y: u16, field: &Field) -> Player {
        Player {
            obj: Box::new(DefaultGameObject::new(
                id,
                Box::new(DefaultGeomComp::new(
                    Box::new(Rect::new(Vector { x: x as f64, y: y as f64 }, Vector::new(0., 1.), (field.width as f64) / 25., (field.height as f64) / 5.))
                )),
                Box::new(DefaultPhysicsComp::new(
                    Vector::zero(),
                    true
                ))
            ))
        }
    }
}

// #[derive(Clone, Debug, PartialEq)]
#[derive(Debug)]
pub struct Ball {
    pub obj: Box<dyn GameObject>,
}

impl Ball {
    pub fn new(id: u16, x: u16, y: u16, field: &Field) -> Ball {
        Ball {
            obj: Box::new(DefaultGameObject::new(
                id,
                Box::new(DefaultGeomComp::new(
                    Box::new(Circle::new(Vector { x: x as f64, y: y as f64 }, Vector::zero(), (field.width as f64) / 80.)
                ))),
                Box::new(DefaultPhysicsComp::new(
                    Vector::zero(),
                    false
                ))
            ))
        }
    }
}

#[derive(Debug)]
pub struct Bounds {
    pub objs: Vec<Box<dyn GameObject>>,
}

impl Bounds {
    pub fn new(width: u16, height: u16) -> Bounds {
        Bounds {
            objs: vec![
                // top
                Box::new(DefaultGameObject::new(
                    90,
                    Box::new(DefaultGeomComp::new(
                        Box::new(Rect::new(Vector {x: (width / 2) as f64, y: 0 as f64}, Vector::new(1., 0.), width as f64, 2.)
                    ))),
                    Box::new(DefaultPhysicsComp::new_static())
                )),
                // bottom
                Box::new(DefaultGameObject::new(
                    91,
                    Box::new(DefaultGeomComp::new(
                        Box::new(Rect::new(Vector {x: (width / 2) as f64, y: height as f64}, Vector::new(-1., 0.), width as f64, 2.)
                        ))),
                    Box::new(DefaultPhysicsComp::new_static())
                )),
                // left
                Box::new(DefaultGameObject::new(
                    92,
                    Box::new(DefaultGeomComp::new(
                        Box::new(Rect::new(Vector {x: 0 as f64, y: (height / 2) as f64}, Vector::new(0., 1.), 2., height as f64)
                        ))),
                    Box::new(DefaultPhysicsComp::new_static())
                )),
                // right
                Box::new(DefaultGameObject::new(
                    93,
                    Box::new(DefaultGeomComp::new(
                        Box::new(Rect::new(Vector {x: width as f64, y: (height / 2) as f64}, Vector::new(0., -1.), 2., height as f64)
                        ))),
                    Box::new(DefaultPhysicsComp::new_static())
                ))
            ],
        }
    }
}
