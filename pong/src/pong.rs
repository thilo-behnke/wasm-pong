pub mod pong_collisions {
    use std::cell::{RefCell, RefMut};
    use std::ops::Add;
    use std::rc::Rc;
    use crate::game_object::game_object::GameObject;
    use crate::geom::geom::Vector;
    use crate::geom::shape::ShapeType;

    pub fn handle_player_ball_collision(
        ball: Rc<RefCell<Box<dyn GameObject>>>,
        player: Rc<RefCell<Box<dyn GameObject>>>,
    ) {
        // reflect
        let mut ball = RefCell::borrow_mut(&ball);
        let player = player.borrow();
        ball.vel_mut().reflect(&player.orientation());
        // use vel of player obj
        if *player.vel() != Vector::zero() {
            let mut adjusted = player.vel().clone();
            adjusted.normalize();
            ball.vel_mut().add(&adjusted);
        }
        // move out of collision
        let mut b_to_a = ball.pos().clone();
        b_to_a.sub(&player.pos());
        b_to_a.normalize();
        ball.pos_mut().add(&b_to_a);
    }

    pub fn handle_ball_bounds_collision(
        ball: Rc<RefCell<Box<dyn GameObject>>>,
        bound: Rc<RefCell<Box<dyn GameObject>>>,
    ) {
        let mut ball = RefCell::borrow_mut(&ball);
        let bound = RefCell::borrow(&bound);
        ball.vel_mut().reflect(&bound.orientation());
    }

    pub fn handle_player_bound_collision(
        player: Rc<RefCell<Box<dyn GameObject>>>,
        bound: Rc<RefCell<Box<dyn GameObject>>>,
    ) {
        let mut player = RefCell::borrow_mut(&player);
        let bound = RefCell::borrow(&bound);
        let shape = player.shape().clone();
        let player_orientation = player.orientation().clone();
        let height = match shape {
            ShapeType::Rect(_, _, height) => height.clone() / 2.,
            ShapeType::Circle(_, radius) => radius,
        };
        let mut perpendicular = player_orientation.get_opposing_orthogonal(bound.orientation());
        perpendicular.y *= (height + 1.);
        let mut new_pos = bound.pos().clone();
        new_pos.add(&perpendicular);
        let player_pos = player.pos_mut();
        player_pos.y = new_pos.y;
    }
}

pub mod pong_events {
    use serde_json::json;
    use serde::{Serialize};
    use crate::event::event::{Event, EventWriter};
    use crate::geom::geom::Vector;

    #[derive(Serialize)]
    pub enum PongEventType {
        GameObjUpdate(GameObjUpdate)
    }

    #[derive(Serialize)]
    pub struct GameObjUpdate {
        pub obj_id: String,
        pub pos: Vector,
        pub vel: Vector,
        pub orientation: Vector
    }

    pub trait PongEventWriter {
        fn write(&self, event: PongEventType) -> std::io::Result<()>;
    }

    pub struct DefaultPongEventWriter {
        writer: EventWriter
    }

    impl PongEventWriter for DefaultPongEventWriter {
        fn write(&self, event: PongEventType) -> std::io::Result<()> {
            let out_event = match event {
                PongEventType::GameObjUpdate(ref update) => {
                    Event {
                        topic: String::from("obj_update"),
                        key: update.obj_id.clone(),
                        msg: serde_json::to_string(&event).unwrap()
                    }
                }
            };
            self.writer.write(out_event)
        }
    }

    pub struct NoopPongEventWriter {}
    impl NoopPongEventWriter {
        pub fn new() -> Box<dyn PongEventWriter> {
            Box::new(DefaultPongEventWriter {
                writer: EventWriter::noop()
            })
        }
    }

    impl DefaultPongEventWriter {
        pub fn new() -> Box<dyn PongEventWriter> {
            Box::new(DefaultPongEventWriter {
                writer: EventWriter::file()
            })
        }
    }
}
