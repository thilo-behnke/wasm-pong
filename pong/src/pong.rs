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
    use crate::event::event::EventWriter;
    use crate::geom::geom::Vector;

    pub enum PongEventType {
        GameObjUpdate(GameObjUpdate)
    }

    pub struct GameObjUpdate {
        pub pos: Vector,
        pub vel: Vector,
        pub orientation: Vector
    }

    pub struct PongEventWriter {
        writer: EventWriter
    }

    impl PongEventWriter {
        pub fn new() -> PongEventWriter {
            PongEventWriter {
                writer: EventWriter::file()
            }
        }

        pub fn write(&self, event: PongEventType) -> std::io::Result<()> {
            // TODO: Event to string
            self.writer.write()
        }
    }


}
