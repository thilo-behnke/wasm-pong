#[cfg(test)]
mod game_field_tests {
    use pong::game_field::{Field, Input, InputType};

    #[test]
    fn player_input_update_pos__up() {
        let height = 1000;
        let mut field = Field::mock(1000, height);
        field.add_player(1, 50, height / 2);
        let inputs = vec![Input {
            input: InputType::UP,
            obj_id: 1,
        }];
        field.tick(inputs);
        let players = field.players();
        let player = players.first().unwrap();
        assert_eq!(player.obj.y, height / 2 + 1);
    }

    #[test]
    fn player_input_update_pos__down() {
        let height = 1000;
        let mut field = Field::mock(1000, height);
        field.add_player(1, 50, height / 2);
        let inputs = vec![Input {
            input: InputType::DOWN,
            obj_id: 1,
        }];
        field.tick(inputs);
        let players = field.players();
        let player = players.first().unwrap();
        assert_eq!(player.obj.y, height / 2 - 1);
    }

    #[test]
    fn player_input_update_out_of_bounds__up() {
        let height = 1000;
        let mut field = Field::mock(1000, height);
        field.add_player(1, 50, height - 6);
        let inputs = vec![Input {
            input: InputType::UP,
            obj_id: 1,
        }];
        field.tick(inputs);
        let players = field.players();
        let player = players.first().unwrap();
        assert_eq!(player.obj.y, height - 6);
    }

    #[test]
    fn player_input_update_out_of_bounds__down() {
        let height = 1000;
        let mut field = Field::mock(1000, height);
        field.add_player(1, 50, height / 5 / 2);
        let inputs = vec![Input {
            input: InputType::DOWN,
            obj_id: 1,
        }];
        field.tick(inputs);
        let players = field.players();
        let player = players.first().unwrap();
        assert_eq!(player.obj.y, height / 5);
    }
}
