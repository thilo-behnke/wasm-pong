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
        pub pos: Vector,
        pub orientation: Vector,
        pub shape: Shape,
        pub shape_params: Vec<u16>,
        pub vel: Vector,
        pub is_static: bool,
    }

    impl GameObject {

        pub fn update_pos(&mut self) {
            self.pos.add(&self.vel);
            // Keep last orientation if vel is now zero.
            if self.vel == Vector::zero() {
                return;
            }
            let mut orientation = self.vel.clone();
            orientation.normalize();
            self.orientation = orientation;
        }

        pub fn set_vel_x(&mut self, x: f64) {
            self.vel.x = x
        }

        pub fn set_vel_y(&mut self, y: f64) {
            self.vel.y = y
        }

        pub fn bounding_box(&self) -> BoundingBox {
            match self.shape {
                Shape::Rect => {
                    BoundingBox::create(&self.pos, self.shape_params[0], self.shape_params[1])
                }
                Shape::Circle => {
                    BoundingBox::create(&self.pos, self.shape_params[0] * 2, self.shape_params[0] * 2)
                }
            }
        }
    }
}
