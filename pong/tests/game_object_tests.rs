use rstest::rstest;
use pong::game_object::game_object::{GameObject, Shape};
use pong::geom::geom::{Point, Vector};

#[rstest]
#[case(Point::create(100, 100), Vector::new(-1., 1.), Point::create(99, 101))]
pub fn should_update_pos(#[case] start_pos: Point, #[case] vel: Vector, #[case] expected_pos: Point) {
    let mut obj = GameObject {
        id: 1,
        x: start_pos.x as u16,
        y: start_pos.y as u16,
        vel,
        shape: Shape::Rect,
        shape_params: vec![],
        is_static: false,
    };
    obj.update_pos(1000, 1000);
    assert_eq!(obj.x, expected_pos.x as u16);
    assert_eq!(obj.y, expected_pos.y as u16);
}
