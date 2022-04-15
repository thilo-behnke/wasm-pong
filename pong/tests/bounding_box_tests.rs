use rstest::rstest;
use pong::geom::geom::{BoundingBox, Vector};

#[rstest]
#[case(BoundingBox::create_from_coords(10., 10., 5, 5), Vector::new(10., 10.), true)]
#[case(BoundingBox::create_from_coords(10., 10., 5, 5), Vector::new(8., 8.), true)]
#[case(BoundingBox::create_from_coords(10., 10., 5, 5), Vector::new(20., 20.), false)]
pub fn should_correctly_determine_if_point_is_within_box(
    #[case] bounding_box: BoundingBox,
    #[case] point: Vector,
    #[case] expected: bool,
) {
    let res = bounding_box.is_point_within(&point);
    assert_eq!(res, expected);
}

#[rstest]
#[case(
    BoundingBox::create_from_coords(10., 10., 5, 5),
    BoundingBox::create_from_coords(10., 10., 5, 5),
    true
)]
#[case(
    BoundingBox::create_from_coords(10., 10., 5, 5),
    BoundingBox::create_from_coords(8., 8., 5, 5),
    true
)]
#[case(
    BoundingBox::create_from_coords(10., 10., 5, 5),
    BoundingBox::create_from_coords(4.9, 4.9, 5, 5),
    false
)]
#[case(
    BoundingBox::create_from_coords(10., 10., 5, 5),
    BoundingBox::create_from_coords(5., 5., 5, 5),
    true
)]
pub fn should_correctly_determine_if_overlap(
    #[case] bounding_box_a: BoundingBox,
    #[case] bounding_box_b: BoundingBox,
    #[case] expected: bool,
) {
    let res = bounding_box_a.overlaps(&bounding_box_b);
    assert_eq!(res, expected);
}
