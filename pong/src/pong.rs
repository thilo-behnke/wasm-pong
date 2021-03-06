pub mod pong_collisions {
    use crate::game_object::game_object::GameObject;
    use crate::geom::shape::{get_bounding_box, ShapeType};
    use crate::geom::vector::Vector;
    use std::cell::{Ref, RefCell, RefMut};
    use std::f64::consts::{FRAC_PI_4};
    use std::rc::Rc;
    use crate::collision::handler::FieldStats;
    use crate::pong::pong_events::GameObjUpdate;
    use crate::utils::number_utils::is_in_range;

    pub fn handle_player_ball_collision(
        stats: &FieldStats,
        ball: &Rc<RefCell<Box<dyn GameObject>>>,
        player: &Rc<RefCell<Box<dyn GameObject>>>,
    ) {
        let mut ball = RefCell::borrow_mut(&ball);
        let ball_pos = ball.pos().clone();
        let mut ball_dimensions = ball.shape().dimensions();
        ball_dimensions.scalar_multiplication(0.5);
        let ball_height = ball_dimensions.y;

        // player is crushing the ball out of bounds
        if is_in_range(ball_pos.y, stats.dimensions.1 - ball_height, stats.dimensions.1 + ball_height) || is_in_range(ball_pos.y, 0. - ball_height, 0. + ball_height) {
            let mut player = player.borrow_mut();
            *player.vel_mut() = Vector::zero();
            return;
        }

        let player = player.borrow();
        // reflect
        let ball_vel = ball.vel_mut();
        let mut ball_vel_total = ball_vel.len();
        ball_vel.reflect(&player.orientation());
        ball_vel.normalize();

        // use vel of player obj
        if *player.vel() == Vector::zero() {
            ball_vel.y *= 0.50; // friction, if player does not move reduce vertical velocity.
            ball_vel.normalize();
        } else if player.vel().angle(&ball_vel) > FRAC_PI_4 {
            let mut collision_effect = player.orientation().clone();
            collision_effect.scalar_multiplication(0.5);

            ball_vel.add(&collision_effect);
            ball_vel.normalize();
        }

        ball_vel_total *= 1.02; // get faster every collision
        ball_vel_total = f64::min(ball_vel_total, 1000.); // max velocity.
        ball_vel.scalar_multiplication(ball_vel_total);
        *ball.vel_mut() = ball_vel.clone();

        // move out of collision
        let mut ball_vel = ball.vel().clone();
        ball_vel.normalize();
        ball_vel.scalar_multiplication(ball_dimensions.len());

        escape_collision(&mut ball, &player);
        ball.set_dirty(true);
    }

    pub fn handle_ball_bounds_collision(
        _stats: &FieldStats,
        ball: &Rc<RefCell<Box<dyn GameObject>>>,
        bound: &Rc<RefCell<Box<dyn GameObject>>>,
    ) {
        let mut ball = RefCell::borrow_mut(&ball);
        let mut ball_dimensions = ball.shape().dimensions();
        ball_dimensions.scalar_multiplication(0.5);
        let bound = RefCell::borrow(&bound);
        ball.vel_mut().reflect(&bound.orientation());

        escape_collision(&mut ball, &bound);
        ball.set_dirty(true);
    }

    pub fn handle_player_bound_collision(
        _stats: &FieldStats,
        player: &Rc<RefCell<Box<dyn GameObject>>>,
        bound: &Rc<RefCell<Box<dyn GameObject>>>,
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
        perpendicular.y *= height + 1.;
        let mut new_pos = bound.pos().clone();
        new_pos.add(&perpendicular);
        let player_pos = player.pos_mut();
        player_pos.y = new_pos.y;

        player.set_dirty(true);
    }

    fn escape_collision(game_obj_a: &mut RefMut<Box<dyn GameObject>>, game_obj_b: &Ref<Box<dyn GameObject>>) {
        let a_dims = game_obj_a.shape().dimensions();
        let mut a_vel = game_obj_a.vel().clone();
        a_vel.normalize();
        a_vel.multiply(&a_dims);
        let mut tries = 0;
        loop {
            if tries >= 5 {
                break;
            }
            let mut a_pos_updated = game_obj_a.pos().clone();
            a_pos_updated.add(&a_vel);
            *game_obj_a.pos_mut() = a_pos_updated;
            let ball_bounding_box = get_bounding_box(game_obj_a.shape());
            let player_bounding_box = get_bounding_box(game_obj_b.shape());
            if !ball_bounding_box.overlaps(&player_bounding_box) {
                break;
            }
            tries += 1;
        }
    }

    #[cfg(test)]
    mod tests {
        use rstest::rstest;
        use std::cell::RefCell;
        use std::rc::Rc;
        use crate::collision::handler::FieldStats;
        use crate::game_field::{Bound, Field};
        use crate::game_object::game_object::{DefaultGameObject, GameObject};
        use crate::geom::vector::Vector;
        use crate::pong::pong_collisions::handle_player_bound_collision;
        use crate::pong::pong_events::NoopPongEventWriter;
        use crate::utils::utils::DefaultLoggerFactory;

        #[rstest]
        #[case(
        // given
        create_player("1", 10, 0, Vector::new(0., -1.)),
        get_bound(Bound::BOTTOM),
        // expected
        create_player("2", 10, 31, Vector::new(0., -1.)),
        get_bound(Bound::BOTTOM)
        )]
        #[case(
        // given
        create_player("1", 10, 1, Vector::new(0., -1.)),
        get_bound(Bound::BOTTOM),
        // expected
        create_player("2", 10, 31, Vector::new(0., -1.)),
        get_bound(Bound::BOTTOM)
        )]
        #[case(
        // given
        create_player("1", 10, 601, Vector::new(0., 1.)),
        get_bound(Bound::TOP),
        // expected
        create_player("2", 10, 569, Vector::new(0., 1.)),
        get_bound(Bound::TOP)
        )]
        #[case(
        // given
        create_player("1", 10, 599, Vector::new(0., 1.)),
        get_bound(Bound::TOP),
        // expected
        create_player("2", 10, 569, Vector::new(0., 1.)),
        get_bound(Bound::TOP)
        )]
        pub fn should_correctly_handle_player_bounds_collision(
            #[case] player: Rc<RefCell<Box<dyn GameObject>>>,
            #[case] bounds: Rc<RefCell<Box<dyn GameObject>>>,
            #[case] player_expected: Rc<RefCell<Box<dyn GameObject>>>,
            #[case] bounds_expected: Rc<RefCell<Box<dyn GameObject>>>,
        ) {
            let stats = FieldStats {dimensions: (1000., 1000.)};
            handle_player_bound_collision(&stats, &player, &bounds);
            assert_eq!(player_expected.borrow().pos(), player.borrow().pos());
            assert_eq!(bounds_expected.borrow().pos(), bounds.borrow().pos());
        }

        fn create_player(id: &str, x: u16, y: u16, orientation: Vector) -> Rc<RefCell<Box<dyn GameObject>>> {
            let logger = DefaultLoggerFactory::noop();
            let event_writer = NoopPongEventWriter::new();
            let field = Field::new(logger, event_writer);
            let mut player = DefaultGameObject::player(id, x, y, &field);
            let player_orientation = player.orientation_mut();
            player_orientation.x = orientation.x;
            player_orientation.y = orientation.y;
            Rc::new(RefCell::new(player))
        }

        fn get_bound(bound: Bound) -> Rc<RefCell<Box<dyn GameObject>>> {
            let logger = DefaultLoggerFactory::noop();
            let event_writer = NoopPongEventWriter::new();
            let field = Field::new(logger, event_writer);
            let bounds = DefaultGameObject::bounds(field.width, field.height);
            return Rc::new(RefCell::new(
                bounds.into_iter().find(|b| b.0 == bound).unwrap().inner(),
            ));
        }
    }
}

