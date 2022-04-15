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
        assert_eq!(player.obj.pos.y, height as f64 / 2. + 1.);
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
        assert_eq!(player.obj.pos.y, height as f64 / 2. - 1.);
    }

    #[test]
    fn player_input_update_out_of_bounds__up() {
        let height = 1000;
        let mut field = Field::mock(1000, height);
        field.add_player(1, 50, height - height / 5 / 2);
        let inputs = vec![Input {
            input: InputType::UP,
            obj_id: 1,
        }];
        field.tick(inputs);
        let players = field.players();
        let player = players.first().unwrap();
        assert_eq!(player.obj.pos.y, height as f64 - height as f64 / 5. / 2.);
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
        assert_eq!(player.obj.pos.y, height as f64 / 5. / 2.);
    }
}
