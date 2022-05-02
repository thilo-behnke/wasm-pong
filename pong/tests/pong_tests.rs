use std::cell::RefCell;
use std::rc::Rc;
use rstest::rstest;
use pong::game_field::{Bound, Field};
use pong::game_object::game_object::{DefaultGameObject, GameObject};
use pong::geom::geom::Vector;
use pong::pong::pong_collisions::handle_player_bound_collision;
use pong::utils::utils::{DefaultLoggerFactory, NoopLogger};

#[rstest]
#[case(
    // given
    create_player(1, 10, 0, Vector::new(0., -1.)),
    get_bound(Bound::BOTTOM),
    // expected
    create_player(1, 10, 120, Vector::new(0., -1.)),
    get_bound(Bound::BOTTOM)
)]
#[case(
    // given
    create_player(1, 10, 119, Vector::new(0., -1.)),
    get_bound(Bound::BOTTOM),
    // expected
    create_player(1, 10, 120, Vector::new(0., -1.)),
    get_bound(Bound::BOTTOM)
)]
#[case(
    // given
    create_player(1, 10, 121, Vector::new(0., -1.)),
    get_bound(Bound::BOTTOM),
    // expected
    create_player(1, 10, 120, Vector::new(0., -1.)),
    get_bound(Bound::BOTTOM)
)]
#[case(
    // given
    create_player(1, 10, 600, Vector::new(0., 1.)),
    get_bound(Bound::TOP),
    // expected
    create_player(1, 10, 480, Vector::new(0., 1.)),
    get_bound(Bound::TOP)
)]
#[case(
    // given
    create_player(1, 10, 599, Vector::new(0., 1.)),
    get_bound(Bound::TOP),
    // expected
    create_player(1, 10, 480, Vector::new(0., 1.)),
    get_bound(Bound::TOP)
)]
pub fn should_correctly_handle_player_bounds_collision(
    #[case] mut player: Rc<RefCell<Box<dyn GameObject>>>,
    #[case] mut bounds: Rc<RefCell<Box<dyn GameObject>>>,
    #[case] mut player_expected: Rc<RefCell<Box<dyn GameObject>>>,
    #[case] mut bounds_expected: Rc<RefCell<Box<dyn GameObject>>>
) {
    handle_player_bound_collision(player.clone(), bounds.clone());
    assert_eq!(player_expected.borrow().pos(), player.borrow().pos());
    assert_eq!(bounds_expected.borrow().pos(), bounds.borrow().pos());
}

fn create_player(id: u16, x: u16, y: u16, orientation: Vector) -> Rc<RefCell<Box<dyn GameObject>>> {
    let logger = DefaultLoggerFactory::noop();
    let field = Field::new(logger);
    let mut player = DefaultGameObject::player(id, x, y, &field);
    let player_orientation = player.orientation_mut();
    player_orientation.x = orientation.x;
    player_orientation.y = orientation.y;
    Rc::new(RefCell::new(player))
}

fn get_bound(bound: Bound) -> Rc<RefCell<Box<dyn GameObject>>> {
    let logger = DefaultLoggerFactory::noop();
    let field = Field::new(logger);
    let bounds = DefaultGameObject::bounds(field.width, field.height);
    return Rc::new(RefCell::new(bounds.into_iter().find(|b| {
        b.0 == bound
    }).unwrap().inner()));
}
