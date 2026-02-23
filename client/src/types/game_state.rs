use cainome::cairo_serde::ContractAddress;
use enum_as_inner::EnumAsInner;
use starknet::core::types::Felt;
use crate::types::Board;
use crate::types::board_size::BoardSize;

#[derive(Debug, EnumAsInner)]
pub enum GameState {
    InLobby(BoardSize),
    InGame(GameData)
}

#[derive(Debug)]
pub struct GameData {
    pub game_id: Felt,
    pub opponent: ContractAddress,
    pub board: Board,
    pub state: InGameState
}

impl GameData {
    pub fn new(game_id: Felt, opponent: ContractAddress, board_size: BoardSize) -> Self {
        Self {
            game_id,
            opponent,
            board: Board::new(board_size),
            state: InGameState::PlacingShips,
        }
    }

    pub fn can_attack(&self, player_address: &ContractAddress) -> bool {
        let Some(turn) = self.state.as_playing() else {
            return false
        };

        turn.attacking_player == *player_address && turn.current_attack.is_none()
    }

    pub fn board_size(&self) -> BoardSize {
        self.board.size()
    }
}

#[derive(Debug, EnumAsInner)]
pub enum InGameState {
    PlacingShips,
    Playing(PlayTurn),
    Ended
}


#[derive(Debug)]
pub struct PlayTurn {
    pub attacking_player: ContractAddress,
    pub current_attack: Option<(u8, u8)>
}