use crate::types::board_size::BoardSize;
use starknet_rust_core::types::Felt;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lobbies(HashMap<BoardSize, Felt>);

impl Lobbies {
    pub fn new(waitlist: Vec<(BoardSize, impl Into<Felt>)>) -> Self {
        let mut lobbies = HashMap::<BoardSize, Felt>::new();

        waitlist.into_iter().for_each(|(board_size, address)| {
            lobbies.insert(board_size, address.into());
        });

        Lobbies(lobbies)
    }

    pub fn lobby(&self, size: &BoardSize) -> Option<Felt> {
        self.0.get(size).cloned()
    }

    pub fn join(&mut self, size: BoardSize, address: impl Into<Felt>) {
        self.0.insert(size, address.into());
    }

    pub fn exit(&mut self, size: &BoardSize) {
        self.0.remove(size);
    }

    pub fn player_lobby(&self, address: impl Into<Felt>) -> Option<BoardSize> {
        let address: Felt = address.into();
        BoardSize::all()
            .iter()
            .find(|size| {
                if let Some(addr) = self.0.get(size)
                    && *addr == address
                {
                    true
                } else {
                    false
                }
            })
            .cloned()
    }
}
