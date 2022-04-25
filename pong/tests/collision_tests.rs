use std::cell::{Ref, RefCell};
use pong::collision::collision::{Collision, CollisionDetector};
use pong::game_object::game_object::GameObject;
use pong::geom::geom::{BoundingBox, Vector};
use pong::geom::shape::ShapeType;
use rstest::rstest;

#[rstest]
#[case(RefCell::new(vec![]), vec![])]
#[case(
    RefCell::new(vec![
        MockGameObject::new(1, BoundingBox::create(&Vector{x: 50., y: 50.}, 20., 20.)),
        MockGameObject::new(2, BoundingBox::create(&Vector{x: 50., y: 50.}, 20., 20.))
    ]),
    vec![Collision(1, 2)]
)]
#[case(
    RefCell::new(vec![
        MockGameObject::new(1, BoundingBox::create(&Vector{x: 60., y: 65.}, 20., 20.)),
        MockGameObject::new(2, BoundingBox::create(&Vector{x: 50., y: 50.}, 20., 20.)),
    ]),
    vec![Collision(1, 2)]
)]
#[case(
    RefCell::new(vec![
        MockGameObject::new(1, BoundingBox::create(&Vector{x: 50., y: 50.}, 20., 20.)),
        MockGameObject::new(2, BoundingBox::create(&Vector{x: 80., y: 80.}, 20., 20.)),
    ]),
    vec![]
)]
#[case(
    RefCell::new(vec![
        MockGameObject::new(1, BoundingBox::create(&Vector{x: 50., y: 50.}, 50., 50.)),
        MockGameObject::new(2, BoundingBox::create(&Vector{x: 500., y: 50.}, 50., 50.)),
    ]),
    vec![]
)]
pub fn should_detect_collisions(
    #[case] objs: RefCell<Vec<Box<dyn GameObject>>>,
    #[case] expected_collisions: Vec<Collision>,
) {
    let detector = CollisionDetector::new();
    let res = detector.detect_collisions(objs.borrow());
    assert_eq!(
        res.get_collisions(),
        expected_collisions.iter().collect::<Vec<&Collision>>()
    );
}

#[derive(Debug)]
pub struct MockGameObject {
    id: u16,
    bounding_box: BoundingBox,
    zero_vec: Vector,
}

impl MockGameObject {
    pub fn new(id: u16, bounding_box: BoundingBox) -> Box<dyn GameObject> {
        Box::new(MockGameObject {
            id,
            bounding_box,
            zero_vec: Vector::zero(),
        })
    }
}

impl GameObject for MockGameObject {
    fn id(&self) -> u16 {
        self.id
    }

    fn obj_type(&self) -> &str {
        todo!()
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

    fn update_pos(&mut self) {
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
}
