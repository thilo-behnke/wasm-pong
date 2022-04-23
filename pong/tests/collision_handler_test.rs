use rstest::rstest;
use pong::collision::collision::CollisionHandler;
use pong::game_object::game_object::{GameObject, Shape};
use pong::geom::geom::Vector;

#[rstest]
// #[case(
//     // given
//     GameObject {id: 1, pos: Vector::zero(), vel: Vector::new(1., 0.), shape: Shape::Rect, shape_params: vec![], is_static: true, orientation: Vector::new(1., 0.)},
//     GameObject {id: 2, pos: Vector::zero(), vel: Vector::new(0., 1.), shape: Shape::Rect, shape_params: vec![], is_static: true, orientation: Vector::new(0., 1.)},
//     // expected
//     GameObject {id: 1, pos: Vector::zero(), vel: Vector::new(1., 0.), shape: Shape::Rect, shape_params: vec![], is_static: true, orientation: Vector::new(1., 0.)},
//     GameObject {id: 2, pos: Vector::zero(), vel: Vector::new(0., 1.), shape: Shape::Rect, shape_params: vec![], is_static: true, orientation: Vector::new(0., 1.)},
// )]
// #[case(
// // given
//     GameObject {id: 1, pos: Vector::zero(), vel: Vector::new(1., 0.), shape: Shape::Rect, shape_params: vec![], is_static: false, orientation: Vector::new(1., 0.)},
//     GameObject {id: 2, pos: Vector::zero(), vel: Vector::new(0., 1.), shape: Shape::Rect, shape_params: vec![], is_static: true, orientation: Vector::new(0., 1.)},
//     // expected
//     GameObject {id: 1, pos: Vector::zero(), vel: Vector::new(-1., 0.), shape: Shape::Rect, shape_params: vec![], is_static: false, orientation: Vector::new(1., 0.)},
//     GameObject {id: 2, pos: Vector::zero(), vel: Vector::new(0., 1.), shape: Shape::Rect, shape_params: vec![], is_static: true, orientation: Vector::new(0., 1.)},
// )]
// #[case(
//     // given
//     GameObject {id: 1, pos: Vector::zero(), vel: Vector::new(-1., 0.), shape: Shape::Rect, shape_params: vec![], is_static: false, orientation: Vector::new(-1., 0.)},
//     GameObject {id: 2, pos: Vector::zero(), vel: Vector::new(0., 1.), shape: Shape::Rect, shape_params: vec![], is_static: true, orientation: Vector::new(0., 1.)},
//     // expected
//     GameObject {id: 1, pos: Vector::zero(), vel: Vector::new(1., 0.), shape: Shape::Rect, shape_params: vec![], is_static: false, orientation: Vector::new(-1., 0.)},
//     GameObject {id: 2, pos: Vector::zero(), vel: Vector::new(0., 1.), shape: Shape::Rect, shape_params: vec![], is_static: true, orientation: Vector::new(0., 1.)},
// )]
// #[case(
// // given
//     GameObject {id: 1, pos: Vector::zero(), vel: Vector::new(1., 1.), shape: Shape::Rect, shape_params: vec![], is_static: false, orientation: Vector::new(1., 1.)},
//     GameObject {id: 2, pos: Vector::zero(), vel: Vector::new(0., 1.), shape: Shape::Rect, shape_params: vec![], is_static: true, orientation: Vector::new(0., 1.)},
//     // expected
//     GameObject {id: 1, pos: Vector::zero(), vel: Vector::new(-1., 1.), shape: Shape::Rect, shape_params: vec![], is_static: false, orientation: Vector::new(1., 1.)},
//     GameObject {id: 2, pos: Vector::zero(), vel: Vector::new(0., 1.), shape: Shape::Rect, shape_params: vec![], is_static: true, orientation: Vector::new(0., 1.)},
// )]
#[case(
    // given
    GameObject {id: 1, pos: Vector::zero(), vel: Vector::new(-2., 0.), shape: Shape::Rect, shape_params: vec![], is_static: false, orientation: Vector::new(-1., 0.)},
    GameObject {id: 2, pos: Vector::zero(), vel: Vector::new(0., 0.), shape: Shape::Rect, shape_params: vec![], is_static: true, orientation: Vector::new(0., 1.)},
    // expected
    GameObject {id: 1, pos: Vector::zero(), vel: Vector::new(2., 0.), shape: Shape::Rect, shape_params: vec![], is_static: false, orientation: Vector::new(-1., 0.)},
    GameObject {id: 2, pos: Vector::zero(), vel: Vector::new(0., 0.), shape: Shape::Rect, shape_params: vec![], is_static: true, orientation: Vector::new(0., 1.)},
)]
pub fn should_handle_collision(
    #[case] mut obj_a: GameObject,
    #[case] obj_b: GameObject,
    #[case] expected_a: GameObject,
    #[case] expected_b: GameObject,
) {
    let handler = CollisionHandler {};
    handler.handle(&mut obj_a, &obj_b);
    assert_eq!(obj_a, expected_a);
    assert_eq!(obj_b, expected_b);
}
