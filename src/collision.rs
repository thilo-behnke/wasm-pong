
pub mod collision {
    use std::collections::HashMap;
    use crate::GameObject;

    pub struct CollisionDetector {}

    impl CollisionDetector {
        pub fn new() -> CollisionDetector {
            CollisionDetector {}
        }

        pub fn detect_collisions(objs: Vec<GameObject>) -> Box<dyn CollisionRegistry> {
            let registry = Collisions::new(HashMap::new());
            return Box::new(registry);
        }
    }

    trait CollisionRegistry {
        fn get_collisions(&self) -> Vec<&Collision>;
        // fn get_collisions_by_id() -> Vec<Collision>;
    }

    pub struct Collisions {
        state: HashMap<String, Vec<Collision>>
    }

    impl Collisions {
        pub fn new(collisions: HashMap<String, Vec<Collision>>) -> Collisions {
            Collisions {
                state: collisions
            }
        }
    }

    impl CollisionRegistry for Collisions {
        fn get_collisions(&self) -> Vec<&Collision> {
            self.state.values().flatten().collect()
        }
    }

    pub struct Collision(GameObject, GameObject);
}
