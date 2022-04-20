pub mod geom {
    #[derive(Debug, Clone, PartialEq)]
    pub struct Vector {
        pub x: f64,
        pub y: f64,
    }

    impl Vector {
        pub fn zero() -> Vector {
            Vector { x: 0., y: 0. }
        }

        pub fn unit() -> Vector {
            Vector { x: 1., y: 1. }
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

        pub fn add(&mut self, other: &Vector) {
            self.x += other.x;
            self.y += other.y;
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

        pub fn len(&self) -> f64 {
            let distance = self.x.powi(2) + self.y.powi(2);
            return (distance as f64).sqrt();
        }
    }

    pub struct BoundingBox {
        top_left: Point,
        top_right: Point,
        bottom_left: Point,
        bottom_right: Point,
    }

    impl BoundingBox {
        pub fn create(center_x: u16, center_y: u16, width: u16, height: u16) -> BoundingBox {
            let top_left = Point {
                x: center_x as i32 - (width / 2) as i32,
                y: center_y as i32 + (height / 2) as i32,
            };
            let top_right = Point {
                x: center_x as i32 + (width / 2) as i32,
                y: center_y as i32 + (height / 2) as i32,
            };
            let bottom_left = Point {
                x: center_x as i32 - (width / 2) as i32,
                y: center_y as i32 - (height / 2) as i32,
            };
            let bottom_right = Point {
                x: center_x as i32 + (width / 2) as i32,
                y: center_y as i32 - (height / 2) as i32,
            };
            BoundingBox {
                top_left,
                top_right,
                bottom_left,
                bottom_right,
            }
        }

        pub fn points(&self) -> Vec<&Point> {
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

        pub fn is_point_within(&self, point: &Point) -> bool {
            return point.x >= self.top_left.x
                && point.x <= self.top_right.x
                && point.y <= self.top_left.y
                && point.y >= self.bottom_left.y;
        }
    }

    pub struct Point {
        pub x: i32,
        pub y: i32,
    }

    impl Point {
        pub fn create(x: i32, y: i32) -> Point {
            Point { x, y }
        }
    }
}
