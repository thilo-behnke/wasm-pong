pub mod detection {
    use crate::collision::collision::{Collision, CollisionRegistry, Collisions};
    use crate::game_object::game_object::GameObject;
    use crate::utils::utils::{Logger, LoggerFactory};
    use std::cell::RefCell;
    use std::rc::Rc;

    pub struct CollisionDetectorConfig {
        groups: Vec<CollisionGroup>,
    }

    impl CollisionDetectorConfig {
        pub fn new() -> CollisionDetectorConfig {
            CollisionDetectorConfig { groups: vec![] }
        }

        pub fn matches_any_group(&self, type_a: &str, type_b: &str) -> bool {
            self.groups.iter().any(|g| g.matches(type_a, type_b))
        }
    }

    #[derive(Debug)]
    pub struct CollisionGroup(pub String, pub String);

    impl CollisionGroup {
        pub fn matches(&self, type_a: &str, type_b: &str) -> bool {
            self.0 == type_a && self.1 == type_b || self.0 == type_b && self.1 == type_a
        }
    }

    pub struct CollisionDetector {
        config: CollisionDetectorConfig,
        #[allow(dead_code)]
        logger: Box<dyn Logger>,
    }

    impl CollisionDetector {
        pub fn new(logger_factory: &Box<dyn LoggerFactory>) -> CollisionDetector {
            let logger = logger_factory.get("collision_detector");
            CollisionDetector {
                config: CollisionDetectorConfig::new(),
                logger,
            }
        }

        pub fn set_groups(&mut self, groups: Vec<CollisionGroup>) {
            self.config.groups = groups;
        }

        pub fn detect_collisions(
            &self,
            objs: Vec<Rc<RefCell<Box<dyn GameObject>>>>,
        ) -> Box<dyn CollisionRegistry> {
            if objs.is_empty() {
                return Box::new(Collisions::new(vec![]));
            }
            let mut collisions: Vec<Collision> = vec![];
            let mut i = 0;
            loop {
                let o = &objs[i];
                let obj = RefCell::borrow(o);
                i += 1;

                let rest = &objs[i..];
                for other in rest.iter().map(|o| o.borrow()) {
                    if !self
                        .config
                        .matches_any_group(obj.obj_type(), other.obj_type())
                    {
                        // self.logger.log(&*format!("objs {} and {} do not match any group: {:?}", obj.obj_type(), other.obj_type(), self.config.groups));
                        continue;
                    }
                    let has_collision = obj.bounding_box().overlaps(&other.bounding_box());
                    if !has_collision {
                        continue;
                    }
                    collisions.push(Collision(obj.id().to_owned(), other.id().to_owned()))
                }
                if i >= objs.len() {
                    break;
                }
            }
            let registry = Collisions::new(collisions);
            return Box::new(registry);
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::collision::collision::Collision;
        use crate::collision::detection::{CollisionDetector, CollisionGroup};
        use crate::game_object::game_object::GameObject;
        use crate::geom::shape::ShapeType;
        use crate::geom::utils::BoundingBox;
        use crate::geom::vector::Vector;
        use crate::utils::utils::DefaultLoggerFactory;
        use rstest::rstest;
        use std::cell::RefCell;
        use std::rc::Rc;

        #[rstest]
        #[case(vec![], vec![])]
        #[case(
        vec![
        MockGameObject::new("1", "a", BoundingBox::create(&Vector{x: 50., y: 50.}, 20., 20.)),
        MockGameObject::new("2", "b", BoundingBox::create(&Vector{x: 50., y: 50.}, 20., 20.))
        ],
        vec![Collision::new("1", "2")]
        )]
        #[case(
        vec![
        MockGameObject::new("1", "a", BoundingBox::create(&Vector{x: 60., y: 65.}, 20., 20.)),
        MockGameObject::new("2", "b", BoundingBox::create(&Vector{x: 50., y: 50.}, 20., 20.)),
        ],
        vec![Collision::new("1", "2")]
        )]
        #[case(
        vec![
        MockGameObject::new("1", "a", BoundingBox::create(&Vector{x: 50., y: 50.}, 20., 20.)),
        MockGameObject::new("2", "b", BoundingBox::create(&Vector{x: 80., y: 80.}, 20., 20.)),
        ],
        vec![]
        )]
        #[case(
        vec![
        MockGameObject::new("1", "a", BoundingBox::create(&Vector{x: 50., y: 50.}, 50., 50.)),
        MockGameObject::new("2", "b", BoundingBox::create(&Vector{x: 500., y: 50.}, 50., 50.)),
        ],
        vec![]
        )]
        #[case(
        vec![
        MockGameObject::new("1", "a", BoundingBox::create(&Vector{x: 60., y: 65.}, 20., 20.)),
        MockGameObject::new("2", "c", BoundingBox::create(&Vector{x: 50., y: 50.}, 20., 20.)),
        ],
        vec![]
        )]
        pub fn should_detect_collisions(
            #[case] objs: Vec<Rc<RefCell<Box<dyn GameObject>>>>,
            #[case] expected_collisions: Vec<Collision>,
        ) {
            let logger = DefaultLoggerFactory::noop();
            let mut detector = CollisionDetector::new(&logger);
            detector.set_groups(vec![CollisionGroup(String::from("a"), String::from("b"))]);
            let res = detector.detect_collisions(objs);
            assert_eq!(
                res.get_collisions(),
                expected_collisions.iter().collect::<Vec<&Collision>>()
            );
        }

        #[derive(Debug)]
        pub struct MockGameObject {
            id: String,
            obj_type: String,
            bounding_box: BoundingBox,
        }

        impl MockGameObject {
            pub fn new(
                id: &str,
                obj_type: &str,
                bounding_box: BoundingBox,
            ) -> Rc<RefCell<Box<dyn GameObject>>> {
                Rc::new(RefCell::new(Box::new(MockGameObject {
                    id: id.to_owned(),
                    obj_type: String::from(obj_type),
                    bounding_box,
                })))
            }
        }

        impl GameObject for MockGameObject {
            fn id(&self) -> &str {
                &self.id
            }

            fn obj_type(&self) -> &str {
                &*self.obj_type
            }

            fn shape(&self) -> &ShapeType {
                todo!()
            }

            fn pos(&self) -> &Vector {
                todo!()
            }

            fn pos_mut(&mut self) -> &mut Vector {
                todo!()
            }

            fn orientation(&self) -> &Vector {
                todo!()
            }

            fn orientation_mut(&mut self) -> &mut Vector {
                todo!()
            }

            fn update_pos(&mut self, _ms_diff: f64) {
                todo!()
            }

            fn bounding_box(&self) -> BoundingBox {
                self.bounding_box.clone()
            }

            fn vel(&self) -> &Vector {
                todo!()
            }

            fn vel_mut(&mut self) -> &mut Vector {
                todo!()
            }

            fn is_static(&self) -> bool {
                todo!()
            }

            fn is_dirty(&self) -> bool {
                todo!()
            }

            fn set_dirty(&mut self, _is_dirty: bool) {
                todo!()
            }
        }
    }
}

