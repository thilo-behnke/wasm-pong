pub mod geom {
    #[derive(Debug, Clone)]
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
            Vector {
                x, y
            }
        }

        pub fn normalize(&mut self) {
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
            return self.x * other.x + self.y * other.y
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
                return orthogonal1;
            }
            let mut orthogonal2 = onto.clone();
            orthogonal2.orthogonal_counter_clockwise();
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
            (self.x * 1000.).round() == (other.x * 1000.).round() &&
                (self.y * 1000.).round() == (other.y * 1000.).round()
        }
    }

    pub struct BoundingBox {
        top_left: Vector,
        top_right: Vector,
        bottom_left: Vector,
        bottom_right: Vector,
    }

    impl BoundingBox {
        pub fn create_from_coords(x: f64, y: f64, width: u16, height: u16) -> BoundingBox {
            let center = Vector::new(x, y);
            return BoundingBox::create(&center, width, height)
        }

        pub fn create(center: &Vector, width: u16, height: u16) -> BoundingBox {
            let center_x = center.x;
            let center_y = center.y;
            let top_left = Vector {
                x: center_x - (width as f64 / 2.),
                y: center_y + (height as f64 / 2.),
            };
            let top_right = Vector {
                x: center_x + (width as f64 / 2.),
                y: center_y + (height as f64 / 2.),
            };
            let bottom_left = Vector {
                x: center_x - (width as f64 / 2.),
                y: center_y - (height as f64 / 2.),
            };
            let bottom_right = Vector {
                x: center_x + (width as f64 / 2.),
                y: center_y - (height as f64 / 2.),
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

        pub fn overlaps(&self, other: &BoundingBox) -> bool {
            return self.points().iter().any(|p| other.is_point_within(p)) || other.points().iter().any(|p| self.is_point_within(p));
        }

        pub fn is_point_within(&self, point: &Vector) -> bool {
            return point.x >= self.top_left.x
                && point.x <= self.top_right.x
                && point.y <= self.top_left.y
                && point.y >= self.bottom_left.y;
        }
    }
}
