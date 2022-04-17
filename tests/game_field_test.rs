
#[cfg(test)]
mod game_field_tests {
    use rust_wasm::{Field, GameObject, Input, InputType, Player};

    #[test]
    fn player_input_update_pos__up() {
        let height = 1000;
        let players = vec![
            Player {obj: GameObject {id: 1, x: 10, y: height / 2, shape: 10 << 8 | 5}}
        ];
        let mut field = Field::mock(
            1000, 1000, players, vec![]
        );
        let inputs = vec![Input {input: InputType::UP, obj_id: 1}];
        field.tick_inner(inputs);
        let player = field.players().first().unwrap();
        assert_eq!(player.obj.y, height / 2 + 5);
    }

    #[test]
    fn player_input_update_pos__down() {
        let height = 1000;
        let players = vec![
            Player {obj: GameObject {id: 1, x: 10, y: height / 2, shape: 10 << 8 | 5}}
        ];
        let mut field = Field::mock(
            1000, 1000, players, vec![]
        );
        let inputs = vec![Input {input: InputType::DOWN, obj_id: 1}];
        field.tick_inner(inputs);
        let player = field.players().first().unwrap();
        assert_eq!(player.obj.y, height / 2 - 5);
    }

    #[test]
    fn player_input_update_out_of_bounds__up() {
        let height = 1000;
        let players = vec![
            Player {obj: GameObject {id: 1, x: 10, y: height - 6, shape: 10 << 8 | 5}}
        ];
        let mut field = Field::mock(
            1000, 1000, players, vec![]
        );
        let inputs = vec![Input {input: InputType::UP, obj_id: 1}];
        field.tick_inner(inputs);
        let player = field.players().first().unwrap();
        assert_eq!(player.obj.y, 998);
    }

    #[test]
    fn player_input_update_out_of_bounds__down() {
        let height = 1000;
        let players = vec![
            Player {obj: GameObject {id: 1, x: 10, y: 0 + 6, shape: 10 << 8 | 5}}
        ];
        let mut field = Field::mock(
            1000, 1000, players, vec![]
        );
        let inputs = vec![Input {input: InputType::DOWN, obj_id: 1}];
        field.tick_inner(inputs);
        let player = field.players().first().unwrap();
        assert_eq!(player.obj.y, 2);
    }
}
