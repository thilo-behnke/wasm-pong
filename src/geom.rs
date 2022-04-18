pub mod geom {
    #[derive(Debug, Clone, PartialEq)]
    pub struct Vector {
        pub x: i32,
        pub y: i32
    }

    impl Vector {
        pub fn zero() -> Vector {
            Vector {x: 0, y: 0}
        }

        pub fn unit() -> Vector {
            Vector {x: 1, y: 1}
        }

        pub fn normalize(&mut self) {
            let length = self.len();
            self.x /= length;
            self.y /= length;
        }

        pub fn invert(&mut self) {
            self.x = self.x * -1;
            self.y = self.y * -1;
        }

        pub fn len(&self) -> i32 {
            let distance = self.x.pow(2) + self.y.pow(2);
            return (distance as f32).sqrt() as i32;
        }
    }

    pub struct BoundingBox {
        top_left: Point,
        top_right: Point,
        bottom_left: Point,
        bottom_right: Point
    }

    impl BoundingBox {
        pub fn create(center_x: u16, center_y: u16, width: u16, height: u16) -> BoundingBox {
            let top_left = Point {x: center_x as i16 - (width / 2) as i16, y: center_y as i16 + (height / 2) as i16};
            let top_right = Point {x: center_x as i16 + (width / 2) as i16, y: center_y as i16 + (height / 2) as i16};
            let bottom_left = Point {x: center_x as i16 - (width / 2) as i16, y: center_y as i16 - (height / 2) as i16};
            let bottom_right = Point {x: center_x as i16 + (width / 2) as i16, y: center_y as i16 - (height / 2) as i16};
            BoundingBox {
                top_left, top_right, bottom_left, bottom_right
            }
        }

        pub fn points(&self) -> Vec<&Point> {
            return vec![
                &self.top_left, &self.top_right, &self.bottom_left, &self.bottom_right
            ]
        }

        pub fn overlaps(&self, other: &BoundingBox) -> bool {
            return other.points().iter().any(|p| self.is_point_within(p))
        }

        pub fn is_point_within(&self, point: &Point) -> bool {
            return point.x >= self.top_left.x && point.y <= self.top_left.y && point.y >= self.bottom_left.y
        }
    }

    pub struct Point {
        pub x: i16,
        pub y: i16
    }

    impl Point {
        pub fn create(x: i16, y: i16) -> Point {
            Point { x, y }
        }
    }
}
