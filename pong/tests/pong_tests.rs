use rstest::rstest;
use pong::game_field::{Bound, Field};
use pong::game_object::game_object::{DefaultGameObject, GameObject};
use pong::geom::geom::Vector;
use pong::pong::pong_collisions::handle_player_bound_collision;
use pong::utils::utils::NoopLogger;

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
    #[case] mut player: Box<dyn GameObject>,
    #[case] mut bounds: Box<dyn GameObject>,
    #[case] mut player_expected: Box<dyn GameObject>,
    #[case] mut bounds_expected: Box<dyn GameObject>
) {
    handle_player_bound_collision(&mut player, &mut bounds);
    assert_eq!(player_expected.pos(), player.pos());
    assert_eq!(bounds_expected.pos(), bounds.pos());
}

fn create_player(id: u16, x: u16, y: u16, orientation: Vector) -> Box<dyn GameObject> {
    let field = Field::new(Box::new(NoopLogger{}));
    let mut player = DefaultGameObject::player(id, x, y, &field);
    let player_orientation = player.orientation_mut();
    player_orientation.x = orientation.x;
    player_orientation.y = orientation.y;
    player
}

fn get_bound(bound: Bound) -> Box<dyn GameObject> {
    let field = Field::new(Box::new(NoopLogger{}));
    let bounds = DefaultGameObject::bounds(field.width, field.height);
    return bounds.into_iter().find(|b| {
        b.0 == bound
    }).unwrap().inner();
}
