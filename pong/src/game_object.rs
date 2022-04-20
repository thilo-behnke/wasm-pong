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
        pub fn update_pos(&mut self, field_width: u16, field_height: u16) {
            let updated_x = self.x.wrapping_add(self.vel.x as u16);
            let updated_y = self.y.wrapping_add(self.vel.y as u16);
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
