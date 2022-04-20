pub mod game_object {
    use crate::geom::geom::{BoundingBox, Vector};

    #[derive(Clone, Debug, PartialEq)]
    pub enum Shape {
        Rect = 0,
        Circle = 1,
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct GameObject {
        pub id: u16,
        pub x: u16,
        pub y: u16,
        pub shape: Shape,
        pub shape_params: Vec<u16>,
        pub vel: Vector,
        pub is_static: bool,
    }

    impl GameObject {
        // TODO: Migrate pos to f64
        pub fn update_pos(&mut self, field_width: u16, field_height: u16) {
            let abs_x = self.vel.x.abs() as u16;
            let updated_x = match self.vel.x {
                n if n >= 0. => self.x.wrapping_add(abs_x),
                _ => self.x.wrapping_sub(abs_x)
            };
            let abs_y = self.vel.y.abs() as u16;
            let updated_y = match self.vel.y {
                n if n >= 0. => self.y.wrapping_add(abs_y),
                _ => self.y.wrapping_sub(abs_y)
            };
            self.x = updated_x;
            self.y = updated_y;
        }

        pub fn set_vel_x(&mut self, x: f64) {
            self.vel.x = x
        }

        pub fn set_vel_y(&mut self, y: f64) {
            self.vel.y = y
        }

        pub fn bounding_box(&self) -> BoundingBox {
            self.bounding_box_from(self.x, self.y)
        }

        fn bounding_box_from(&self, x: u16, y: u16) -> BoundingBox {
            match self.shape {
                Shape::Rect => {
                    BoundingBox::create(x, y, self.shape_params[0], self.shape_params[1])
                }
                Shape::Circle => {
                    BoundingBox::create(x, y, self.shape_params[0] * 2, self.shape_params[0] * 2)
                }
            }
        }
    }
}
