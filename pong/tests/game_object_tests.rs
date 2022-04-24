use rstest::rstest;
use pong::game_object::components::{DefaultGeomComp, DefaultPhysicsComp};
use pong::game_object::game_object::{DefaultGameObject, GameObject};
use pong::geom::geom::{Vector};
use pong::geom::shape::Shape;

#[rstest]
#[case(Vector::new(100., 100.), Vector::new(-1., 1.), Vector::new(99., 101.))]
pub fn should_update_pos(#[case] start_pos: Vector, #[case] vel: Vector, #[case] expected_pos: Vector) {
    let mut obj = DefaultGameObject::new(
        1,
        Box::new(DefaultGeomComp::new(
            Shape::rect(Vector::new(start_pos.x as f64, start_pos.y as f64), Vector::new(1., 0.), 0., 0.)
        )),
        Box::new(DefaultPhysicsComp::new(
            vel, false
        ))
    );
    obj.update_pos();
    assert_eq!(*obj.pos(), expected_pos);
}
