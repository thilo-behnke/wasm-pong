pub mod collision {
    use crate::game_object::game_object::GameObject;
    use crate::geom::geom::Vector;
    use std::alloc::handle_alloc_error;
    use std::cell::{Ref, RefCell, RefMut};
    use std::collections::HashMap;
    use std::fmt::Debug;
    use std::rc::Rc;

    pub struct CollisionDetector {}

    impl CollisionDetector {
        pub fn new() -> CollisionDetector {
            CollisionDetector {}
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

    #[derive(Debug, Eq, PartialEq)]
    pub struct Collision(pub u16, pub u16);

    #[derive(Clone)]
    pub struct CollisionHandler {
        handlers: HashMap<(String, String), fn(Rc<RefCell<Box<dyn GameObject>>>, Rc<RefCell<Box<dyn GameObject>>>)>
    }

    impl CollisionHandler {
        pub fn new() -> CollisionHandler {
            CollisionHandler {
                handlers: HashMap::new(),
            }
        }

        pub fn register(
            &mut self,
            mapping: (String, String),
            callback: fn(Rc<RefCell<Box<dyn GameObject>>>, Rc<RefCell<Box<dyn GameObject>>>),
        ) {
            if self.handlers.contains_key(&mapping) {
                panic!(
                    "Collision handler for mapping {:?} is already registered.",
                    mapping
                )
            }
            self.handlers.insert(mapping, callback);
        }

        pub fn handle(
            &self,
            obj_a: Rc<RefCell<Box<dyn GameObject>>>,
            obj_b: Rc<RefCell<Box<dyn GameObject>>>,
        ) -> bool {
            let key = (RefCell::borrow(&obj_a).obj_type().to_string(), RefCell::borrow(&obj_b).obj_type().to_string());
            if !self.handlers.contains_key(&key) {
                return false;
            }
            let handler = self.handlers[&key];
            handler(obj_a, obj_b);
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
