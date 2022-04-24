pub mod game_object {
    use crate::game_object::components::{GeomComp, PhysicsComp};
    use crate::geom::geom::{BoundingBox, Vector};
    use crate::geom::shape::ShapeType;

    #[derive(Clone, Debug, PartialEq)]
    pub struct GameObject {
        pub id: u16,
        pub geom: Box<dyn GeomComp>,
        pub physics: Box<dyn PhysicsComp>
    }

    impl GameObject {
        pub fn update_pos(&mut self) {
            let center = self.geom.center_mut();
            center.add(&self.vel);
            // Keep last orientation if vel is now zero.
            let vel = self.physics.vel();
            if vel == Vector::zero() {
                return;
            }
            let mut orientation = vel.clone();
            orientation.normalize();
            self.orientation = orientation;
        }

        pub fn set_vel_x(&mut self, x: f64) {
            let vel = self.physics.vel_mut();
            vel.x = x;
        }

        pub fn set_vel_y(&mut self, y: f64) {
            let vel = self.physics.vel_mut();
            vel.y = y;
        }

        pub fn bounding_box(&self) -> BoundingBox {
            self.geom.shape().bounding_box()
        }
    }
}

pub mod components {
    use crate::geom::geom::Vector;
    use crate::geom::shape::Shape;

    pub trait GeomComp {
        fn orientation(&self) -> &Vector;
        fn orientation_mut(&mut self) -> &mut Vector;
        fn center(&self) -> &Vector;
        fn center_mut(&mut self) -> &mut Vector;
    }

    pub struct DefaultGeomComp {
        shape: Box<dyn Shape>
    }
    impl DefaultGeomComp {
        pub fn new(shape: Box<dyn Shape>) -> DefaultGeomComp {
            DefaultGeomComp {shape}
        }
    }
    impl GeomComp for DefaultGeomComp {
        fn orientation(&self) -> &Vector {
            &self.shape.orientation()
        }

        fn orientation_mut(&mut self) -> &mut Vector {
            &mut self.shape.orientation()
        }

        fn center(&self) -> &Vector {
            &self.shape.center()
        }

        fn center_mut(&mut self) -> &mut Vector {
            &mut self.shape.center()
        }
    }

    pub trait PhysicsComp {
        fn vel(&self) -> &Vector;
        fn vel_mut(&mut self) -> &mut Vector;
        fn is_static(&self) -> bool;
    }

    pub struct DefaultPhysicsComp {
        vel: Vector,
        is_static: bool
    }
    impl DefaultPhysicsComp {
        pub fn new(vel: Vector, is_static: bool) -> DefaultPhysicsComp {
            DefaultPhysicsComp {vel, is_static}
        }
    }
    impl PhysicsComp for DefaultPhysicsComp {
        fn vel(&self) -> &Vector {
            &self.vel
        }

        fn vel_mut(&mut self) -> &mut Vector {
            &mut self.vel
        }

        fn is_static(&self) -> bool {
            self.is_static
        }
    }
}
