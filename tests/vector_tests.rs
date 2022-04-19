use rstest::rstest;
use rust_wasm::geom::geom::Vector;

#[rstest]
#[case(1, 0, 1)]
#[case(0, 1, 1)]
#[case(3, 4, 5)]
pub fn should_get_correct_length(#[case] x: i32, #[case] y: i32, #[case] expected: i32) {
    let vector = Vector { x, y };
    let len = vector.len();
    assert_eq!(len, expected);
}

#[rstest]
#[case(1, 0, 1, 0)]
#[case(3, 0, 1, 0)]
#[case(3, 4, 3 / 5, 4 / 5)]
pub fn should_normalize_correctly(
    #[case] x: i32,
    #[case] y: i32,
    #[case] expected_x: i32,
    #[case] expected_y: i32,
) {
    let mut vector = Vector { x, y };
    let expected = Vector {
        x: expected_x,
        y: expected_y,
    };
    vector.normalize();
    assert_eq!(vector, expected);
}
