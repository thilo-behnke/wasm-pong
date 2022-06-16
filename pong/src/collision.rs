pub mod collision {
    use crate::game_object::game_object::GameObject;
    use crate::utils::utils::{Logger, LoggerFactory};
    use std::cell::{RefCell};
    use std::collections::HashMap;
    use std::fmt::Debug;
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
                    collisions.push(Collision(obj.id(), other.id()))
                }
                if i >= objs.len() {
                    break;
                }
            }
            let registry = Collisions::new(collisions);
            return Box::new(registry);
        }
    }

    pub trait CollisionRegistry: Debug {
        fn get_collisions(&self) -> Vec<&Collision>;
        fn get_collisions_by_id(&self, id: u16) -> Vec<&Collision>;
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
        fn get_collisions_by_id(&self, id: u16) -> Vec<&Collision> {
            self.state
                .iter()
                .filter(|c| c.0 == id || c.1 == id)
                .collect()
        }
    }

    pub struct CollisionHandlerRegistry {
        handlers: HashMap<
            (String, String),
            fn(&Rc<RefCell<Box<dyn GameObject>>>, &Rc<RefCell<Box<dyn GameObject>>>),
        >,
    }

    type CollisionCallback = fn(&Rc<RefCell<Box<dyn GameObject>>>, &Rc<RefCell<Box<dyn GameObject>>>);

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
                &Rc<RefCell<Box<dyn GameObject>>>,
                &Rc<RefCell<Box<dyn GameObject>>>,
            ),
        ) -> bool {
            let regular = self.handlers.get(&mapping);
            if let Some(callback) = regular {
                callback(values.0, values.1);
                return true;
            }
            let inverse = self.handlers.get(&(mapping.clone().1, mapping.clone().0));
            if let Some(callback) = inverse {
                callback(values.1, values.0);
                return true;
            }
            return false;
        }
    }

    #[derive(Debug, Eq, PartialEq)]
    pub struct Collision(pub u16, pub u16);

    pub struct CollisionHandler {
        logger: Box<dyn Logger>,
        handlers: CollisionHandlerRegistry,
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
            callback: fn(&Rc<RefCell<Box<dyn GameObject>>>, &Rc<RefCell<Box<dyn GameObject>>>),
        ) {
            self.handlers.add(mapping, callback)
        }

        pub fn handle(
            &self,
            obj_a: &Rc<RefCell<Box<dyn GameObject>>>,
            obj_b: &Rc<RefCell<Box<dyn GameObject>>>,
        ) -> bool {
            let key = (
                RefCell::borrow(&obj_a).obj_type().to_string(),
                RefCell::borrow(&obj_b).obj_type().to_string(),
            );
            let handler_res = self.handlers.call(&key, (&obj_a, &obj_b));
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
}
