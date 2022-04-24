use rstest::rstest;
use pong::collision::collision::{Collision, CollisionDetector};
use pong::game_object::game_object::GameObject;
use pong::geom::geom::{BoundingBox, Vector};


#[rstest]
#[case(vec![], vec![])]
#[case(
    vec![
        MockGameObject::new(1, BoundingBox::new(Vector{x: 50., y: 50.}, 20., 20.)),
        MockGameObject::new(2, BoundingBox::new(Vector{x: 50., y: 50.}, 20., 20.))
    ],
    vec![Collision(1, 2)]
)]
#[case(
    vec![
        MockGameObject::new(1, BoundingBox::new(Vector{x: 60., y: 65.}, 20., 20.)),
        MockGameObject::new(2, BoundingBox::new(Vector{x: 50., y: 50.}, 20., 20.)),
    ],
    vec![Collision(1, 2)]
)]
#[case(
    vec![
        MockGameObject::new(1, BoundingBox::new(Vector{x: 50., y: 50.}, 20., 20.)),
        MockGameObject::new(2, BoundingBox::new(Vector{x: 80., y: 80.}, 20., 20.)),
    ],
    vec![]
)]
#[case(
    vec![
        MockGameObject::new(1, BoundingBox::new(Vector{x: 50., y: 50.}, 50., 50.)),
        MockGameObject::new(2, BoundingBox::new(Vector{x: 500., y: 50.}, 50., 50.)),
    ],
    vec![]
)]
pub fn should_detect_collisions(
    #[case] objs: Vec<Box<dyn GameObject>>,
    #[case] expected_collisions: Vec<Collision>,
) {
    let detector = CollisionDetector::new();
    let res = detector.detect_collisions(objs.iter().collect());
    assert_eq!(
        res.get_collisions(),
        expected_collisions.iter().collect::<Vec<&Collision>>()
    );
}

#[derive(Debug)]
pub struct MockGameObject {
    id: u16,
    bounding_box: BoundingBox
}

impl MockGameObject {
    pub fn new(id: u16, bounding_box: BoundingBox) -> Box<dyn GameObject> {
        Box::new(
            MockGameObject {
                id, bounding_box
            }
        )
    }
}

impl GameObject for MockGameObject {
    fn id(&self) -> u16 {
        self.id
    }

    fn pos(&self) -> &Vector {
        &Vector::zero()
    }

    fn pos_mut(&mut self) -> &mut Vector {
        &mut Vector::zero()
    }

    fn orientation(&self) -> &Vector {
        &Vector::zero()
    }

    fn update_pos(&mut self) {
    }

    fn bounding_box(&self) -> BoundingBox {
        self.bounding_box.clone()
    }

    fn vel(&self) -> &Vector {
        &Vector::zero()
    }

    fn vel_mut(&mut self) -> &mut Vector {
        &mut Vector::zero()
    }

    fn is_static(&self) -> bool {
        return false;
    }
}
