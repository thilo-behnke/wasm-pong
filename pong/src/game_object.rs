pub mod game_object {
    use std::fmt::Debug;
    use crate::game_object::components::{GeomComp, PhysicsComp};
    use crate::geom::geom::{BoundingBox, Vector};
    use crate::geom::shape::ShapeType;

    pub trait GameObject {
        fn update_pos(&mut self);
        fn vel(&self) -> &Vector;
        fn set_vel_x(&mut self, x: f64);
        fn set_vel_y(&mut self, y: f64);
        fn bounding_box(&self) -> BoundingBox;
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct DefaultGameObject {
        pub id: u16,
        geom: Box<dyn GeomComp>,
        physics: Box<dyn PhysicsComp>
    }

    impl DefaultGameObject {
        pub fn new(id: u16, geom: Box<dyn GeomComp>, physics: Box<dyn PhysicsComp>) -> DefaultGameObject {
            DefaultGameObject {id, geom, physics}
        }
    }

    impl GameObject for DefaultGameObject {
        fn update_pos(&mut self) {
            let vel = self.vel();
            let center = self.geom.center_mut();
            center.add(vel);
            // Keep last orientation if vel is now zero.
            if *vel == Vector::zero() {
                return;
            }
            let mut updated_orientation = vel.clone();
            updated_orientation.normalize();
            let orientation = self.geom.orientation_mut();
            orientation.x = updated_orientation.x;
            orientation.y = updated_orientation.y;
        }

        fn vel(&self) -> &Vector {
            &self.physics.vel()
        }

        fn set_vel_x(&mut self, x: f64) {
            let vel = self.physics.vel_mut();
            vel.x = x;
        }

        fn set_vel_y(&mut self, y: f64) {
            let vel = self.physics.vel_mut();
            vel.y = y;
        }

        fn bounding_box(&self) -> BoundingBox {
            self.geom.bounding_box()
        }
    }
}

pub mod components {
    use crate::geom::geom::{BoundingBox, Vector};
    use crate::geom::shape::Shape;

    pub trait GeomComp {
        fn orientation(&self) -> &Vector;
        fn orientation_mut(&mut self) -> &mut Vector;
        fn center(&self) -> &Vector;
        fn center_mut(&mut self) -> &mut Vector;
        fn bounding_box(&self) -> BoundingBox;
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

        fn bounding_box(&self) -> BoundingBox {
            self.bounding_box()
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

        pub fn new_static() -> DefaultPhysicsComp {
            DefaultPhysicsComp::new(
                Vector::zero(),
                true
            )
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
