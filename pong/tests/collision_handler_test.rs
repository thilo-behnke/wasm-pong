use rstest::rstest;
use pong::collision::collision::CollisionHandler;
use pong::game_object::components::{DefaultGeomComp, DefaultPhysicsComp};
use pong::game_object::game_object::{DefaultGameObject, GameObject};
use pong::geom::geom::Vector;
use pong::geom::shape::Rect;

#[rstest]
#[case(
    // given
    create_game_obj(1, Vector::new(1., 0.), Vector::new(1., 0.), true),
    create_game_obj(2, Vector::new(0., 0.), Vector::new(0., 1.), true),
    // expected
    create_game_obj(1, Vector::new(1., 0.), Vector::new(1., 0.), true),
    create_game_obj(2, Vector::new(0., 0.), Vector::new(0., 1.), true),
)]
#[case(
    // given
    create_game_obj(1, Vector::new(1., 0.), Vector::new(1., 0.), false),
    create_game_obj(2, Vector::new(0., 0.), Vector::new(0., 1.), true),
    // expected
    create_game_obj(1, Vector::new(-1., 0.), Vector::new(1., 0.), false),
    create_game_obj(2, Vector::new(0., 0.), Vector::new(0., 1.), true),
)]
#[case(
    // given
    create_game_obj(1, Vector::new(-1., 0.), Vector::new(-1., 0.), false),
    create_game_obj(2, Vector::new(0., 0.), Vector::new(0., 1.), true),
    // expected
    create_game_obj(1, Vector::new(1., 0.), Vector::new(-1., 0.), false),
    create_game_obj(2, Vector::new(0., 0.), Vector::new(0., 1.), true),
)]
#[case(
    // given
    create_game_obj(1, Vector::new(1., 1.), Vector::new(1., 1.), false),
    create_game_obj(2, Vector::new(0., 0.), Vector::new(0., 1.), true),
    // expected
    create_game_obj(1, Vector::new(-1., 1.), Vector::new(1., 1.), false),
    create_game_obj(2, Vector::new(0., 0.), Vector::new(0., 1.), true),
)]
#[case(
    // given
    create_game_obj(1, Vector::new(-2., 1.), Vector::new(-1., 0.), false),
    create_game_obj(2, Vector::new(0., 0.), Vector::new(0., 1.), true),
    // expected
    create_game_obj(1, Vector::new(2., 1.), Vector::new(-1., 0.), false),
    create_game_obj(2, Vector::new(0., 0.), Vector::new(0., 1.), true),
)]
#[case(
    // given
    create_game_obj(1, Vector::new(1., 0.), Vector::new(1., 0.), false),
    create_game_obj(2, Vector::new(0., 1.), Vector::new(0., 1.), true),
    // expected
    create_game_obj(1, Vector::new(-1., 1.), Vector::new(1., 0.), false),
    create_game_obj(2, Vector::new(0., 1.), Vector::new(0., 1.), true),
)]
#[case(
    // given
    create_game_obj(1, Vector::new(-2., 1.), Vector::new(-1., 0.), false),
    create_game_obj(2, Vector::new(0., 0.), Vector::new(0., 1.), true),
    // expected
    create_game_obj(1, Vector::new(2., 1.), Vector::new(-1., 0.), false),
    create_game_obj(2, Vector::new(0., 0.), Vector::new(0., 1.), true),
)]
pub fn should_handle_collision(
    #[case] mut obj_a: Box<dyn GameObject>,
    #[case] obj_b: Box<dyn GameObject>,
    #[case] expected_a: Box<dyn GameObject>,
    #[case] expected_b: Box<dyn GameObject>,
) {
    let handler = CollisionHandler {};
    handler.handle(&mut obj_a, &obj_b);
    assert_eq!(obj_a.pos(), expected_a.pos());
    assert_eq!(obj_a.vel(), expected_a.vel());
    assert_eq!(obj_b.pos(), expected_b.pos());
    assert_eq!(obj_b.vel(), expected_b.vel());
}

fn create_game_obj(id: u16, vel: Vector, orientation: Vector, is_static: bool) -> Box<dyn GameObject> {
    Box::new(DefaultGameObject::new(
        id,
    Box::new(DefaultGeomComp::new(
        Box::new(
            Rect::new(Vector::zero(), orientation, 20., 20.)
        ))),
        Box::new(DefaultPhysicsComp::new(
            vel, is_static
        ))
    ))
}
