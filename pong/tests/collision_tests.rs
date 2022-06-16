use pong::collision::collision::{Collision, CollisionDetector, CollisionGroup};
use pong::game_object::game_object::GameObject;
use pong::geom::geom::{BoundingBox};
use pong::geom::shape::ShapeType;
use pong::utils::utils::DefaultLoggerFactory;
use rstest::rstest;
use std::cell::{RefCell};
use std::rc::Rc;
use pong::geom::vector::Vector;

#[rstest]
#[case(vec![], vec![])]
#[case(
    vec![
        MockGameObject::new(1, "a", BoundingBox::create(&Vector{x: 50., y: 50.}, 20., 20.)),
        MockGameObject::new(2, "b", BoundingBox::create(&Vector{x: 50., y: 50.}, 20., 20.))
    ],
    vec![Collision(1, 2)]
)]
#[case(
    vec![
        MockGameObject::new(1, "a", BoundingBox::create(&Vector{x: 60., y: 65.}, 20., 20.)),
        MockGameObject::new(2, "b", BoundingBox::create(&Vector{x: 50., y: 50.}, 20., 20.)),
    ],
    vec![Collision(1, 2)]
)]
#[case(
    vec![
        MockGameObject::new(1, "a", BoundingBox::create(&Vector{x: 50., y: 50.}, 20., 20.)),
        MockGameObject::new(2, "b", BoundingBox::create(&Vector{x: 80., y: 80.}, 20., 20.)),
    ],
    vec![]
)]
#[case(
    vec![
        MockGameObject::new(1, "a", BoundingBox::create(&Vector{x: 50., y: 50.}, 50., 50.)),
        MockGameObject::new(2, "b", BoundingBox::create(&Vector{x: 500., y: 50.}, 50., 50.)),
    ],
    vec![]
)]
#[case(
    vec![
        MockGameObject::new(1, "a", BoundingBox::create(&Vector{x: 60., y: 65.}, 20., 20.)),
        MockGameObject::new(2, "c", BoundingBox::create(&Vector{x: 50., y: 50.}, 20., 20.)),
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
    id: u16,
    obj_type: String,
    bounding_box: BoundingBox
}

impl MockGameObject {
    pub fn new(
        id: u16,
        obj_type: &str,
        bounding_box: BoundingBox,
    ) -> Rc<RefCell<Box<dyn GameObject>>> {
        Rc::new(RefCell::new(Box::new(MockGameObject {
            id,
            obj_type: String::from(obj_type),
            bounding_box
        })))
    }
}

impl GameObject for MockGameObject {
    fn id(&self) -> u16 {
        self.id
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
