pub mod collision {
    use crate::GameObject;

    pub struct CollisionDetector {}

    impl CollisionDetector {
        pub fn new() -> CollisionDetector {
            CollisionDetector {}
        }

        pub fn detect_collisions(&self, objs: Vec<&GameObject>) -> Box<dyn CollisionRegistry> {
            if objs.is_empty() {
                return Box::new(Collisions::new(vec![]));
            }
            let mut collisions: Vec<Collision> = vec![];
            let mut i = 0;
            loop {
                let obj = objs[i];
                i += 1;

                let rest = &objs[i..];
                for other in rest.iter() {
                    let has_collision = obj.bounding_box().overlaps(&other.bounding_box());
                    if !has_collision {
                        continue;
                    }
                    collisions.push(Collision(obj.id, other.id))
                }
                if i >= objs.len() {
                    break;
                }
            }
            let registry = Collisions::new(collisions);
            return Box::new(registry);
        }
    }

    pub trait CollisionRegistry {
        fn get_collisions(&self) -> Vec<&Collision>;
        fn get_collisions_by_id(&self, id: u16) -> Vec<&Collision>;
    }

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
             self.state.iter().filter(|c| c.0 == id || c.1 == id).collect()
        }
    }

    #[derive(Debug, Eq, PartialEq)]
    pub struct Collision(pub u16, pub u16);
}