pub mod handler {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;
    use crate::game_object::game_object::GameObject;
    use crate::utils::utils::{Logger, LoggerFactory};

    pub struct CollisionHandler {
        logger: Box<dyn Logger>,
        handlers: CollisionHandlerRegistry,
    }

    pub struct FieldStats {
        pub dimensions: (f64, f64)
    }

    impl CollisionHandler {
        pub fn new(logger_factory: &Box<dyn LoggerFactory>) -> CollisionHandler {
            let logger = logger_factory.get("collision_handler");
            CollisionHandler {
                logger,
                handlers: CollisionHandlerRegistry::new(),
            }
        }

        pub fn register(
            &mut self,
            mapping: (String, String),
            callback: fn(&FieldStats, &Rc<RefCell<Box<dyn GameObject>>>, &Rc<RefCell<Box<dyn GameObject>>>),
        ) {
            self.handlers.add(mapping, callback)
        }

        pub fn handle(
            &self,
            stats: &FieldStats,
            obj_a: &Rc<RefCell<Box<dyn GameObject>>>,
            obj_b: &Rc<RefCell<Box<dyn GameObject>>>,
        ) -> bool {
            let key = (
                RefCell::borrow(&obj_a).obj_type().to_string(),
                RefCell::borrow(&obj_b).obj_type().to_string(),
            );
            let handler_res = self.handlers.call(&key, (&stats, &obj_a, &obj_b));
            if !handler_res {
                self.logger
                    .log(&*format!("Found no matching collision handler: {:?}", key));
                return false;
            }
            return true;
        }

