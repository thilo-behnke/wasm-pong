pub mod game_object {
    use std::fmt::Debug;
    use crate::game_object::components::{GeomComp, PhysicsComp};
    use crate::geom::geom::{BoundingBox, Vector};
    use crate::geom::shape::{Shape, ShapeType};

    pub trait GameObject : Debug {
        fn id(&self) -> u16;
        fn shape(&self) -> &Box<dyn Shape>;
        fn pos(&self) -> &Vector;
        fn pos_mut(&mut self) -> &mut Vector;
        fn orientation(&self) -> &Vector;
        fn update_pos(&mut self);
        fn bounding_box(&self) -> BoundingBox;
        fn vel(&self) -> &Vector;
        fn vel_mut(&mut self) -> &mut Vector;
        fn is_static(&self) -> bool;
    }

    // #[derive(Clone, Debug, PartialEq)]
    #[derive(Debug)]
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
        fn id(&self) -> u16 {
            self.id
        }

        fn shape(&self) -> &Box<dyn Shape> {
            self.geom.shape()
        }

        fn pos(&self) -> &Vector {
            self.geom.center()
        }

        fn pos_mut(&mut self) -> &mut Vector {
            self.geom.center_mut()
        }

        fn orientation(&self) -> &Vector {
            self.geom.orientation()
        }

        fn update_pos(&mut self) {
            let vel = self.vel().clone();
            let center = self.geom.center_mut();
            center.add(&vel);
            // Keep last orientation if vel is now zero.
            if vel == Vector::zero() {
                return;
            }
            let mut updated_orientation = vel.clone();
            updated_orientation.normalize();
            let orientation = self.geom.orientation_mut();
            orientation.x = updated_orientation.x;
            orientation.y = updated_orientation.y;
        }

        fn bounding_box(&self) -> BoundingBox {
            self.geom.bounding_box()
        }

        fn vel(&self) -> &Vector {
            &self.physics.vel()
        }

        fn vel_mut(&mut self) -> &mut Vector {
            self.physics.vel_mut()
        }

        fn is_static(&self) -> bool {
            self.physics.is_static()
        }
    }
}

pub mod components {
    use std::fmt::Debug;
    use crate::geom::geom::{BoundingBox, Vector};
    use crate::geom::shape::Shape;

    pub trait GeomComp : Debug {
        fn shape(&self) -> &Box<dyn Shape>;
        fn orientation(&self) -> &Vector;
        fn orientation_mut(&mut self) -> &mut Vector;
        fn center(&self) -> &Vector;
        fn center_mut(&mut self) -> &mut Vector;
        fn bounding_box(&self) -> BoundingBox;
    }

    #[derive(Debug)]
    pub struct DefaultGeomComp {
        shape: Box<dyn Shape>
    }

    impl DefaultGeomComp {
        pub fn new(shape: Box<dyn Shape>) -> DefaultGeomComp {
            DefaultGeomComp {shape}
        }
    }

    impl GeomComp for DefaultGeomComp {
        fn shape(&self) -> &Box<dyn Shape> {
            &self.shape
        }

        fn orientation(&self) -> &Vector {
            &self.shape.orientation()
        }

        fn orientation_mut(&mut self) -> &mut Vector {
            self.shape.orientation_mut()
        }

        fn center(&self) -> &Vector {
            &self.shape.center()
        }

        fn center_mut(&mut self) -> &mut Vector {
            self.shape.center_mut()
        }

        fn bounding_box(&self) -> BoundingBox {
            self.shape.bounding_box()
        }
    }

    pub trait PhysicsComp : Debug {
        fn vel(&self) -> &Vector;
        fn vel_mut(&mut self) -> &mut Vector;
        fn is_static(&self) -> bool;
    }

    #[derive(Debug)]
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
