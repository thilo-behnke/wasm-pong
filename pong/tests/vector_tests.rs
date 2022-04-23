use rstest::rstest;
use pong::geom::geom::Vector;
use std::f64::consts::PI;
use std::f64::consts::FRAC_PI_2;
use std::f64::consts::FRAC_PI_4;

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

#[rstest]
#[case(Vector::new(1., 0.), Vector::new(0., -1.))]
#[case(Vector::new(0., 1.), Vector::new(1., 0.))]
#[case(Vector::new(7., 7.), Vector::new(7., -7.))]
pub fn should_get_orthogonal_clockwise(
    #[case] mut vector: Vector,
    #[case] expected: Vector
) {
    vector.orthogonal_clockwise();
    assert_eq!(vector, expected);
}

#[rstest]
#[case(Vector::new(0., -1.), Vector::new(1., 0.))]
#[case(Vector::new(1., 0.), Vector::new(0., 1.))]
#[case(Vector::new(7., 7.), Vector::new(-7., 7.))]
pub fn should_get_orthogonal_counter_clockwise(
    #[case] mut vector: Vector,
    #[case] expected: Vector
) {
    vector.orthogonal_counter_clockwise();
    assert_eq!(vector, expected);
}

#[rstest]
#[case(Vector::new(1., 0.), FRAC_PI_4, Vector::unit())]
pub fn should_correctly_rotate(
    #[case] mut vector: Vector,
    #[case] radians: f64,
    #[case] expected: Vector
) {
    vector.rotate(radians);
    assert_eq!(vector, expected);
}

#[rstest]
#[case(Vector::new(1., 0.), Vector::new(1., 0.), 1.)]
#[case(Vector::new(1., 0.), Vector::new(0., 1.), 0.)]
#[case(Vector::new(1., 0.), Vector::new(-1., 0.), -1.)]
pub fn should_calculate_dot_product(
    #[case] mut vector: Vector,
    #[case] mut other: Vector,
    #[case] expected: f64

) {
    let dot = vector.dot(&other);
    assert_eq!(dot, expected);
}

#[rstest]
#[case(Vector::new(0., 1.), Vector::new(1., 0.), Vector::new(0., 0.))]
#[case(Vector::new(1., 0.), Vector::new(1., 0.), Vector::new(1., 0.))]
#[case(Vector::new(-1., 0.), Vector::new(1., 0.), Vector::new(-1., 0.))]
#[case(Vector::new(1., 1.), Vector::new(1., 0.), Vector::new(1., 0.))]
#[case(Vector::new(2., 1.), Vector::new(1., 0.), Vector::new(2., 0.))]
pub fn should_get_projection(
    #[case] vector: Vector,
    #[case] other: Vector,
    #[case] expected: Vector,
) {
    let projected = vector.get_projection(&other);
    assert_eq!(projected, expected);
}

#[rstest]
#[case(Vector::new(1., 1.), Vector::new(1., 0.), Vector::new(0., -1.))]
#[case(Vector::new(-1., -1.), Vector::new(1., 0.), Vector::new(0., 1.))]
pub fn should_get_opposing_orthogonal(
    #[case] vector: Vector,
    #[case] onto: Vector,
    #[case] expected: Vector,
) {
    let orthogonal = vector.get_opposing_orthogonal(&onto);
    assert_eq!(orthogonal, expected);
}


