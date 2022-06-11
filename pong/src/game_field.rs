use crate::collision::collision::{
    Collision, CollisionDetector, CollisionGroup, CollisionHandler, CollisionRegistry, Collisions,
};
use crate::event::event::Event;
use crate::game_object::components::{DefaultGeomComp, DefaultPhysicsComp};
use crate::game_object::game_object::{DefaultGameObject, GameObject};
use crate::geom::geom::Vector;
use crate::geom::shape::{Shape, ShapeType};
use crate::pong::pong_collisions::{
    handle_ball_bounds_collision, handle_player_ball_collision, handle_player_bound_collision,
};
use crate::pong::pong_events::{
    DefaultPongEventWriter, GameObjUpdate, NoopPongEventWriter, PongEventType, PongEventWriter,
};
use crate::utils::utils::{DefaultLoggerFactory, Logger, LoggerFactory, NoopLogger};
use std::borrow::{Borrow, BorrowMut};
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputType {
    UP,
    DOWN,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Input {
    pub input: InputType,
    pub obj_id: u16,
    pub player: u16,
}

pub struct Field {
    pub logger_factory: Box<dyn LoggerFactory>,
    pub logger: Box<dyn Logger>,
    pub width: u16,
    pub height: u16,
    pub collisions: Box<dyn CollisionRegistry>,
    objs: Vec<Rc<RefCell<Box<dyn GameObject>>>>,
    event_writer: Box<dyn PongEventWriter>,
    collision_detector: CollisionDetector,
    collision_handler: CollisionHandler,
}

impl Field {
    pub fn new(
        logger_factory: Box<dyn LoggerFactory>,
        event_writer: Box<dyn PongEventWriter>,
    ) -> Field {
        let width = 800;
        let height = 600;

        let mut field = Field {
            logger: logger_factory.get("game_field"),
            width,
            height,
            objs: DefaultGameObject::bounds(width, height)
                .into_iter()
                .map(|b| Rc::new(RefCell::new(b.inner())))
                .collect(),
            collisions: Box::new(Collisions::new(vec![])),
            collision_detector: CollisionDetector::new(&logger_factory),
            collision_handler: CollisionHandler::new(&logger_factory),
            event_writer,
            logger_factory,
        };

        field.add_player(0, 0 + width / 20, height / 2);
        field.add_player(1, width - width / 20, height / 2);
        field.add_ball(2, width / 2, height / 2);

        field.collision_handler.register(
            (String::from("ball"), String::from("player")),
            handle_player_ball_collision,
        );

        field.collision_handler.register(
            (String::from("ball"), String::from("bound")),
            handle_ball_bounds_collision,
        );

        field.collision_handler.register(
            (String::from("player"), String::from("bound")),
            handle_player_bound_collision,
        );

        field.collision_detector.set_groups(vec![
            CollisionGroup(String::from("player"), String::from("ball")),
            CollisionGroup(String::from("player"), String::from("bound")),
            CollisionGroup(String::from("ball"), String::from("bound")),
        ]);

        return field;
    }

    pub fn mock(width: u16, height: u16) -> Field {
        let logger_factory = DefaultLoggerFactory::new(Box::new(NoopLogger {}));
        let event_writer = NoopPongEventWriter::new();
        Field {
            logger: logger_factory.get("game_field"),
            width,
            height,
            objs: DefaultGameObject::bounds(width, height)
                .into_iter()
                .map(|b| Rc::new(RefCell::new(b.inner())))
                .collect(),
            collisions: Box::new(Collisions::new(vec![])),
            collision_detector: CollisionDetector::new(&logger_factory),
            collision_handler: CollisionHandler::new(&logger_factory),
            event_writer,
            logger_factory,
        }
    }

    pub fn add_player(&mut self, id: u16, x: u16, y: u16) {
        let player = DefaultGameObject::player(id, x, y, &self);
        self.objs.push(Rc::new(RefCell::new(player)));
    }

    pub fn add_ball(&mut self, id: u16, x: u16, y: u16) {
        let ball = DefaultGameObject::ball(id, x, y, &self);
        self.objs.push(Rc::new(RefCell::new(ball)));
    }

    pub fn tick(&mut self, inputs: Vec<Input>, ms_diff: f64) {
        for obj in self.objs.iter() {
            let mut obj_mut = RefCell::borrow_mut(obj);
            if obj_mut.obj_type() != "ball" {
                continue;
            }
            if *obj_mut.vel() == Vector::zero() {
                obj_mut.vel_mut().add(&Vector::new(-300., 0.))
            }
        }

        {
            for obj in self.objs.iter() {
                let mut obj_mut = RefCell::borrow_mut(obj);
                if obj_mut.obj_type() != "player" {
                    continue;
                }
                let input_opt = inputs.iter().find(|i| i.obj_id == obj_mut.id());
                if let None = input_opt {
                    obj_mut.vel_mut().y = 0.;
                    continue;
                }
                let input = input_opt.unwrap();
                match input.input {
                    InputType::UP => {
                        let updated_vel_y = (obj_mut.vel().y + 30.).min(1000.);
                        obj_mut.vel_mut().y = updated_vel_y;
                    }
                    InputType::DOWN => {
                        let updated_vel_y = (obj_mut.vel().y - 30.).max(-1000.);
                        obj_mut.vel_mut().y = updated_vel_y;
                    }
                };
            }
        }

        {
            for obj in self.objs.iter() {
                let mut obj_mut = RefCell::borrow_mut(obj);
                obj_mut.update_pos(ms_diff);
            }
        }

        let collisions = self.get_collisions();

        let collision_handler = &self.collision_handler;
        let registered_collisions = collisions.get_collisions();
        self.logger.log(&*format!(
            "Found {} collisions: {:?}",
            registered_collisions.len(),
            registered_collisions
        ));
        for collision in registered_collisions.iter() {
            let objs = &self.objs;
            let obj_a = objs
                .iter()
                .find(|o| RefCell::borrow(o).id() == collision.0)
                .unwrap()
                .clone();
            let obj_b = objs
                .iter()
                .find(|o| RefCell::borrow(o).id() == collision.1)
                .unwrap()
                .clone();
            collision_handler.handle(obj_a, obj_b);
        }

        {
            for obj in self.objs.iter().filter(|o| RefCell::borrow(o).is_dirty()) {
                let mut obj = RefCell::borrow_mut(obj);
                self.event_writer
                    .write(PongEventType::GameObjUpdate(GameObjUpdate {
                        obj_id: &obj.id().to_string(),
                        vel: obj.vel(),
                        orientation: obj.orientation(),
                        pos: obj.pos(),
                    }));
                obj.set_dirty(false);
            }
        }
    }

    fn get_collisions(&self) -> Box<dyn CollisionRegistry> {
        let objs = self.objs.iter().map(|o| o.clone()).collect();
        self.collision_detector.detect_collisions(objs)
    }

    pub fn objs(&self) -> Vec<&Rc<RefCell<Box<dyn GameObject>>>> {
        self.objs.iter().collect()
    }
}

impl DefaultGameObject {
    pub fn player(id: u16, x: u16, y: u16, field: &Field) -> Box<dyn GameObject> {
        Box::new(DefaultGameObject::new(
            id,
            "player".to_string(),
            Box::new(DefaultGeomComp::new(Shape::rect(
                Vector {
                    x: x as f64,
                    y: y as f64,
                },
                Vector::new(0., 1.),
                (field.width as f64) / 25.,
                (field.height as f64) / 5.,
            ))),
            Box::new(DefaultPhysicsComp::new(Vector::zero(), true)),
        ))
    }
}

impl DefaultGameObject {
    pub fn ball(id: u16, x: u16, y: u16, field: &Field) -> Box<dyn GameObject> {
        Box::new(DefaultGameObject::new(
            id,
            "ball".to_string(),
            Box::new(DefaultGeomComp::new(Shape::circle(
                Vector {
                    x: x as f64,
                    y: y as f64,
                },
                Vector::zero(),
                (field.width as f64) / 80.,
            ))),
            Box::new(DefaultPhysicsComp::new(Vector::zero(), false)),
        ))
    }
}

impl DefaultGameObject {
    pub fn bounds(width: u16, height: u16) -> Vec<Bounds> {
        let bounds = vec![
            Bounds(
                Bound::BOTTOM,
                Box::new(DefaultGameObject::new(
                    90,
                    "bound".to_string(),
                    Box::new(DefaultGeomComp::new(Shape::rect(
                        Vector {
                            x: (width / 2) as f64,
                            y: 0 as f64,
                        },
                        Vector::new(1., 0.),
                        width as f64,
                        2.,
                    ))),
                    Box::new(DefaultPhysicsComp::new_static()),
                )),
            ),
            Bounds(
                Bound::TOP,
                Box::new(DefaultGameObject::new(
                    91,
                    "bound".to_string(),
                    Box::new(DefaultGeomComp::new(Shape::rect(
                        Vector {
                            x: (width / 2) as f64,
                            y: height as f64,
                        },
                        Vector::new(-1., 0.),
                        width as f64,
                        2.,
                    ))),
                    Box::new(DefaultPhysicsComp::new_static()),
                )),
            ),
            Bounds(
                Bound::LEFT,
                Box::new(DefaultGameObject::new(
                    92,
                    "bound".to_string(),
                    Box::new(DefaultGeomComp::new(Shape::rect(
                        Vector {
                            x: 0 as f64,
                            y: (height / 2) as f64,
                        },
                        Vector::new(0., 1.),
                        2.,
                        height as f64,
                    ))),
                    Box::new(DefaultPhysicsComp::new_static()),
                )),
            ),
            Bounds(
                Bound::RIGHT,
                Box::new(DefaultGameObject::new(
                    93,
                    "bound".to_string(),
                    Box::new(DefaultGeomComp::new(Shape::rect(
                        Vector {
                            x: width as f64,
                            y: (height / 2) as f64,
                        },
                        Vector::new(0., -1.),
                        2.,
                        height as f64,
                    ))),
                    Box::new(DefaultPhysicsComp::new_static()),
                )),
            ),
        ];
        bounds
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Bound {
    TOP,
    RIGHT,
    BOTTOM,
    LEFT,
}

pub struct Bounds(pub Bound, pub Box<dyn GameObject>);

impl Bounds {
    pub fn inner(self) -> Box<dyn GameObject> {
        self.1
    }
}
