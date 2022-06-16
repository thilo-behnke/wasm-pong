#[cfg(test)]
mod game_field_tests {
    use pong::game_field::{Field, Input, InputType};
    use std::cell::RefCell;

    #[test]
    fn player_input_update_pos_up() {
        let height = 1000;
        let mut field = Field::mock(1000, height);
        field.add_player(1, 50, height / 2);
        let inputs = vec![Input {
            input: InputType::UP,
            obj_id: 1,
            player: 1
        }];
        field.tick(inputs, 1_000.);
        let player = RefCell::borrow(
            field
                .objs()
                .iter()
                .find(|o| RefCell::borrow(o).obj_type() == "player")
                .unwrap(),
        );
        assert_eq!(player.pos().y, height as f64 / 2. + 1.);
    }

    #[test]
    fn player_input_update_pos_down() {
        let height = 1000;
        let mut field = Field::mock(1000, height);
        field.add_player(1, 50, height / 2);
        let inputs = vec![Input {
            input: InputType::DOWN,
            obj_id: 1,
            player: 1
        }];
        field.tick(inputs, 1_000.);
        let objs = field.objs();
        let player = objs
            .iter()
            .find(|o| RefCell::borrow(o).obj_type() == "player")
            .unwrap();
        assert_eq!(RefCell::borrow(player).pos().y, height as f64 / 2. - 1.);
    }
}