        // pub fn new() -> CollisionHandler {
        //     CollisionHandler {}
        // }
        // pub fn handle(&self, obj_a: &mut Box<dyn GameObject>, obj_b: &Box<dyn GameObject>) {
        //     if !obj_a.is_static() {
        //         obj_a.vel_mut().reflect(&obj_b.orientation());
        //         if *obj_b.vel() != Vector::zero() {
        //             let mut adjusted = obj_b.vel().clone();
        //             adjusted.normalize();
        //             obj_a.vel_mut().add(&adjusted);
        //         }
        //     }
        //     let mut b_to_a = obj_a.pos().clone();
        //     b_to_a.sub(&obj_b.pos());
        //     b_to_a.normalize();
        //     obj_a.pos_mut().add(&b_to_a);
        // }
    }

    pub struct CollisionHandlerRegistry {
        handlers: HashMap<
            (String, String),
            fn(&FieldStats, &Rc<RefCell<Box<dyn GameObject>>>, &Rc<RefCell<Box<dyn GameObject>>>),
        >,
    }

    type CollisionCallback =
    fn(&FieldStats, &Rc<RefCell<Box<dyn GameObject>>>, &Rc<RefCell<Box<dyn GameObject>>>);

    impl CollisionHandlerRegistry {
        pub fn new() -> CollisionHandlerRegistry {
            CollisionHandlerRegistry {
                handlers: HashMap::new(),
            }
        }

        pub fn add(&mut self, mapping: (String, String), callback: CollisionCallback) {
            if self.handlers.contains_key(&mapping) {
                panic!(
                    "Collision handler for mapping {:?} is already registered.",
                    mapping
                )
            }
            self.handlers.insert(mapping, callback);
        }

        pub fn call(
            &self,
            mapping: &(String, String),
            values: (
                &FieldStats,
                &Rc<RefCell<Box<dyn GameObject>>>,
                &Rc<RefCell<Box<dyn GameObject>>>,
            ),
        ) -> bool {
            let regular = self.handlers.get(&mapping);
            if let Some(callback) = regular {
                callback(values.0, values.1, values.2);
                return true;
            }
            let inverse = self.handlers.get(&(mapping.clone().1, mapping.clone().0));
            if let Some(callback) = inverse {
                callback(values.0, values.2, values.1);
                return true;
            }
            return false;
        }
    }

    #[cfg(test)]
    mod tests {
        use rstest::rstest;
        use std::cell::RefCell;
        use std::rc::Rc;
        use crate::collision::handler::{CollisionHandler, FieldStats};
        use crate::game_object::components::{DefaultGeomComp, DefaultPhysicsComp};
        use crate::game_object::game_object::{DefaultGameObject, GameObject};
        use crate::geom::shape::Shape;
        use crate::geom::vector::Vector;
        use crate::utils::utils::DefaultLoggerFactory;

