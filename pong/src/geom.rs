pub mod vector {
    use serde::Serialize;

    #[derive(Debug, Clone, Serialize)]
    pub struct Vector {
        pub x: f64,
        pub y: f64,
    }

    impl Vector {
        pub fn zero() -> Vector {
            Vector { x: 0., y: 0. }
        }

        pub fn unit() -> Vector {
            let mut vector = Vector { x: 1., y: 1. };
            vector.normalize();
            vector
        }

        pub fn new(x: f64, y: f64) -> Vector {
            Vector { x, y }
        }

        pub fn inverted(vec: &Vector) -> Vector {
            let mut inverted = vec.clone();
            inverted.invert();
            return inverted;
        }

        pub fn normalize(&mut self) {
            if self == &Vector::zero() {
                return;
            }
            let length = self.len();
            self.x /= length;
            self.y /= length;
        }

        pub fn orthogonal_clockwise(&mut self) {
            let updated_x = self.y;
            let updated_y = -self.x;
            self.x = updated_x;
            self.y = updated_y;
        }

        pub fn orthogonal_counter_clockwise(&mut self) {
            let updated_x = -self.y;
            let updated_y = self.x;
            self.x = updated_x;
            self.y = updated_y;
        }

        pub fn rotate(&mut self, radians: f64) {
            let updated_x = self.x * radians.cos() - self.y * radians.sin();
            let updated_y = self.x * radians.sin() + self.y * radians.cos();
            self.x = updated_x;
            self.y = updated_y;
        }

        pub fn add(&mut self, other: &Vector) {
            self.x += other.x;
            self.y += other.y;
        }

        pub fn sub(&mut self, other: &Vector) {
            self.x -= other.x;
            self.y -= other.y;
        }

        pub fn invert(&mut self) {
            self.x = self.x * -1.;
            self.y = self.y * -1.;
        }

        pub fn dot(&self, other: &Vector) -> f64 {
            return self.x * other.x + self.y * other.y;
        }

        pub fn angle(&self, other: &Vector) -> f64 {
            let mut self_clone = self.clone();
            self_clone.normalize();
            let mut other_clone = other.clone();
            other_clone.normalize();

            let dot = self_clone.dot(&other_clone);
            let dot_float = dot as f64;
            let acos_res = dot_float.acos();
            (acos_res * 100.0).round() / 100.0
        }

        // r = d - 2 * (d * n) * n
        pub fn reflect(&mut self, onto: &Vector) {
            let dot = self.dot(onto);
            if dot == 0. {
                self.invert();
                return;
            }
            let mut orthogonal = self.get_opposing_orthogonal(onto);
            let d_dot_n = orthogonal.dot(self);
            orthogonal.scalar_multiplication(d_dot_n);
            orthogonal.scalar_multiplication(2.);
            self.sub(&orthogonal);
        }

        pub fn get_projection(&self, onto: &Vector) -> Vector {
            let mut onto_normalized = onto.clone();
            onto_normalized.normalize();
            let dot = self.dot(&onto_normalized);
            let mut projected = onto_normalized.clone();
            projected.scalar_multiplication(dot);
            projected
        }

        pub fn get_opposing_orthogonal(&self, onto: &Vector) -> Vector {
            let mut orthogonal1 = onto.clone();
            orthogonal1.orthogonal_clockwise();
            if self.dot(&orthogonal1) < 0. {
                // orthogonal1.normalize();
                return orthogonal1;
            }
            let mut orthogonal2 = onto.clone();
            orthogonal2.orthogonal_counter_clockwise();
            // orthogonal2.normalize();
            return orthogonal2;
        }

        pub fn scalar_multiplication(&mut self, n: f64) {
            self.x *= n;
            self.y *= n;
        }

        pub fn len(&self) -> f64 {
            let distance = self.x.powi(2) + self.y.powi(2);
            return (distance as f64).sqrt();
        }

        pub fn multiply(&mut self, other: &Vector) {
            self.x = self.x * other.x;
            self.y = self.y * other.y;
        }

        pub fn max(&mut self, other: &Vector) {
            self.x = self.x.max(other.x);
            self.y = self.y.max(other.y);
        }

        pub fn min(&mut self, other: &Vector) {
            self.x = self.x.min(other.x);
            self.y = self.y.min(other.y);
        }

        pub fn abs(&mut self) {
            self.x = self.x.abs();
            self.y = self.y.abs();
        }
    }

    impl PartialEq for Vector {
        fn eq(&self, other: &Self) -> bool {
            (self.x * 1000.).round() == (other.x * 1000.).round()
                && (self.y * 1000.).round() == (other.y * 1000.).round()
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::geom::vector::Vector;
        use rstest::rstest;
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
            #[case] expected_angle: f64,
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
            #[case] expected: Vector,
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
            #[case] expected: Vector,
        ) {
            vector.orthogonal_counter_clockwise();
            assert_eq!(vector, expected);
        }

        #[rstest]
        #[case(Vector::new(1., 0.), FRAC_PI_4, Vector::unit())]
        pub fn should_correctly_rotate(
            #[case] mut vector: Vector,
            #[case] radians: f64,
            #[case] expected: Vector,
        ) {
            vector.rotate(radians);
            assert_eq!(vector, expected);
        }

        #[rstest]
        #[case(Vector::new(1., 0.), Vector::new(1., 0.), 1.)]
        #[case(Vector::new(1., 0.), Vector::new(0., 1.), 0.)]
        #[case(Vector::new(1., 0.), Vector::new(-1., 0.), -1.)]
        pub fn should_calculate_dot_product(
            #[case] vector: Vector,
            #[case] other: Vector,
            #[case] expected: f64,
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

        #[rstest]
        #[case(Vector::new(1., 1.), Vector::new(1., 0.), Vector::new(1., -1.))]
        #[case(Vector::new(-1., 1.), Vector::new(1., 0.), Vector::new(-1., -1.))]
        #[case(Vector::new(1., -1.), Vector::new(0., 1.), Vector::new(-1., -1.))]
        #[case(Vector::new(-1., -1.), Vector::new(0., 1.), Vector::new(1., -1.))]
        pub fn should_reflect_vector(
            #[case] mut vector: Vector,
            #[case] onto: Vector,
            #[case] expected: Vector,
        ) {
            vector.reflect(&onto);
            assert_eq!(vector, expected);
        }
    }
}

pub mod utils {
    use crate::geom::vector::Vector;

    #[derive(Clone, Debug)]
    pub struct BoundingBox {
        top_left: Vector,
        top_right: Vector,
        bottom_left: Vector,
        bottom_right: Vector,
    }

    impl BoundingBox {
        pub fn create(center: &Vector, width: f64, height: f64) -> BoundingBox {
            let center_x = center.x;
            let center_y = center.y;
            let top_left = Vector {
                x: center_x - width / 2.,
                y: center_y + height / 2.,
            };
            let top_right = Vector {
                x: center_x + width / 2.,
                y: center_y + height / 2.,
            };
            let bottom_left = Vector {
                x: center_x - width / 2.,
                y: center_y - height / 2.,
            };
            let bottom_right = Vector {
                x: center_x + width / 2.,
                y: center_y - height / 2.,
            };
            BoundingBox {
                top_left,
                top_right,
                bottom_left,
                bottom_right,
            }
        }

        pub fn points(&self) -> Vec<&Vector> {
            return vec![
                &self.top_left,
                &self.top_right,
                &self.bottom_left,
                &self.bottom_right,
            ];
        }

        pub fn vert(&self) -> Range {
            Range::new(self.bottom_left.y, self.top_left.y)
        }

        pub fn hor(&self) -> Range {
            Range::new(self.top_left.x, self.top_right.x)
        }

        pub fn overlaps(&self, other: &BoundingBox) -> bool {
            self.vert().overlaps(&other.vert()) && self.hor().overlaps(&other.hor())
        }

        pub fn is_point_within(&self, point: &Vector) -> bool {
            return point.x >= self.top_left.x
                && point.x <= self.top_right.x
                && point.y <= self.top_left.y
                && point.y >= self.bottom_left.y;
        }
    }

    pub struct Range {
        min: f64,
        max: f64,
    }

    impl Range {
        pub fn new(a: f64, b: f64) -> Range {
            if a <= b {
                return Range { min: a, max: b };
            }
            return Range { min: b, max: a };
        }

        pub fn overlaps(&self, other: &Range) -> bool {
            if self.min >= other.min && self.max <= other.max {
                return true;
            }

            if self.max >= other.min && self.max <= other.max {
                return true;
            }

            if other.min >= self.min && other.max <= self.max {
                return true;
            }

            if other.max >= self.min && other.max <= self.max {
                return true;
            }

            return false;
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::geom::utils::BoundingBox;
        use crate::geom::vector::Vector;
        use rstest::rstest;

        #[rstest]
        #[case(BoundingBox::create(&Vector::new(10., 10.), 5., 5.), Vector::new(10., 10.), true)]
        #[case(BoundingBox::create(&Vector::new(10., 10.), 5., 5.), Vector::new(8., 8.), true)]
        #[case(BoundingBox::create(&Vector::new(10., 10.), 5., 5.), Vector::new(20., 20.), false)]
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
        BoundingBox::create(&Vector::new(10., 10.), 5., 5.),
        BoundingBox::create(&Vector::new(10., 10.), 5., 5.),
        true
        )]
        #[case(
        BoundingBox::create(&Vector::new(10., 10.), 5., 5.),
        BoundingBox::create(&Vector::new(8., 8.), 5., 5.),
        true
        )]
        #[case(
        BoundingBox::create(&Vector::new(10., 10.), 5., 5.),
        BoundingBox::create(&Vector::new(4.9, 4.9), 5., 5.),
        false
        )]
        #[case(
        BoundingBox::create(&Vector::new(10., 10.), 5., 5.),
        BoundingBox::create(&Vector::new(5., 5.), 5., 5.),
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
    }
}

