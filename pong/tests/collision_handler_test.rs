use pong::collision::collision::CollisionHandler;
use pong::game_object::components::{DefaultGeomComp, DefaultPhysicsComp};
use pong::game_object::game_object::{DefaultGameObject, GameObject};
use pong::geom::geom::Vector;
use pong::geom::shape::Shape;
use pong::utils::utils::DefaultLoggerFactory;
use rstest::rstest;
use std::cell::RefCell;
use std::rc::Rc;

#[rstest]
#[case(
    create_game_obj(1, Vector::new(1., 0.), Vector::new(1., 0.), true),
    create_game_obj(2, Vector::new(0., 0.), Vector::new(0., 1.), true),
)]
#[case(
    create_game_obj(1, Vector::new(1., 0.), Vector::new(1., 0.), false),
    create_game_obj(2, Vector::new(0., 0.), Vector::new(0., 1.), true),
)]
#[case(
    create_game_obj(1, Vector::new(-1., 0.), Vector::new(-1., 0.), false),
    create_game_obj(2, Vector::new(0., 0.), Vector::new(0., 1.), true),
)]
#[case(
    create_game_obj(1, Vector::new(1., 1.), Vector::new(1., 1.), false),
    create_game_obj(2, Vector::new(0., 0.), Vector::new(0., 1.), true),
)]
#[case(
    create_game_obj(1, Vector::new(-2., 1.), Vector::new(-1., 0.), false),
    create_game_obj(2, Vector::new(0., 0.), Vector::new(0., 1.), true),
)]
#[case(
    create_game_obj(1, Vector::new(1., 0.), Vector::new(1., 0.), false),
    create_game_obj(2, Vector::new(0., 1.), Vector::new(0., 1.), true),
)]
#[case(
    create_game_obj(1, Vector::new(-2., 1.), Vector::new(-1., 0.), false),
    create_game_obj(2, Vector::new(0., 0.), Vector::new(0., 1.), true),
)]
pub fn should_handle_collision(
    #[case] obj_a: Rc<RefCell<Box<dyn GameObject>>>,
    #[case] obj_b: Rc<RefCell<Box<dyn GameObject>>>
) {
    let logger = DefaultLoggerFactory::noop();
    let mut handler = CollisionHandler::new(&logger);
    handler.register((String::from("obj"), String::from("obj")), |_a, _b| {
        let mut a_mut = RefCell::borrow_mut(_a);
        let mut vel_inverted = a_mut.vel().clone();
        vel_inverted.invert();
        *a_mut.vel_mut() = vel_inverted;
    });
    let expected_vel_a = Vector::inverted(RefCell::borrow(&obj_a).vel());
    let res = handler.handle(&obj_a, &obj_b);
    assert_eq!(true, res);
    assert_eq!(RefCell::borrow(&obj_a).pos(), RefCell::borrow(&obj_a).pos());
    assert_eq!(RefCell::borrow(&obj_a).vel(), &expected_vel_a);
    assert_eq!(RefCell::borrow(&obj_b).pos(), RefCell::borrow(&obj_b).pos());
    assert_eq!(RefCell::borrow(&obj_a).vel(), RefCell::borrow(&obj_a).vel());
}

fn create_game_obj(
    id: u16,
    vel: Vector,
    orientation: Vector,
    is_static: bool,
) -> Rc<RefCell<Box<dyn GameObject>>> {
    Rc::new(RefCell::new(Box::new(DefaultGameObject::new(
        id,
        "obj".to_string(),
        Box::new(DefaultGeomComp::new(Shape::rect(
            Vector::zero(),
            orientation,
            20.,
            20.,
        ))),
        Box::new(DefaultPhysicsComp::new(vel, is_static)),
    ))))
}
