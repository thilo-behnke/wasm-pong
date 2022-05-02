pub mod geom {
    use serde::{Serialize};

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
    }

    impl PartialEq for Vector {
        fn eq(&self, other: &Self) -> bool {
            (self.x * 1000.).round() == (other.x * 1000.).round()
                && (self.y * 1000.).round() == (other.y * 1000.).round()
        }
    }

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
}

pub mod shape {
    use crate::geom::geom::{BoundingBox, Vector};
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

        fn center(&self) -> &Vector {
            &self.center
        }

        fn center_mut(&mut self) -> &mut Vector {
            &mut self.center
        }

        fn orientation(&self) -> &Vector {
            &self.orientation
        }

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