        #[rstest]
        #[case(
        create_game_obj("1", Vector::new(1., 0.), Vector::new(1., 0.), true),
        create_game_obj("2", Vector::new(0., 0.), Vector::new(0., 1.), true)
        )]
        #[case(
        create_game_obj("1", Vector::new(1., 0.), Vector::new(1., 0.), false),
        create_game_obj("2", Vector::new(0., 0.), Vector::new(0., 1.), true)
        )]
        #[case(
        create_game_obj("1", Vector::new(-1., 0.), Vector::new(-1., 0.), false),
        create_game_obj("2", Vector::new(0., 0.), Vector::new(0., 1.), true),
        )]
        #[case(
        create_game_obj("1", Vector::new(1., 1.), Vector::new(1., 1.), false),
        create_game_obj("2", Vector::new(0., 0.), Vector::new(0., 1.), true)
        )]
        #[case(
        create_game_obj("1", Vector::new(-2., 1.), Vector::new(-1., 0.), false),
        create_game_obj("2", Vector::new(0., 0.), Vector::new(0., 1.), true),
        )]
        #[case(
        create_game_obj("1", Vector::new(1., 0.), Vector::new(1., 0.), false),
        create_game_obj("2", Vector::new(0., 1.), Vector::new(0., 1.), true)
        )]
        #[case(
        create_game_obj("1", Vector::new(-2., 1.), Vector::new(-1., 0.), false),
        create_game_obj("2", Vector::new(0., 0.), Vector::new(0., 1.), true),
        )]
        pub fn should_handle_collision(
            #[case] obj_a: Rc<RefCell<Box<dyn GameObject>>>,
            #[case] obj_b: Rc<RefCell<Box<dyn GameObject>>>,
        ) {
            let logger = DefaultLoggerFactory::noop();
            let mut handler = CollisionHandler::new(&logger);
            let field_stats = FieldStats {
                dimensions: (1000., 1000.)
            };
            handler.register((String::from("obj"), String::from("obj")), |_stats, _a, _b| {
                let mut a_mut = RefCell::borrow_mut(_a);
                let mut vel_inverted = a_mut.vel().clone();
                vel_inverted.invert();
                *a_mut.vel_mut() = vel_inverted;
            });
            let expected_vel_a = Vector::inverted(RefCell::borrow(&obj_a).vel());
            let res = handler.handle(&field_stats, &obj_a, &obj_b);
            assert_eq!(true, res);
            assert_eq!(RefCell::borrow(&obj_a).pos(), RefCell::borrow(&obj_a).pos());
            assert_eq!(RefCell::borrow(&obj_a).vel(), &expected_vel_a);
            assert_eq!(RefCell::borrow(&obj_b).pos(), RefCell::borrow(&obj_b).pos());
            assert_eq!(RefCell::borrow(&obj_a).vel(), RefCell::borrow(&obj_a).vel());
        }

        fn create_game_obj(
            id: &str,
            vel: Vector,
            orientation: Vector,
            is_static: bool,
        ) -> Rc<RefCell<Box<dyn GameObject>>> {
            Rc::new(RefCell::new(Box::new(DefaultGameObject::new(
                id,
                "obj".to_string(),
                Box::new(DefaultGeomComp::new(Shape::rect(
                    Vector::zero(),
                    orientation,
                    20.,
                    20.,
                ))),
                Box::new(DefaultPhysicsComp::new(vel, is_static)),
            ))))
        }
    }
}

pub mod collision {
    use std::fmt::Debug;

    pub trait CollisionRegistry: Debug {
        fn get_collisions(&self) -> Vec<&Collision>;
        fn get_collisions_by_id(&self, id: &str) -> Vec<&Collision>;
    }

    #[derive(Debug)]
    pub struct Collisions {
        pub state: Vec<Collision>,
    }

    impl Collisions {
        pub fn new(collisions: Vec<Collision>) -> Collisions {
            Collisions { state: collisions }
        }
    }

    impl CollisionRegistry for Collisions {
        fn get_collisions(&self) -> Vec<&Collision> {
            self.state.iter().collect()
        }
        fn get_collisions_by_id(&self, id: &str) -> Vec<&Collision> {
            self.state
                .iter()
                .filter(|c| c.0 == id || c.1 == id)
                .collect()
        }
    }

    #[derive(Debug, Eq, PartialEq)]
    pub struct Collision(pub String, pub String);

    impl Collision {
        pub fn new(obj_a: &str, obj_b: &str) -> Collision {
            Collision(obj_a.to_owned(), obj_b.to_owned())
        }
    }

}
