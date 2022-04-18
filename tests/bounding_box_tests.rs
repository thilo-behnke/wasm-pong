use rstest::rstest;
use rust_wasm::{BoundingBox, Point};

#[rstest]
#[case(BoundingBox::create(10, 10, 5, 5), Point::create(10, 10), true)]
#[case(BoundingBox::create(10, 10, 5, 5), Point::create(8, 8), true)]
#[case(BoundingBox::create(10, 10, 5, 5), Point::create(20, 20), false)]
pub fn should_correctly_determine_if_point_is_within_box(#[case] bounding_box: BoundingBox, #[case] point: Point, #[case] expected: bool) {
    let res = bounding_box.is_point_within(&point);
    assert_eq!(res, expected);
}

#[rstest]
#[case(BoundingBox::create(10, 10, 5, 5), BoundingBox::create(10, 10, 5, 5), true)]
#[case(BoundingBox::create(10, 10, 5, 5), BoundingBox::create(8, 8, 5, 5), true)]
#[case(BoundingBox::create(10, 10, 5, 5), BoundingBox::create(5, 5, 5, 5), false)]
pub fn should_correctly_determine_if_overlap(#[case] bounding_box_a: BoundingBox, #[case] bounding_box_b: BoundingBox, #[case] expected: bool) {
    let res = bounding_box_a.overlaps(&bounding_box_b);
    assert_eq!(res, expected);
}
