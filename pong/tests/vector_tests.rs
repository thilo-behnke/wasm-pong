use rstest::rstest;
use pong::geom::geom::Vector;

#[rstest]
#[case(1., 0., 1.)]
#[case(0., 1., 1.)]
#[case(3., 4., 5.)]
pub fn should_get_correct_length(#[case] x: f64, #[case] y: f64, #[case] expected: f64) {
    let vector = Vector { x, y };
    let len = vector.len();
    assert_eq!(len, expected);
}

#[rstest]
#[case(1., 0., 1., 0.)]
#[case(3., 0., 1., 0.)]
#[case(3., 4., 3. / 5., 4. / 5.)]
pub fn should_normalize_correctly(
    #[case] x: f64,
    #[case] y: f64,
    #[case] expected_x: f64,
    #[case] expected_y: f64,
) {
    let mut vector = Vector { x, y };
    let expected = Vector {
        x: expected_x,
        y: expected_y,
    };
    vector.normalize();
    assert_eq!(vector, expected);
}

#[rstest]
#[case(Vector::new(1., 1.), Vector::new(2., 2.), 0.)]
#[case(Vector::new(1., 0.), Vector::new(1., 1.), 0.79)]
pub fn should_calculate_angle_correctly(
    #[case] vector_a: Vector,
    #[case] vector_b: Vector,
    #[case] expected_angle: f64
) {
    let res = vector_a.angle(&vector_b);
    assert_eq!(res, expected_angle);
}
