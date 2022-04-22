use rstest::rstest;
use pong::game_object::game_object::{GameObject, Shape};
use pong::geom::geom::{Vector};

#[rstest]
#[case(Vector::new(100., 100.), Vector::new(-1., 1.), Vector::new(99., 101.))]
pub fn should_update_pos(#[case] start_pos: Vector, #[case] vel: Vector, #[case] expected_pos: Vector) {
    let mut obj = GameObject {
        id: 1,
        pos: Vector::new(start_pos.x as f64, start_pos.y as f64),
        vel,
        shape: Shape::Rect,
        shape_params: vec![],
        is_static: false,
    };
    obj.update_pos();
    assert_eq!(obj.pos, expected_pos);
}
