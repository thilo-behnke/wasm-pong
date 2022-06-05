use pong::game_object::components::{DefaultGeomComp, DefaultPhysicsComp};
use pong::game_object::game_object::{DefaultGameObject, GameObject};
use pong::geom::geom::Vector;
use pong::geom::shape::Shape;
use rstest::rstest;

#[rstest]
#[case(Vector::new(100., 100.), Vector::new(-1., 1.), Vector::new(99.9, 100.1), 0.1)]
#[case(Vector::new(300., 400.), Vector::new(-5., 0.), Vector::new(299.5, 400.), 0.1)]
#[case(Vector::new(300., 400.), Vector::new(-5., 0.), Vector::new(299.5, 400.), 0.013)]
pub fn should_update_pos(
    #[case] start_pos: Vector,
    #[case] vel: Vector,
    #[case] expected_pos: Vector,
    #[case] ms_diff: f64
) {
    let mut obj = DefaultGameObject::new(
        1,
        "obj".to_string(),
        Box::new(DefaultGeomComp::new(Shape::rect(
            Vector::new(start_pos.x as f64, start_pos.y as f64),
            Vector::new(1., 0.),
            0.,
            0.,
        ))),
        Box::new(DefaultPhysicsComp::new(vel, false)),
    );
    obj.update_pos(ms_diff);
    assert_eq!(*obj.pos(), expected_pos);
}
