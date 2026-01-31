use enum_as_inner::EnumAsInner;
use starknet::core::types::Felt;
use starknet::core::utils::get_selector_from_name;
use starkwaves_macros::StarknetEvent;

#[derive(StarknetEvent, Clone, Debug, PartialEq, EnumAsInner)]
pub enum GameEvent {
    PlayersAssembled {
        #[key]
        game_id: Felt,
        player_a: Felt,
        player_b: Felt,
    },
    GameStarted {
        #[key]
        game_id: Felt,
        attacker: Felt,
        defender: Felt,
    },
    Attack {
        #[key]
        game_id: Felt,
        player: Felt,
        x: Felt,
        y: Felt,
    },
    Hit {
        #[key]
        game_id: Felt,
        attacker: Felt,
        defender: Felt,
        x: Felt,
        y: Felt,
        ship_kind: Felt,
    },
    GameRevealRequest {
        #[key]
        game_id: Felt,
        player_a: Felt,
        player_b: Felt,
    },
    GameOver {
        #[key]
        game_id: Felt,
        player_a: Felt,
        player_b: Felt,
        outcome: Felt,
    },
}

impl GameEvent {
    pub fn keys(game_id: Felt) -> Vec<Vec<Felt>> {
        vec![
            vec![
                get_selector_from_name("PlayersAssembled").unwrap(),
                game_id,
            ],
            vec![
                get_selector_from_name("GameStarted").unwrap(),
                game_id,
            ],
            vec![
                get_selector_from_name("Attack").unwrap(),
                game_id,
            ],
            vec![
                get_selector_from_name("Hit").unwrap(),
                game_id,
            ],
            vec![
                get_selector_from_name("GameRevealRequest").unwrap(),
                game_id,
            ],
            vec![
                get_selector_from_name("GameOver").unwrap(),
                game_id,
            ],
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use starknet::core::types::Event;

    #[test]
    fn test_from_starknet_event() {
        let game_id = Felt::from_hex("0x1").unwrap();
        let player_a =
            Felt::from_hex("0x78662e7352d062084b0010068b99288486c2d8b914f6e2a55ce945f8792c8b1")
                .unwrap();
        let player_b =
            Felt::from_hex("0x49dfb8ce986e21d354ac93ea65e6a11f639c1934ea253e5ff14ca62eca0f38e")
                .unwrap();
        let event = Event {
            from_address: Felt::from_hex(
                "0xe4edd67e0999fb497aa263ba116a5bb0d43b514a9007953070f0f8bc872a22",
            )
            .unwrap(),
            keys: vec![
                Felt::from_hex("0xcb9bae45759e10a42c2316bdf1f532acc155bad8107904928045716eeef86d")
                    .unwrap(),
                game_id,
            ],
            data: vec![player_a, player_b],
        };

        let game_event: GameEvent = TryFrom::try_from(event).unwrap();

        assert_eq!(
            game_event,
            GameEvent::PlayersAssembled {
                game_id: game_id,
                player_a: player_a,
                player_b: player_b,
            }
        )
    }
}