pub mod shape {
    use crate::geom::utils::BoundingBox;
    use crate::geom::vector::Vector;
    use std::fmt::Debug;

    #[derive(Clone, Debug, PartialEq)]
    pub enum ShapeType {
        Rect(Shape, f64, f64),
        Circle(Shape, f64),
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct Shape {
        center: Vector,
        orientation: Vector,
    }

    impl Shape {
        pub fn rect(center: Vector, orientation: Vector, width: f64, height: f64) -> ShapeType {
            ShapeType::Rect(
                Shape {
                    center,
                    orientation,
                },
                width,
                height,
            )
        }

        pub fn circle(center: Vector, orientation: Vector, radius: f64) -> ShapeType {
            ShapeType::Circle(
                Shape {
                    center,
                    orientation,
                },
                radius,
            )
        }

        #[allow(dead_code)]
        fn center(&self) -> &Vector {
            &self.center
        }

        #[allow(dead_code)]
        fn center_mut(&mut self) -> &mut Vector {
            &mut self.center
        }

        #[allow(dead_code)]
        fn orientation(&self) -> &Vector {
            &self.orientation
        }

        #[allow(dead_code)]
        fn orientation_mut(&mut self) -> &mut Vector {
            &mut self.orientation
        }
    }

    pub fn get_center(shape: &ShapeType) -> &Vector {
        match shape {
            ShapeType::Rect(ref s, _, _) => &s.center,
            ShapeType::Circle(ref s, _) => &s.center,
        }
    }

    pub fn get_center_mut(shape: &mut ShapeType) -> &mut Vector {
        match shape {
            ShapeType::Rect(ref mut s, _, _) => &mut s.center,
            ShapeType::Circle(ref mut s, _) => &mut s.center,
        }
    }

    pub fn get_orientation(shape: &ShapeType) -> &Vector {
        match shape {
            ShapeType::Rect(s, _, _) => &s.orientation,
            ShapeType::Circle(s, _) => &s.orientation,
        }
    }

    pub fn get_orientation_mut(shape: &mut ShapeType) -> &mut Vector {
        match shape {
            ShapeType::Rect(ref mut s, _, _) => &mut s.orientation,
            ShapeType::Circle(ref mut s, _) => &mut s.orientation,
        }
    }

    pub fn get_bounding_box(shape: &ShapeType) -> BoundingBox {
        match shape {
            ShapeType::Rect(s, width, height) => BoundingBox::create(&s.center, *width, *height),
            ShapeType::Circle(s, radius) => {
                BoundingBox::create(&s.center, *radius * 2., *radius * 2.)
            }
        }
    }
}
