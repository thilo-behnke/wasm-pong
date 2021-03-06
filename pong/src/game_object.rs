pub mod game_object {
    use crate::game_object::components::{GeomComp, PhysicsComp};
    use crate::geom::shape::ShapeType;
    use crate::geom::utils::BoundingBox;
    use crate::geom::vector::Vector;
    use std::fmt::Debug;

    pub trait GameObject: Debug {
        fn id(&self) -> &str;
        fn obj_type(&self) -> &str;
        fn shape(&self) -> &ShapeType;
        fn pos(&self) -> &Vector;
        fn pos_mut(&mut self) -> &mut Vector;
        fn orientation(&self) -> &Vector;
        fn orientation_mut(&mut self) -> &mut Vector;
        fn update_pos(&mut self, ms_diff: f64);
        fn bounding_box(&self) -> BoundingBox;
        fn vel(&self) -> &Vector;
        fn vel_mut(&mut self) -> &mut Vector;
        fn is_static(&self) -> bool;
        fn is_dirty(&self) -> bool;
        fn set_dirty(&mut self, is_dirty: bool);
    }

    #[derive(Debug)]
    pub struct DefaultGameObject {
        pub id: String,
        pub obj_type: String,
        geom: Box<dyn GeomComp>,
        physics: Box<dyn PhysicsComp>,
        dirty: bool,
    }

    impl DefaultGameObject {
        pub fn new(
            id: &str,
            obj_type: String,
            geom: Box<dyn GeomComp>,
            physics: Box<dyn PhysicsComp>,
        ) -> DefaultGameObject {
            DefaultGameObject {
                id: id.to_owned(),
                obj_type,
                geom,
                physics,
                dirty: false,
            }
        }
    }

    impl GameObject for DefaultGameObject {
        fn id(&self) -> &str {
            &self.id
        }

        fn obj_type(&self) -> &str {
            &self.obj_type
        }

        fn shape(&self) -> &ShapeType {
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

        fn orientation_mut(&mut self) -> &mut Vector {
            self.geom.orientation_mut()
        }

        fn update_pos(&mut self, ms_diff: f64) {
            // Keep last orientation if vel is now zero.
            if self.vel() == &Vector::zero() {
                return;
            }
            let mut vel = self.vel().clone();
            vel.scalar_multiplication(ms_diff);
            let center = self.geom.center_mut();
            center.add(&vel);
            let mut updated_orientation = vel.clone();
            updated_orientation.normalize();
            let orientation = self.geom.orientation_mut();
            orientation.x = updated_orientation.x;
            orientation.y = updated_orientation.y;

            self.dirty = true;
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

        fn is_dirty(&self) -> bool {
            return self.dirty;
        }

        fn set_dirty(&mut self, is_dirty: bool) {
            self.dirty = is_dirty;
        }
    }

    #[cfg(test)]
    mod tests {
        use rstest::rstest;
        use crate::game_object::components::{DefaultGeomComp, DefaultPhysicsComp};
        use crate::game_object::game_object::{DefaultGameObject, GameObject};
        use crate::geom::shape::Shape;
        use crate::geom::vector::Vector;

        #[rstest]
        #[case(Vector::new(100., 100.), Vector::new(-1., 1.), Vector::new(99.9, 100.1), 0.1)]
        #[case(Vector::new(300., 400.), Vector::new(-5., 0.), Vector::new(299.5, 400.), 0.1)]
        #[case(Vector::new(300., 400.), Vector::new(-5., 0.), Vector::new(299.935, 400.), 0.013)]
        pub fn should_update_pos(
            #[case] start_pos: Vector,
            #[case] vel: Vector,
            #[case] expected_pos: Vector,
            #[case] ms_diff: f64,
        ) {
            let mut obj = DefaultGameObject::new(
                "1",
                "obj".to_string(),
                Box::new(DefaultGeomComp::new(Shape::rect(
                    Vector::new(start_pos.x as f64, start_pos.y as f64),
                    Vector::new(1., 0.),
                    0.,
                    0.,
                ))),
                Box::new(DefaultPhysicsComp::new(vel, false)),
            );
            obj.update_pos(ms_diff);
            assert_eq!(*obj.pos(), expected_pos);
        }
    }
}

pub mod components {
    use crate::geom::shape::{
        get_bounding_box, get_center, get_center_mut, get_orientation, get_orientation_mut,
        ShapeType,
    };
    use crate::geom::utils::BoundingBox;
    use crate::geom::vector::Vector;
    use std::fmt::Debug;

    pub trait GeomComp: Debug {
        fn shape(&self) -> &ShapeType;
        fn orientation(&self) -> &Vector;
        fn orientation_mut(&mut self) -> &mut Vector;
        fn center(&self) -> &Vector;
        fn center_mut(&mut self) -> &mut Vector;
        fn bounding_box(&self) -> BoundingBox;
    }

    #[derive(Debug)]
    pub struct DefaultGeomComp {
        shape: ShapeType,
    }

    impl DefaultGeomComp {
        pub fn new(shape: ShapeType) -> DefaultGeomComp {
            DefaultGeomComp { shape }
        }
    }

    impl GeomComp for DefaultGeomComp {
        fn shape(&self) -> &ShapeType {
            &self.shape
        }

        fn orientation(&self) -> &Vector {
            get_orientation(&self.shape)
        }

        fn orientation_mut(&mut self) -> &mut Vector {
            get_orientation_mut(&mut self.shape)
        }

        fn center(&self) -> &Vector {
            get_center(&self.shape)
        }

        fn center_mut(&mut self) -> &mut Vector {
            get_center_mut(&mut self.shape)
        }

        fn bounding_box(&self) -> BoundingBox {
            get_bounding_box(&self.shape)
        }
    }

    pub trait PhysicsComp: Debug {
        fn vel(&self) -> &Vector;
        fn vel_mut(&mut self) -> &mut Vector;
        fn is_static(&self) -> bool;
    }

    #[derive(Debug)]
    pub struct DefaultPhysicsComp {
        vel: Vector,
        is_static: bool,
    }
    impl DefaultPhysicsComp {
        pub fn new(vel: Vector, is_static: bool) -> DefaultPhysicsComp {
            DefaultPhysicsComp { vel, is_static }
        }

        pub fn new_static() -> DefaultPhysicsComp {
            DefaultPhysicsComp::new(Vector::zero(), true)
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
