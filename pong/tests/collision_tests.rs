use rstest::rstest;
use pong::collision::collision::{Collision, CollisionDetector};
use pong::game_object::game_object::{GameObject, Shape};
use pong::geom::geom::{Vector};

#[rstest]
#[case(vec![], vec![])]
#[case(
    vec![
        GameObject{id: 1, pos: Vector{x: 50., y: 50.}, shape: Shape::Rect, shape_params: vec![20, 20], vel: Vector::zero(), is_static: false},
        GameObject{id: 2, pos: Vector{x: 50., y: 50.}, shape: Shape::Rect, shape_params: vec![20, 20], vel: Vector::zero(), is_static: false}
    ],
    vec![Collision(1, 2)]
)]
#[case(
    vec![
        GameObject{id: 1, pos: Vector{x: 60., y: 65.}, shape: Shape::Rect, shape_params: vec![20, 20], vel: Vector::zero(), is_static: false},
        GameObject{id: 2, pos: Vector{x: 50., y: 50.}, shape: Shape::Rect, shape_params: vec![20, 20], vel: Vector::zero(), is_static: false}
    ],
    vec![Collision(1, 2)]
)]
#[case(
    vec![
        GameObject{id: 1, pos: Vector{x: 50., y: 50.}, shape: Shape::Rect, shape_params: vec![20, 20], vel: Vector::zero(), is_static: false},
        GameObject{id: 2, pos: Vector{x: 80., y: 80.}, shape: Shape::Rect, shape_params: vec![20, 20], vel: Vector::zero(), is_static: false}
    ],
    vec![]
)]
#[case(
    vec![
        GameObject{id: 1, pos: Vector{x: 50., y: 50.}, shape: Shape::Rect, shape_params: vec![50, 50], vel: Vector::zero(), is_static: false},
        GameObject{id: 2, pos: Vector{x: 500., y: 50.}, shape: Shape::Rect, shape_params: vec![50, 50], vel: Vector::zero(), is_static: false}
    ],
    vec![]
)]
pub fn should_detect_collisions(
    #[case] objs: Vec<GameObject>,
    #[case] expected_collisions: Vec<Collision>,
) {
    let detector = CollisionDetector::new();
    let res = detector.detect_collisions(objs.iter().collect());
    assert_eq!(
        res.get_collisions(),
        expected_collisions.iter().collect::<Vec<&Collision>>()
    );
}