pub mod pong_events {
    use crate::event::event::{EventWrapper, EventWriter};
    use crate::geom::vector::Vector;
    use serde::Serialize;

    #[derive(Serialize)]
    pub enum PongEventType<'a> {
        GameObjUpdate(GameObjUpdate<'a>),
    }

    #[derive(Serialize)]
    pub struct GameObjUpdate<'a> {
        pub obj_id: &'a str,
        pub pos: &'a Vector,
        pub vel: &'a Vector,
        pub orientation: &'a Vector,
    }

    pub trait PongEventWriter {
        fn write(&mut self, event: PongEventType) -> Result<(), String>;
    }

    pub struct DefaultPongEventWriter {
        writer: EventWriter,
    }

    impl PongEventWriter for DefaultPongEventWriter {
        fn write(&mut self, event: PongEventType) -> Result<(), String> {
            // TODO: Fix
            // let out_event = match event {
            //     PongEventType::GameObjUpdate(ref update) => Event {
            //         topic: String::from("obj_update"),
            //         key: Some(update.obj_id.clone().to_string()),
            //         msg: serde_json::to_string(&event).unwrap(),
            //     },
            // };
            // self.writer.write(out_event)
            Ok(())
        }
    }

    pub struct NoopPongEventWriter {}
    impl NoopPongEventWriter {
        pub fn new() -> Box<dyn PongEventWriter> {
            Box::new(DefaultPongEventWriter {
                writer: EventWriter::noop(),
            })
        }
    }

    impl DefaultPongEventWriter {
        pub fn new() -> Box<dyn PongEventWriter> {
            Box::new(DefaultPongEventWriter {
                writer: EventWriter::file(),
            })
        }
    }
}
