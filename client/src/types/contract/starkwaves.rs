#[derive()]
pub struct Starkwaves<A: starknet_rust::accounts::ConnectedAccount + Sync> {
    pub address: starknet_rust::core::types::Felt,
    pub account: A,
    pub block_id: starknet_rust::core::types::BlockId,
}
impl<A: starknet_rust::accounts::ConnectedAccount + Sync> Starkwaves<A> {
    pub fn new(address: starknet_rust::core::types::Felt, account: A) -> Self {
        Self {
            address,
            account,
            block_id: starknet_rust::core::types::BlockId::Tag(
                starknet_rust::core::types::BlockTag::PreConfirmed,
            ),
        }
    }
    pub fn set_contract_address(&mut self, address: starknet_rust::core::types::Felt) {
        self.address = address;
    }
    pub fn provider(&self) -> &A::Provider {
        self.account.provider()
    }
    pub fn set_block(&mut self, block_id: starknet_rust::core::types::BlockId) {
        self.block_id = block_id;
    }
    pub fn with_block(self, block_id: starknet_rust::core::types::BlockId) -> Self {
        Self { block_id, ..self }
    }
}
#[derive()]
pub struct StarkwavesReader<P: starknet_rust::providers::Provider + Sync> {
    pub address: starknet_rust::core::types::Felt,
    pub provider: P,
    pub block_id: starknet_rust::core::types::BlockId,
}
impl<P: starknet_rust::providers::Provider + Sync> StarkwavesReader<P> {
    pub fn new(address: starknet_rust::core::types::Felt, provider: P) -> Self {
        Self {
            address,
            provider,
            block_id: starknet_rust::core::types::BlockId::Tag(
                starknet_rust::core::types::BlockTag::PreConfirmed,
            ),
        }
    }
    pub fn set_contract_address(&mut self, address: starknet_rust::core::types::Felt) {
        self.address = address;
    }
    pub fn provider(&self) -> &P {
        &self.provider
    }
    pub fn set_block(&mut self, block_id: starknet_rust::core::types::BlockId) {
        self.block_id = block_id;
    }
    pub fn with_block(self, block_id: starknet_rust::core::types::BlockId) -> Self {
        Self { block_id, ..self }
    }
}
#[derive(Debug, Clone)]
pub struct AttackEvent {
    pub game_id: starknet_rust::core::types::Felt,
    pub player: cainome::cairo_serde::ContractAddress,
    pub x: u8,
    pub y: u8,
}
impl cainome::cairo_serde::CairoSerde for AttackEvent {
    type RustType = Self;
    const SERIALIZED_SIZE: std::option::Option<usize> = None;
    #[inline]
    fn cairo_serialized_size(__rust: &Self::RustType) -> usize {
        let mut __size = 0;
        __size += starknet_rust::core::types::Felt::cairo_serialized_size(&__rust.game_id);
        __size
            += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                &__rust.player,
            );
        __size += u8::cairo_serialized_size(&__rust.x);
        __size += u8::cairo_serialized_size(&__rust.y);
        __size
    }
    fn cairo_serialize(__rust: &Self::RustType) -> Vec<starknet_rust::core::types::Felt> {
        let mut __out: Vec<starknet_rust::core::types::Felt> = vec![];
        __out.extend(starknet_rust::core::types::Felt::cairo_serialize(&__rust.game_id));
        __out
            .extend(
                cainome::cairo_serde::ContractAddress::cairo_serialize(&__rust.player),
            );
        __out.extend(u8::cairo_serialize(&__rust.x));
        __out.extend(u8::cairo_serialize(&__rust.y));
        __out
    }
    fn cairo_deserialize(
        __felts: &[starknet_rust::core::types::Felt],
        __offset: usize,
    ) -> cainome::cairo_serde::Result<Self::RustType> {
        let mut __offset = __offset;
        let game_id = starknet_rust::core::types::Felt::cairo_deserialize(__felts, __offset)?;
        __offset += starknet_rust::core::types::Felt::cairo_serialized_size(&game_id);
        let player = cainome::cairo_serde::ContractAddress::cairo_deserialize(
            __felts,
            __offset,
        )?;
        __offset
            += cainome::cairo_serde::ContractAddress::cairo_serialized_size(&player);
        let x = u8::cairo_deserialize(__felts, __offset)?;
        __offset += u8::cairo_serialized_size(&x);
        let y = u8::cairo_deserialize(__felts, __offset)?;
        __offset += u8::cairo_serialized_size(&y);
        Ok(AttackEvent {
            game_id,
            player,
            x,
            y,
        })
    }
}
impl AttackEvent {
    pub fn event_selector() -> starknet_rust::core::types::Felt {
        starknet_rust::core::utils::get_selector_from_name("AttackEvent").unwrap()
    }
    pub fn event_name() -> &'static str {
        "AttackEvent"
    }
}
#[derive(Debug, Clone)]
pub struct AttackResultEvent {
    pub game_id: starknet_rust::core::types::Felt,
    pub attacker: cainome::cairo_serde::ContractAddress,
    pub defender: cainome::cairo_serde::ContractAddress,
    pub x: u8,
    pub y: u8,
    pub ship_kind: Option<ShipKind>,
}
impl cainome::cairo_serde::CairoSerde for AttackResultEvent {
    type RustType = Self;
    const SERIALIZED_SIZE: std::option::Option<usize> = None;
    #[inline]
    fn cairo_serialized_size(__rust: &Self::RustType) -> usize {
        let mut __size = 0;
        __size += starknet_rust::core::types::Felt::cairo_serialized_size(&__rust.game_id);
        __size
            += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                &__rust.attacker,
            );
        __size
            += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                &__rust.defender,
            );
        __size += u8::cairo_serialized_size(&__rust.x);
        __size += u8::cairo_serialized_size(&__rust.y);
        __size += Option::<ShipKind>::cairo_serialized_size(&__rust.ship_kind);
        __size
    }
    fn cairo_serialize(__rust: &Self::RustType) -> Vec<starknet_rust::core::types::Felt> {
        let mut __out: Vec<starknet_rust::core::types::Felt> = vec![];
        __out.extend(starknet_rust::core::types::Felt::cairo_serialize(&__rust.game_id));
        __out
            .extend(
                cainome::cairo_serde::ContractAddress::cairo_serialize(&__rust.attacker),
            );
        __out
            .extend(
                cainome::cairo_serde::ContractAddress::cairo_serialize(&__rust.defender),
            );
        __out.extend(u8::cairo_serialize(&__rust.x));
        __out.extend(u8::cairo_serialize(&__rust.y));
        __out.extend(Option::<ShipKind>::cairo_serialize(&__rust.ship_kind));
        __out
    }
    fn cairo_deserialize(
        __felts: &[starknet_rust::core::types::Felt],
        __offset: usize,
    ) -> cainome::cairo_serde::Result<Self::RustType> {
        let mut __offset = __offset;
        let game_id = starknet_rust::core::types::Felt::cairo_deserialize(__felts, __offset)?;
        __offset += starknet_rust::core::types::Felt::cairo_serialized_size(&game_id);
        let attacker = cainome::cairo_serde::ContractAddress::cairo_deserialize(
            __felts,
            __offset,
        )?;
        __offset
            += cainome::cairo_serde::ContractAddress::cairo_serialized_size(&attacker);
        let defender = cainome::cairo_serde::ContractAddress::cairo_deserialize(
            __felts,
            __offset,
        )?;
        __offset
            += cainome::cairo_serde::ContractAddress::cairo_serialized_size(&defender);
        let x = u8::cairo_deserialize(__felts, __offset)?;
        __offset += u8::cairo_serialized_size(&x);
        let y = u8::cairo_deserialize(__felts, __offset)?;
        __offset += u8::cairo_serialized_size(&y);
        let ship_kind = Option::<ShipKind>::cairo_deserialize(__felts, __offset)?;
        __offset += Option::<ShipKind>::cairo_serialized_size(&ship_kind);
        Ok(AttackResultEvent {
            game_id,
            attacker,
            defender,
            x,
            y,
            ship_kind,
        })
    }
}
impl AttackResultEvent {
    pub fn event_selector() -> starknet_rust::core::types::Felt {
        starknet_rust::core::utils::get_selector_from_name("AttackResultEvent").unwrap()
    }
    pub fn event_name() -> &'static str {
        "AttackResultEvent"
    }
}
#[derive(Debug, Clone)]
pub struct GameOverEvent {
    pub game_id: starknet_rust::core::types::Felt,
    pub player_a: cainome::cairo_serde::ContractAddress,
    pub player_b: cainome::cairo_serde::ContractAddress,
    pub outcome: Outcome,
}
impl cainome::cairo_serde::CairoSerde for GameOverEvent {
    type RustType = Self;
    const SERIALIZED_SIZE: std::option::Option<usize> = None;
    #[inline]
    fn cairo_serialized_size(__rust: &Self::RustType) -> usize {
        let mut __size = 0;
        __size += starknet_rust::core::types::Felt::cairo_serialized_size(&__rust.game_id);
        __size
            += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                &__rust.player_a,
            );
        __size
            += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                &__rust.player_b,
            );
        __size += Outcome::cairo_serialized_size(&__rust.outcome);
        __size
    }
    fn cairo_serialize(__rust: &Self::RustType) -> Vec<starknet_rust::core::types::Felt> {
        let mut __out: Vec<starknet_rust::core::types::Felt> = vec![];
        __out.extend(starknet_rust::core::types::Felt::cairo_serialize(&__rust.game_id));
        __out
            .extend(
                cainome::cairo_serde::ContractAddress::cairo_serialize(&__rust.player_a),
            );
        __out
            .extend(
                cainome::cairo_serde::ContractAddress::cairo_serialize(&__rust.player_b),
            );
        __out.extend(Outcome::cairo_serialize(&__rust.outcome));
        __out
    }
    fn cairo_deserialize(
        __felts: &[starknet_rust::core::types::Felt],
        __offset: usize,
    ) -> cainome::cairo_serde::Result<Self::RustType> {
        let mut __offset = __offset;
        let game_id = starknet_rust::core::types::Felt::cairo_deserialize(__felts, __offset)?;
        __offset += starknet_rust::core::types::Felt::cairo_serialized_size(&game_id);
        let player_a = cainome::cairo_serde::ContractAddress::cairo_deserialize(
            __felts,
            __offset,
        )?;
        __offset
            += cainome::cairo_serde::ContractAddress::cairo_serialized_size(&player_a);
        let player_b = cainome::cairo_serde::ContractAddress::cairo_deserialize(
            __felts,
            __offset,
        )?;
        __offset
            += cainome::cairo_serde::ContractAddress::cairo_serialized_size(&player_b);
        let outcome = Outcome::cairo_deserialize(__felts, __offset)?;
        __offset += Outcome::cairo_serialized_size(&outcome);
        Ok(GameOverEvent {
            game_id,
            player_a,
            player_b,
            outcome,
        })
    }
}
impl GameOverEvent {
    pub fn event_selector() -> starknet_rust::core::types::Felt {
        starknet_rust::core::utils::get_selector_from_name("GameOverEvent").unwrap()
    }
    pub fn event_name() -> &'static str {
        "GameOverEvent"
    }
}
#[derive(Debug, Clone)]
pub struct GameRevealRequestEvent {
    pub game_id: starknet_rust::core::types::Felt,
    pub player_a: cainome::cairo_serde::ContractAddress,
    pub player_b: cainome::cairo_serde::ContractAddress,
}
impl cainome::cairo_serde::CairoSerde for GameRevealRequestEvent {
    type RustType = Self;
    const SERIALIZED_SIZE: std::option::Option<usize> = None;
    #[inline]
    fn cairo_serialized_size(__rust: &Self::RustType) -> usize {
        let mut __size = 0;
        __size += starknet_rust::core::types::Felt::cairo_serialized_size(&__rust.game_id);
        __size
            += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                &__rust.player_a,
            );
        __size
            += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                &__rust.player_b,
            );
        __size
    }
    fn cairo_serialize(__rust: &Self::RustType) -> Vec<starknet_rust::core::types::Felt> {
        let mut __out: Vec<starknet_rust::core::types::Felt> = vec![];
        __out.extend(starknet_rust::core::types::Felt::cairo_serialize(&__rust.game_id));
        __out
            .extend(
                cainome::cairo_serde::ContractAddress::cairo_serialize(&__rust.player_a),
            );
        __out
            .extend(
                cainome::cairo_serde::ContractAddress::cairo_serialize(&__rust.player_b),
            );
        __out
    }
    fn cairo_deserialize(
        __felts: &[starknet_rust::core::types::Felt],
        __offset: usize,
    ) -> cainome::cairo_serde::Result<Self::RustType> {
        let mut __offset = __offset;
        let game_id = starknet_rust::core::types::Felt::cairo_deserialize(__felts, __offset)?;
        __offset += starknet_rust::core::types::Felt::cairo_serialized_size(&game_id);
        let player_a = cainome::cairo_serde::ContractAddress::cairo_deserialize(
            __felts,
            __offset,
        )?;
        __offset
            += cainome::cairo_serde::ContractAddress::cairo_serialized_size(&player_a);
        let player_b = cainome::cairo_serde::ContractAddress::cairo_deserialize(
            __felts,
            __offset,
        )?;
        __offset
            += cainome::cairo_serde::ContractAddress::cairo_serialized_size(&player_b);
        Ok(GameRevealRequestEvent {
            game_id,
            player_a,
            player_b,
        })
    }
}
impl GameRevealRequestEvent {
    pub fn event_selector() -> starknet_rust::core::types::Felt {
        starknet_rust::core::utils::get_selector_from_name("GameRevealRequestEvent").unwrap()
    }
    pub fn event_name() -> &'static str {
        "GameRevealRequestEvent"
    }
}
#[derive(Debug, Clone)]
pub struct GameStartedEvent {
    pub game_id: starknet_rust::core::types::Felt,
    pub attacker: cainome::cairo_serde::ContractAddress,
    pub defender: cainome::cairo_serde::ContractAddress,
}
impl cainome::cairo_serde::CairoSerde for GameStartedEvent {
    type RustType = Self;
    const SERIALIZED_SIZE: std::option::Option<usize> = None;
    #[inline]
    fn cairo_serialized_size(__rust: &Self::RustType) -> usize {
        let mut __size = 0;
        __size += starknet_rust::core::types::Felt::cairo_serialized_size(&__rust.game_id);
        __size
            += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                &__rust.attacker,
            );
        __size
            += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                &__rust.defender,
            );
        __size
    }
    fn cairo_serialize(__rust: &Self::RustType) -> Vec<starknet_rust::core::types::Felt> {
        let mut __out: Vec<starknet_rust::core::types::Felt> = vec![];
        __out.extend(starknet_rust::core::types::Felt::cairo_serialize(&__rust.game_id));
        __out
            .extend(
                cainome::cairo_serde::ContractAddress::cairo_serialize(&__rust.attacker),
            );
        __out
            .extend(
                cainome::cairo_serde::ContractAddress::cairo_serialize(&__rust.defender),
            );
        __out
    }
    fn cairo_deserialize(
        __felts: &[starknet_rust::core::types::Felt],
        __offset: usize,
    ) -> cainome::cairo_serde::Result<Self::RustType> {
        let mut __offset = __offset;
        let game_id = starknet_rust::core::types::Felt::cairo_deserialize(__felts, __offset)?;
        __offset += starknet_rust::core::types::Felt::cairo_serialized_size(&game_id);
        let attacker = cainome::cairo_serde::ContractAddress::cairo_deserialize(
            __felts,
            __offset,
        )?;
        __offset
            += cainome::cairo_serde::ContractAddress::cairo_serialized_size(&attacker);
        let defender = cainome::cairo_serde::ContractAddress::cairo_deserialize(
            __felts,
            __offset,
        )?;
        __offset
            += cainome::cairo_serde::ContractAddress::cairo_serialized_size(&defender);
        Ok(GameStartedEvent {
            game_id,
            attacker,
            defender,
        })
    }
}
impl GameStartedEvent {
    pub fn event_selector() -> starknet_rust::core::types::Felt {
        starknet_rust::core::utils::get_selector_from_name("GameStartedEvent").unwrap()
    }
    pub fn event_name() -> &'static str {
        "GameStartedEvent"
    }
}
#[derive(Debug, Clone)]
pub struct OwnershipTransferStarted {
    pub previous_owner: cainome::cairo_serde::ContractAddress,
    pub new_owner: cainome::cairo_serde::ContractAddress,
}
impl cainome::cairo_serde::CairoSerde for OwnershipTransferStarted {
    type RustType = Self;
    const SERIALIZED_SIZE: std::option::Option<usize> = None;
    #[inline]
    fn cairo_serialized_size(__rust: &Self::RustType) -> usize {
        let mut __size = 0;
        __size
            += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                &__rust.previous_owner,
            );
        __size
            += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                &__rust.new_owner,
            );
        __size
    }
    fn cairo_serialize(__rust: &Self::RustType) -> Vec<starknet_rust::core::types::Felt> {
        let mut __out: Vec<starknet_rust::core::types::Felt> = vec![];
        __out
            .extend(
                cainome::cairo_serde::ContractAddress::cairo_serialize(
                    &__rust.previous_owner,
                ),
            );
        __out
            .extend(
                cainome::cairo_serde::ContractAddress::cairo_serialize(&__rust.new_owner),
            );
        __out
    }
    fn cairo_deserialize(
        __felts: &[starknet_rust::core::types::Felt],
        __offset: usize,
    ) -> cainome::cairo_serde::Result<Self::RustType> {
        let mut __offset = __offset;
        let previous_owner = cainome::cairo_serde::ContractAddress::cairo_deserialize(
            __felts,
            __offset,
        )?;
        __offset
            += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                &previous_owner,
            );
        let new_owner = cainome::cairo_serde::ContractAddress::cairo_deserialize(
            __felts,
            __offset,
        )?;
        __offset
            += cainome::cairo_serde::ContractAddress::cairo_serialized_size(&new_owner);
        Ok(OwnershipTransferStarted {
            previous_owner,
            new_owner,
        })
    }
}
impl OwnershipTransferStarted {
    pub fn event_selector() -> starknet_rust::core::types::Felt {
        starknet_rust::core::utils::get_selector_from_name("OwnershipTransferStarted")
            .unwrap()
    }
    pub fn event_name() -> &'static str {
        "OwnershipTransferStarted"
    }
}
#[derive(Debug, Clone)]
pub struct OwnershipTransferred {
    pub previous_owner: cainome::cairo_serde::ContractAddress,
    pub new_owner: cainome::cairo_serde::ContractAddress,
}
impl cainome::cairo_serde::CairoSerde for OwnershipTransferred {
    type RustType = Self;
    const SERIALIZED_SIZE: std::option::Option<usize> = None;
    #[inline]
    fn cairo_serialized_size(__rust: &Self::RustType) -> usize {
        let mut __size = 0;
        __size
            += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                &__rust.previous_owner,
            );
        __size
            += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                &__rust.new_owner,
            );
        __size
    }
    fn cairo_serialize(__rust: &Self::RustType) -> Vec<starknet_rust::core::types::Felt> {
        let mut __out: Vec<starknet_rust::core::types::Felt> = vec![];
        __out
            .extend(
                cainome::cairo_serde::ContractAddress::cairo_serialize(
                    &__rust.previous_owner,
                ),
            );
        __out
            .extend(
                cainome::cairo_serde::ContractAddress::cairo_serialize(&__rust.new_owner),
            );
        __out
    }
    fn cairo_deserialize(
        __felts: &[starknet_rust::core::types::Felt],
        __offset: usize,
    ) -> cainome::cairo_serde::Result<Self::RustType> {
        let mut __offset = __offset;
        let previous_owner = cainome::cairo_serde::ContractAddress::cairo_deserialize(
            __felts,
            __offset,
        )?;
        __offset
            += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                &previous_owner,
            );
        let new_owner = cainome::cairo_serde::ContractAddress::cairo_deserialize(
            __felts,
            __offset,
        )?;
        __offset
            += cainome::cairo_serde::ContractAddress::cairo_serialized_size(&new_owner);
        Ok(OwnershipTransferred {
            previous_owner,
            new_owner,
        })
    }
}
impl OwnershipTransferred {
    pub fn event_selector() -> starknet_rust::core::types::Felt {
        starknet_rust::core::utils::get_selector_from_name("OwnershipTransferred").unwrap()
    }
    pub fn event_name() -> &'static str {
        "OwnershipTransferred"
    }
}
#[derive(Debug, Clone)]
pub struct PlayerEnteredLobbyEvent {
    pub lobby: BoardSize,
    pub player: cainome::cairo_serde::ContractAddress,
}
impl cainome::cairo_serde::CairoSerde for PlayerEnteredLobbyEvent {
    type RustType = Self;
    const SERIALIZED_SIZE: std::option::Option<usize> = None;
    #[inline]
    fn cairo_serialized_size(__rust: &Self::RustType) -> usize {
        let mut __size = 0;
        __size += BoardSize::cairo_serialized_size(&__rust.lobby);
        __size
            += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                &__rust.player,
            );
        __size
    }
    fn cairo_serialize(__rust: &Self::RustType) -> Vec<starknet_rust::core::types::Felt> {
        let mut __out: Vec<starknet_rust::core::types::Felt> = vec![];
        __out.extend(BoardSize::cairo_serialize(&__rust.lobby));
        __out
            .extend(
                cainome::cairo_serde::ContractAddress::cairo_serialize(&__rust.player),
            );
        __out
    }
    fn cairo_deserialize(
        __felts: &[starknet_rust::core::types::Felt],
        __offset: usize,
    ) -> cainome::cairo_serde::Result<Self::RustType> {
        let mut __offset = __offset;
        let lobby = BoardSize::cairo_deserialize(__felts, __offset)?;
        __offset += BoardSize::cairo_serialized_size(&lobby);
        let player = cainome::cairo_serde::ContractAddress::cairo_deserialize(
            __felts,
            __offset,
        )?;
        __offset
            += cainome::cairo_serde::ContractAddress::cairo_serialized_size(&player);
        Ok(PlayerEnteredLobbyEvent {
            lobby,
            player,
        })
    }
}
impl PlayerEnteredLobbyEvent {
    pub fn event_selector() -> starknet_rust::core::types::Felt {
        starknet_rust::core::utils::get_selector_from_name("PlayerEnteredLobbyEvent").unwrap()
    }
    pub fn event_name() -> &'static str {
        "PlayerEnteredLobbyEvent"
    }
}
#[derive(Debug, Clone)]
pub struct PlayersAssembledEvent {
    pub game_id: starknet_rust::core::types::Felt,
    pub player_a: cainome::cairo_serde::ContractAddress,
    pub player_b: cainome::cairo_serde::ContractAddress,
    pub board_size: BoardSize,
}
impl cainome::cairo_serde::CairoSerde for PlayersAssembledEvent {
    type RustType = Self;
    const SERIALIZED_SIZE: std::option::Option<usize> = None;
    #[inline]
    fn cairo_serialized_size(__rust: &Self::RustType) -> usize {
        let mut __size = 0;
        __size += starknet_rust::core::types::Felt::cairo_serialized_size(&__rust.game_id);
        __size
            += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                &__rust.player_a,
            );
        __size
            += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                &__rust.player_b,
            );
        __size += BoardSize::cairo_serialized_size(&__rust.board_size);
        __size
    }
    fn cairo_serialize(__rust: &Self::RustType) -> Vec<starknet_rust::core::types::Felt> {
        let mut __out: Vec<starknet_rust::core::types::Felt> = vec![];
        __out.extend(starknet_rust::core::types::Felt::cairo_serialize(&__rust.game_id));
        __out
            .extend(
                cainome::cairo_serde::ContractAddress::cairo_serialize(&__rust.player_a),
            );
        __out
            .extend(
                cainome::cairo_serde::ContractAddress::cairo_serialize(&__rust.player_b),
            );
        __out.extend(BoardSize::cairo_serialize(&__rust.board_size));
        __out
    }
    fn cairo_deserialize(
        __felts: &[starknet_rust::core::types::Felt],
        __offset: usize,
    ) -> cainome::cairo_serde::Result<Self::RustType> {
        let mut __offset = __offset;
        let game_id = starknet_rust::core::types::Felt::cairo_deserialize(__felts, __offset)?;
        __offset += starknet_rust::core::types::Felt::cairo_serialized_size(&game_id);
        let player_a = cainome::cairo_serde::ContractAddress::cairo_deserialize(
            __felts,
            __offset,
        )?;
        __offset
            += cainome::cairo_serde::ContractAddress::cairo_serialized_size(&player_a);
        let player_b = cainome::cairo_serde::ContractAddress::cairo_deserialize(
            __felts,
            __offset,
        )?;
        __offset
            += cainome::cairo_serde::ContractAddress::cairo_serialized_size(&player_b);
        let board_size = BoardSize::cairo_deserialize(__felts, __offset)?;
        __offset += BoardSize::cairo_serialized_size(&board_size);
        Ok(PlayersAssembledEvent {
            game_id,
            player_a,
            player_b,
            board_size,
        })
    }
}
impl PlayersAssembledEvent {
    pub fn event_selector() -> starknet_rust::core::types::Felt {
        starknet_rust::core::utils::get_selector_from_name("PlayersAssembledEvent").unwrap()
    }
    pub fn event_name() -> &'static str {
        "PlayersAssembledEvent"
    }
}
#[derive(Debug, Clone)]
pub enum BoardSize {
    Standard,
    Smaller(SmallerBoardSize),
    Larger(LargerBoardSize),
}
impl cainome::cairo_serde::CairoSerde for BoardSize {
    type RustType = Self;
    const SERIALIZED_SIZE: std::option::Option<usize> = std::option::Option::None;
    #[inline]
    fn cairo_serialized_size(__rust: &Self::RustType) -> usize {
        match __rust {
            BoardSize::Standard => 1,
            BoardSize::Smaller(val) => SmallerBoardSize::cairo_serialized_size(val) + 1,
            BoardSize::Larger(val) => LargerBoardSize::cairo_serialized_size(val) + 1,
            _ => 0,
        }
    }
    fn cairo_serialize(__rust: &Self::RustType) -> Vec<starknet_rust::core::types::Felt> {
        match __rust {
            BoardSize::Standard => usize::cairo_serialize(&0usize),
            BoardSize::Smaller(val) => {
                let mut temp = vec![];
                temp.extend(usize::cairo_serialize(&1usize));
                temp.extend(SmallerBoardSize::cairo_serialize(val));
                temp
            }
            BoardSize::Larger(val) => {
                let mut temp = vec![];
                temp.extend(usize::cairo_serialize(&2usize));
                temp.extend(LargerBoardSize::cairo_serialize(val));
                temp
            }
            _ => vec![],
        }
    }
    fn cairo_deserialize(
        __felts: &[starknet_rust::core::types::Felt],
        __offset: usize,
    ) -> cainome::cairo_serde::Result<Self::RustType> {
        let __f = __felts[__offset];
        let __index = u128::from_be_bytes(__f.to_bytes_be()[16..].try_into().unwrap());
        match __index as usize {
            0usize => Ok(BoardSize::Standard),
            1usize => {
                Ok(
                    BoardSize::Smaller(
                        SmallerBoardSize::cairo_deserialize(__felts, __offset + 1)?,
                    ),
                )
            }
            2usize => {
                Ok(
                    BoardSize::Larger(
                        LargerBoardSize::cairo_deserialize(__felts, __offset + 1)?,
                    ),
                )
            }
            _ => {
                return Err(
                    cainome::cairo_serde::Error::Deserialize(
                        format!("Index not handle for enum {}", "BoardSize"),
                    ),
                );
            }
        }
    }
}
#[derive(Debug, Clone)]
pub enum Event {
    PlayerEntererLobby(PlayerEnteredLobbyEvent),
    PlayersAssembled(PlayersAssembledEvent),
    GameStarted(GameStartedEvent),
    Attack(AttackEvent),
    AttackResult(AttackResultEvent),
    GameRevealRequest(GameRevealRequestEvent),
    GameOver(GameOverEvent),
    OwnableEvent(OwnableComponentEvent),
}
impl cainome::cairo_serde::CairoSerde for Event {
    type RustType = Self;
    const SERIALIZED_SIZE: std::option::Option<usize> = std::option::Option::None;
    #[inline]
    fn cairo_serialized_size(__rust: &Self::RustType) -> usize {
        match __rust {
            Event::PlayerEntererLobby(val) => {
                PlayerEnteredLobbyEvent::cairo_serialized_size(val) + 1
            }
            Event::PlayersAssembled(val) => {
                PlayersAssembledEvent::cairo_serialized_size(val) + 1
            }
            Event::GameStarted(val) => GameStartedEvent::cairo_serialized_size(val) + 1,
            Event::Attack(val) => AttackEvent::cairo_serialized_size(val) + 1,
            Event::AttackResult(val) => AttackResultEvent::cairo_serialized_size(val) + 1,
            Event::GameRevealRequest(val) => {
                GameRevealRequestEvent::cairo_serialized_size(val) + 1
            }
            Event::GameOver(val) => GameOverEvent::cairo_serialized_size(val) + 1,
            Event::OwnableEvent(val) => {
                OwnableComponentEvent::cairo_serialized_size(val) + 1
            }
            _ => 0,
        }
    }
    fn cairo_serialize(__rust: &Self::RustType) -> Vec<starknet_rust::core::types::Felt> {
        match __rust {
            Event::PlayerEntererLobby(val) => {
                let mut temp = vec![];
                temp.extend(usize::cairo_serialize(&0usize));
                temp.extend(PlayerEnteredLobbyEvent::cairo_serialize(val));
                temp
            }
            Event::PlayersAssembled(val) => {
                let mut temp = vec![];
                temp.extend(usize::cairo_serialize(&1usize));
                temp.extend(PlayersAssembledEvent::cairo_serialize(val));
                temp
            }
            Event::GameStarted(val) => {
                let mut temp = vec![];
                temp.extend(usize::cairo_serialize(&2usize));
                temp.extend(GameStartedEvent::cairo_serialize(val));
                temp
            }
            Event::Attack(val) => {
                let mut temp = vec![];
                temp.extend(usize::cairo_serialize(&3usize));
                temp.extend(AttackEvent::cairo_serialize(val));
                temp
            }
            Event::AttackResult(val) => {
                let mut temp = vec![];
                temp.extend(usize::cairo_serialize(&4usize));
                temp.extend(AttackResultEvent::cairo_serialize(val));
                temp
            }
            Event::GameRevealRequest(val) => {
                let mut temp = vec![];
                temp.extend(usize::cairo_serialize(&5usize));
                temp.extend(GameRevealRequestEvent::cairo_serialize(val));
                temp
            }
            Event::GameOver(val) => {
                let mut temp = vec![];
                temp.extend(usize::cairo_serialize(&6usize));
                temp.extend(GameOverEvent::cairo_serialize(val));
                temp
            }
            Event::OwnableEvent(val) => {
                let mut temp = vec![];
                temp.extend(usize::cairo_serialize(&7usize));
                temp.extend(OwnableComponentEvent::cairo_serialize(val));
                temp
            }
            _ => vec![],
        }
    }
    fn cairo_deserialize(
        __felts: &[starknet_rust::core::types::Felt],
        __offset: usize,
    ) -> cainome::cairo_serde::Result<Self::RustType> {
        let __f = __felts[__offset];
        let __index = u128::from_be_bytes(__f.to_bytes_be()[16..].try_into().unwrap());
        match __index as usize {
            0usize => {
                Ok(
                    Event::PlayerEntererLobby(
                        PlayerEnteredLobbyEvent::cairo_deserialize(
                            __felts,
                            __offset + 1,
                        )?,
                    ),
                )
            }
            1usize => {
                Ok(
                    Event::PlayersAssembled(
                        PlayersAssembledEvent::cairo_deserialize(__felts, __offset + 1)?,
                    ),
                )
            }
            2usize => {
                Ok(
                    Event::GameStarted(
                        GameStartedEvent::cairo_deserialize(__felts, __offset + 1)?,
                    ),
                )
            }
            3usize => {
                Ok(Event::Attack(AttackEvent::cairo_deserialize(__felts, __offset + 1)?))
            }
            4usize => {
                Ok(
                    Event::AttackResult(
                        AttackResultEvent::cairo_deserialize(__felts, __offset + 1)?,
                    ),
                )
            }
            5usize => {
                Ok(
                    Event::GameRevealRequest(
                        GameRevealRequestEvent::cairo_deserialize(__felts, __offset + 1)?,
                    ),
                )
            }
            6usize => {
                Ok(
                    Event::GameOver(
                        GameOverEvent::cairo_deserialize(__felts, __offset + 1)?,
                    ),
                )
            }
            7usize => {
                Ok(
                    Event::OwnableEvent(
                        OwnableComponentEvent::cairo_deserialize(__felts, __offset + 1)?,
                    ),
                )
            }
            _ => {
                return Err(
                    cainome::cairo_serde::Error::Deserialize(
                        format!("Index not handle for enum {}", "Event"),
                    ),
                );
            }
        }
    }
}
impl TryFrom<&starknet_rust::core::types::EmittedEvent> for Event {
    type Error = String;
    fn try_from(
        event: &starknet_rust::core::types::EmittedEvent,
    ) -> Result<Self, Self::Error> {
        use cainome::cairo_serde::CairoSerde;
        if event.keys.is_empty() {
            return Err("Event has no key".to_string());
        }
        let selector = event.keys[0];
        if selector
            == starknet_rust::core::utils::get_selector_from_name("PlayerEntererLobby")
                .unwrap_or_else(|_| {
                    panic!("Invalid selector for {}", "PlayerEntererLobby")
                })
        {
            let mut key_offset = 0 + 1;
            let mut data_offset = 0;
            let lobby = match BoardSize::cairo_deserialize(&event.keys, key_offset) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "lobby",
                            "PlayerEntererLobby", e
                        ),
                    );
                }
            };
            key_offset += BoardSize::cairo_serialized_size(&lobby);
            let player = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.data,
                data_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "player",
                            "PlayerEntererLobby", e
                        ),
                    );
                }
            };
            data_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(&player);
            return Ok(
                Event::PlayerEntererLobby(PlayerEnteredLobbyEvent {
                    lobby,
                    player,
                }),
            );
        }
        let selector = event.keys[0];
        if selector
            == starknet_rust::core::utils::get_selector_from_name("PlayersAssembled")
                .unwrap_or_else(|_| {
                    panic!("Invalid selector for {}", "PlayersAssembled")
                })
        {
            let mut key_offset = 0 + 1;
            let mut data_offset = 0;
            let game_id = match starknet_rust::core::types::Felt::cairo_deserialize(
                &event.keys,
                key_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "game_id",
                            "PlayersAssembled", e
                        ),
                    );
                }
            };
            key_offset += starknet_rust::core::types::Felt::cairo_serialized_size(&game_id);
            let player_a = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.data,
                data_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "player_a",
                            "PlayersAssembled", e
                        ),
                    );
                }
            };
            data_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                    &player_a,
                );
            let player_b = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.data,
                data_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "player_b",
                            "PlayersAssembled", e
                        ),
                    );
                }
            };
            data_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                    &player_b,
                );
            let board_size = match BoardSize::cairo_deserialize(
                &event.data,
                data_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "board_size",
                            "PlayersAssembled", e
                        ),
                    );
                }
            };
            data_offset += BoardSize::cairo_serialized_size(&board_size);
            return Ok(
                Event::PlayersAssembled(PlayersAssembledEvent {
                    game_id,
                    player_a,
                    player_b,
                    board_size,
                }),
            );
        }
        let selector = event.keys[0];
        if selector
            == starknet_rust::core::utils::get_selector_from_name("GameStarted")
                .unwrap_or_else(|_| panic!("Invalid selector for {}", "GameStarted"))
        {
            let mut key_offset = 0 + 1;
            let mut data_offset = 0;
            let game_id = match starknet_rust::core::types::Felt::cairo_deserialize(
                &event.keys,
                key_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "game_id",
                            "GameStarted", e
                        ),
                    );
                }
            };
            key_offset += starknet_rust::core::types::Felt::cairo_serialized_size(&game_id);
            let attacker = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.data,
                data_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "attacker",
                            "GameStarted", e
                        ),
                    );
                }
            };
            data_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                    &attacker,
                );
            let defender = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.data,
                data_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "defender",
                            "GameStarted", e
                        ),
                    );
                }
            };
            data_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                    &defender,
                );
            return Ok(
                Event::GameStarted(GameStartedEvent {
                    game_id,
                    attacker,
                    defender,
                }),
            );
        }
        let selector = event.keys[0];
        if selector
            == starknet_rust::core::utils::get_selector_from_name("Attack")
                .unwrap_or_else(|_| panic!("Invalid selector for {}", "Attack"))
        {
            let mut key_offset = 0 + 1;
            let mut data_offset = 0;
            let game_id = match starknet_rust::core::types::Felt::cairo_deserialize(
                &event.keys,
                key_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "game_id",
                            "Attack", e
                        ),
                    );
                }
            };
            key_offset += starknet_rust::core::types::Felt::cairo_serialized_size(&game_id);
            let player = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.data,
                data_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "player",
                            "Attack", e
                        ),
                    );
                }
            };
            data_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(&player);
            let x = match u8::cairo_deserialize(&event.data, data_offset) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "x", "Attack",
                            e
                        ),
                    );
                }
            };
            data_offset += u8::cairo_serialized_size(&x);
            let y = match u8::cairo_deserialize(&event.data, data_offset) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "y", "Attack",
                            e
                        ),
                    );
                }
            };
            data_offset += u8::cairo_serialized_size(&y);
            return Ok(
                Event::Attack(AttackEvent {
                    game_id,
                    player,
                    x,
                    y,
                }),
            );
        }
        let selector = event.keys[0];
        if selector
            == starknet_rust::core::utils::get_selector_from_name("AttackResult")
                .unwrap_or_else(|_| panic!("Invalid selector for {}", "AttackResult"))
        {
            let mut key_offset = 0 + 1;
            let mut data_offset = 0;
            let game_id = match starknet_rust::core::types::Felt::cairo_deserialize(
                &event.keys,
                key_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "game_id",
                            "AttackResult", e
                        ),
                    );
                }
            };
            key_offset += starknet_rust::core::types::Felt::cairo_serialized_size(&game_id);
            let attacker = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.data,
                data_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "attacker",
                            "AttackResult", e
                        ),
                    );
                }
            };
            data_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                    &attacker,
                );
            let defender = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.data,
                data_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "defender",
                            "AttackResult", e
                        ),
                    );
                }
            };
            data_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                    &defender,
                );
            let x = match u8::cairo_deserialize(&event.data, data_offset) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "x",
                            "AttackResult", e
                        ),
                    );
                }
            };
            data_offset += u8::cairo_serialized_size(&x);
            let y = match u8::cairo_deserialize(&event.data, data_offset) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "y",
                            "AttackResult", e
                        ),
                    );
                }
            };
            data_offset += u8::cairo_serialized_size(&y);
            let ship_kind = match Option::<
                ShipKind,
            >::cairo_deserialize(&event.data, data_offset) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "ship_kind",
                            "AttackResult", e
                        ),
                    );
                }
            };
            data_offset += Option::<ShipKind>::cairo_serialized_size(&ship_kind);
            return Ok(
                Event::AttackResult(AttackResultEvent {
                    game_id,
                    attacker,
                    defender,
                    x,
                    y,
                    ship_kind,
                }),
            );
        }
        let selector = event.keys[0];
        if selector
            == starknet_rust::core::utils::get_selector_from_name("GameRevealRequest")
                .unwrap_or_else(|_| {
                    panic!("Invalid selector for {}", "GameRevealRequest")
                })
        {
            let mut key_offset = 0 + 1;
            let mut data_offset = 0;
            let game_id = match starknet_rust::core::types::Felt::cairo_deserialize(
                &event.keys,
                key_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "game_id",
                            "GameRevealRequest", e
                        ),
                    );
                }
            };
            key_offset += starknet_rust::core::types::Felt::cairo_serialized_size(&game_id);
            let player_a = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.data,
                data_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "player_a",
                            "GameRevealRequest", e
                        ),
                    );
                }
            };
            data_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                    &player_a,
                );
            let player_b = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.data,
                data_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "player_b",
                            "GameRevealRequest", e
                        ),
                    );
                }
            };
            data_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                    &player_b,
                );
            return Ok(
                Event::GameRevealRequest(GameRevealRequestEvent {
                    game_id,
                    player_a,
                    player_b,
                }),
            );
        }
        let selector = event.keys[0];
        if selector
            == starknet_rust::core::utils::get_selector_from_name("GameOver")
                .unwrap_or_else(|_| panic!("Invalid selector for {}", "GameOver"))
        {
            let mut key_offset = 0 + 1;
            let mut data_offset = 0;
            let game_id = match starknet_rust::core::types::Felt::cairo_deserialize(
                &event.keys,
                key_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "game_id",
                            "GameOver", e
                        ),
                    );
                }
            };
            key_offset += starknet_rust::core::types::Felt::cairo_serialized_size(&game_id);
            let player_a = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.data,
                data_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "player_a",
                            "GameOver", e
                        ),
                    );
                }
            };
            data_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                    &player_a,
                );
            let player_b = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.data,
                data_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "player_b",
                            "GameOver", e
                        ),
                    );
                }
            };
            data_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                    &player_b,
                );
            let outcome = match Outcome::cairo_deserialize(&event.data, data_offset) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "outcome",
                            "GameOver", e
                        ),
                    );
                }
            };
            data_offset += Outcome::cairo_serialized_size(&outcome);
            return Ok(
                Event::GameOver(GameOverEvent {
                    game_id,
                    player_a,
                    player_b,
                    outcome,
                }),
            );
        }
        let selector = event.keys[0];
        if selector
            == starknet_rust::core::utils::get_selector_from_name("OwnershipTransferred")
                .unwrap_or_else(|_| {
                    panic!("Invalid selector for {}", "OwnershipTransferred")
                })
        {
            let mut key_offset = 0 + 1;
            let mut data_offset = 0;
            let previous_owner = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.keys,
                key_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}",
                            "previous_owner", "OwnershipTransferred", e
                        ),
                    );
                }
            };
            key_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                    &previous_owner,
                );
            let new_owner = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.keys,
                key_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "new_owner",
                            "OwnershipTransferred", e
                        ),
                    );
                }
            };
            key_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                    &new_owner,
                );
            return Ok(
                Event::OwnableEvent(
                    OwnableComponentEvent::OwnershipTransferred(OwnershipTransferred {
                        previous_owner,
                        new_owner,
                    }),
                ),
            );
        }
        let selector = event.keys[0];
        if selector
            == starknet_rust::core::utils::get_selector_from_name("OwnershipTransferStarted")
                .unwrap_or_else(|_| {
                    panic!("Invalid selector for {}", "OwnershipTransferStarted")
                })
        {
            let mut key_offset = 0 + 1;
            let mut data_offset = 0;
            let previous_owner = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.keys,
                key_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}",
                            "previous_owner", "OwnershipTransferStarted", e
                        ),
                    );
                }
            };
            key_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                    &previous_owner,
                );
            let new_owner = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.keys,
                key_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "new_owner",
                            "OwnershipTransferStarted", e
                        ),
                    );
                }
            };
            key_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                    &new_owner,
                );
            return Ok(
                Event::OwnableEvent(
                    OwnableComponentEvent::OwnershipTransferStarted(OwnershipTransferStarted {
                        previous_owner,
                        new_owner,
                    }),
                ),
            );
        }
        Err(format!("Could not match any event from keys {:?}", event.keys))
    }
}
impl TryFrom<&starknet_rust::core::types::Event> for Event {
    type Error = String;
    fn try_from(event: &starknet_rust::core::types::Event) -> Result<Self, Self::Error> {
        use cainome::cairo_serde::CairoSerde;
        if event.keys.is_empty() {
            return Err("Event has no key".to_string());
        }
        let selector = event.keys[0];
        if selector
            == starknet_rust::core::utils::get_selector_from_name("PlayerEntererLobby")
                .unwrap_or_else(|_| {
                    panic!("Invalid selector for {}", "PlayerEntererLobby")
                })
        {
            let mut key_offset = 0 + 1;
            let mut data_offset = 0;
            let lobby = match BoardSize::cairo_deserialize(&event.keys, key_offset) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "lobby",
                            "PlayerEntererLobby", e
                        ),
                    );
                }
            };
            key_offset += BoardSize::cairo_serialized_size(&lobby);
            let player = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.data,
                data_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "player",
                            "PlayerEntererLobby", e
                        ),
                    );
                }
            };
            data_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(&player);
            return Ok(
                Event::PlayerEntererLobby(PlayerEnteredLobbyEvent {
                    lobby,
                    player,
                }),
            );
        }
        let selector = event.keys[0];
        if selector
            == starknet_rust::core::utils::get_selector_from_name("PlayersAssembled")
                .unwrap_or_else(|_| {
                    panic!("Invalid selector for {}", "PlayersAssembled")
                })
        {
            let mut key_offset = 0 + 1;
            let mut data_offset = 0;
            let game_id = match starknet_rust::core::types::Felt::cairo_deserialize(
                &event.keys,
                key_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "game_id",
                            "PlayersAssembled", e
                        ),
                    );
                }
            };
            key_offset += starknet_rust::core::types::Felt::cairo_serialized_size(&game_id);
            let player_a = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.data,
                data_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "player_a",
                            "PlayersAssembled", e
                        ),
                    );
                }
            };
            data_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                    &player_a,
                );
            let player_b = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.data,
                data_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "player_b",
                            "PlayersAssembled", e
                        ),
                    );
                }
            };
            data_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                    &player_b,
                );
            let board_size = match BoardSize::cairo_deserialize(
                &event.data,
                data_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "board_size",
                            "PlayersAssembled", e
                        ),
                    );
                }
            };
            data_offset += BoardSize::cairo_serialized_size(&board_size);
            return Ok(
                Event::PlayersAssembled(PlayersAssembledEvent {
                    game_id,
                    player_a,
                    player_b,
                    board_size,
                }),
            );
        }
        let selector = event.keys[0];
        if selector
            == starknet_rust::core::utils::get_selector_from_name("GameStarted")
                .unwrap_or_else(|_| panic!("Invalid selector for {}", "GameStarted"))
        {
            let mut key_offset = 0 + 1;
            let mut data_offset = 0;
            let game_id = match starknet_rust::core::types::Felt::cairo_deserialize(
                &event.keys,
                key_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "game_id",
                            "GameStarted", e
                        ),
                    );
                }
            };
            key_offset += starknet_rust::core::types::Felt::cairo_serialized_size(&game_id);
            let attacker = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.data,
                data_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "attacker",
                            "GameStarted", e
                        ),
                    );
                }
            };
            data_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                    &attacker,
                );
            let defender = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.data,
                data_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "defender",
                            "GameStarted", e
                        ),
                    );
                }
            };
            data_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                    &defender,
                );
            return Ok(
                Event::GameStarted(GameStartedEvent {
                    game_id,
                    attacker,
                    defender,
                }),
            );
        }
        let selector = event.keys[0];
        if selector
            == starknet_rust::core::utils::get_selector_from_name("Attack")
                .unwrap_or_else(|_| panic!("Invalid selector for {}", "Attack"))
        {
            let mut key_offset = 0 + 1;
            let mut data_offset = 0;
            let game_id = match starknet_rust::core::types::Felt::cairo_deserialize(
                &event.keys,
                key_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "game_id",
                            "Attack", e
                        ),
                    );
                }
            };
            key_offset += starknet_rust::core::types::Felt::cairo_serialized_size(&game_id);
            let player = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.data,
                data_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "player",
                            "Attack", e
                        ),
                    );
                }
            };
            data_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(&player);
            let x = match u8::cairo_deserialize(&event.data, data_offset) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "x", "Attack",
                            e
                        ),
                    );
                }
            };
            data_offset += u8::cairo_serialized_size(&x);
            let y = match u8::cairo_deserialize(&event.data, data_offset) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "y", "Attack",
                            e
                        ),
                    );
                }
            };
            data_offset += u8::cairo_serialized_size(&y);
            return Ok(
                Event::Attack(AttackEvent {
                    game_id,
                    player,
                    x,
                    y,
                }),
            );
        }
        let selector = event.keys[0];
        if selector
            == starknet_rust::core::utils::get_selector_from_name("AttackResult")
                .unwrap_or_else(|_| panic!("Invalid selector for {}", "AttackResult"))
        {
            let mut key_offset = 0 + 1;
            let mut data_offset = 0;
            let game_id = match starknet_rust::core::types::Felt::cairo_deserialize(
                &event.keys,
                key_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "game_id",
                            "AttackResult", e
                        ),
                    );
                }
            };
            key_offset += starknet_rust::core::types::Felt::cairo_serialized_size(&game_id);
            let attacker = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.data,
                data_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "attacker",
                            "AttackResult", e
                        ),
                    );
                }
            };
            data_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                    &attacker,
                );
            let defender = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.data,
                data_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "defender",
                            "AttackResult", e
                        ),
                    );
                }
            };
            data_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                    &defender,
                );
            let x = match u8::cairo_deserialize(&event.data, data_offset) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "x",
                            "AttackResult", e
                        ),
                    );
                }
            };
            data_offset += u8::cairo_serialized_size(&x);
            let y = match u8::cairo_deserialize(&event.data, data_offset) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "y",
                            "AttackResult", e
                        ),
                    );
                }
            };
            data_offset += u8::cairo_serialized_size(&y);
            let ship_kind = match Option::<
                ShipKind,
            >::cairo_deserialize(&event.data, data_offset) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "ship_kind",
                            "AttackResult", e
                        ),
                    );
                }
            };
            data_offset += Option::<ShipKind>::cairo_serialized_size(&ship_kind);
            return Ok(
                Event::AttackResult(AttackResultEvent {
                    game_id,
                    attacker,
                    defender,
                    x,
                    y,
                    ship_kind,
                }),
            );
        }
        let selector = event.keys[0];
        if selector
            == starknet_rust::core::utils::get_selector_from_name("GameRevealRequest")
                .unwrap_or_else(|_| {
                    panic!("Invalid selector for {}", "GameRevealRequest")
                })
        {
            let mut key_offset = 0 + 1;
            let mut data_offset = 0;
            let game_id = match starknet_rust::core::types::Felt::cairo_deserialize(
                &event.keys,
                key_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "game_id",
                            "GameRevealRequest", e
                        ),
                    );
                }
            };
            key_offset += starknet_rust::core::types::Felt::cairo_serialized_size(&game_id);
            let player_a = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.data,
                data_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "player_a",
                            "GameRevealRequest", e
                        ),
                    );
                }
            };
            data_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                    &player_a,
                );
            let player_b = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.data,
                data_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "player_b",
                            "GameRevealRequest", e
                        ),
                    );
                }
            };
            data_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                    &player_b,
                );
            return Ok(
                Event::GameRevealRequest(GameRevealRequestEvent {
                    game_id,
                    player_a,
                    player_b,
                }),
            );
        }
        let selector = event.keys[0];
        if selector
            == starknet_rust::core::utils::get_selector_from_name("GameOver")
                .unwrap_or_else(|_| panic!("Invalid selector for {}", "GameOver"))
        {
            let mut key_offset = 0 + 1;
            let mut data_offset = 0;
            let game_id = match starknet_rust::core::types::Felt::cairo_deserialize(
                &event.keys,
                key_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "game_id",
                            "GameOver", e
                        ),
                    );
                }
            };
            key_offset += starknet_rust::core::types::Felt::cairo_serialized_size(&game_id);
            let player_a = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.data,
                data_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "player_a",
                            "GameOver", e
                        ),
                    );
                }
            };
            data_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                    &player_a,
                );
            let player_b = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.data,
                data_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "player_b",
                            "GameOver", e
                        ),
                    );
                }
            };
            data_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                    &player_b,
                );
            let outcome = match Outcome::cairo_deserialize(&event.data, data_offset) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "outcome",
                            "GameOver", e
                        ),
                    );
                }
            };
            data_offset += Outcome::cairo_serialized_size(&outcome);
            return Ok(
                Event::GameOver(GameOverEvent {
                    game_id,
                    player_a,
                    player_b,
                    outcome,
                }),
            );
        }
        let selector = event.keys[0];
        if selector
            == starknet_rust::core::utils::get_selector_from_name("OwnershipTransferred")
                .unwrap_or_else(|_| {
                    panic!("Invalid selector for {}", "OwnershipTransferred")
                })
        {
            let mut key_offset = 0 + 1;
            let mut data_offset = 0;
            let previous_owner = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.keys,
                key_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}",
                            "previous_owner", "OwnershipTransferred", e
                        ),
                    );
                }
            };
            key_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                    &previous_owner,
                );
            let new_owner = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.keys,
                key_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "new_owner",
                            "OwnershipTransferred", e
                        ),
                    );
                }
            };
            key_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                    &new_owner,
                );
            return Ok(
                Event::OwnableEvent(
                    OwnableComponentEvent::OwnershipTransferred(OwnershipTransferred {
                        previous_owner,
                        new_owner,
                    }),
                ),
            );
        }
        let selector = event.keys[0];
        if selector
            == starknet_rust::core::utils::get_selector_from_name("OwnershipTransferStarted")
                .unwrap_or_else(|_| {
                    panic!("Invalid selector for {}", "OwnershipTransferStarted")
                })
        {
            let mut key_offset = 0 + 1;
            let mut data_offset = 0;
            let previous_owner = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.keys,
                key_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}",
                            "previous_owner", "OwnershipTransferStarted", e
                        ),
                    );
                }
            };
            key_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                    &previous_owner,
                );
            let new_owner = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.keys,
                key_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "new_owner",
                            "OwnershipTransferStarted", e
                        ),
                    );
                }
            };
            key_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                    &new_owner,
                );
            return Ok(
                Event::OwnableEvent(
                    OwnableComponentEvent::OwnershipTransferStarted(OwnershipTransferStarted {
                        previous_owner,
                        new_owner,
                    }),
                ),
            );
        }
        Err(format!("Could not match any event from keys {:?}", event.keys))
    }
}
#[derive(Debug, Clone)]
pub enum FireStatus {
    Miss(starknet_rust::core::types::Felt),
    Hit((ShipKind, starknet_rust::core::types::Felt)),
}
impl cainome::cairo_serde::CairoSerde for FireStatus {
    type RustType = Self;
    const SERIALIZED_SIZE: std::option::Option<usize> = std::option::Option::None;
    #[inline]
    fn cairo_serialized_size(__rust: &Self::RustType) -> usize {
        match __rust {
            FireStatus::Miss(val) => {
                starknet_rust::core::types::Felt::cairo_serialized_size(val) + 1
            }
            FireStatus::Hit(val) => {
                <(ShipKind, starknet_rust::core::types::Felt)>::cairo_serialized_size(val) + 1
            }
            _ => 0,
        }
    }
    fn cairo_serialize(__rust: &Self::RustType) -> Vec<starknet_rust::core::types::Felt> {
        match __rust {
            FireStatus::Miss(val) => {
                let mut temp = vec![];
                temp.extend(usize::cairo_serialize(&0usize));
                temp.extend(starknet_rust::core::types::Felt::cairo_serialize(val));
                temp
            }
            FireStatus::Hit(val) => {
                let mut temp = vec![];
                temp.extend(usize::cairo_serialize(&1usize));
                temp.extend(
                    <(ShipKind, starknet_rust::core::types::Felt)>::cairo_serialize(val),
                );
                temp
            }
            _ => vec![],
        }
    }
    fn cairo_deserialize(
        __felts: &[starknet_rust::core::types::Felt],
        __offset: usize,
    ) -> cainome::cairo_serde::Result<Self::RustType> {
        let __f = __felts[__offset];
        let __index = u128::from_be_bytes(__f.to_bytes_be()[16..].try_into().unwrap());
        match __index as usize {
            0usize => {
                Ok(
                    FireStatus::Miss(
                        starknet_rust::core::types::Felt::cairo_deserialize(
                            __felts,
                            __offset + 1,
                        )?,
                    ),
                )
            }
            1usize => {
                Ok(
                    FireStatus::Hit(
                        <(
                            ShipKind,
                            starknet_rust::core::types::Felt,
                        )>::cairo_deserialize(__felts, __offset + 1)?,
                    ),
                )
            }
            _ => {
                return Err(
                    cainome::cairo_serde::Error::Deserialize(
                        format!("Index not handle for enum {}", "FireStatus"),
                    ),
                );
            }
        }
    }
}
#[derive(Debug, Clone)]
pub enum LargerBoardSize {
    TwelveByTwelve,
    FourteenByFourteen,
    TwentyByTwenty,
}
impl cainome::cairo_serde::CairoSerde for LargerBoardSize {
    type RustType = Self;
    const SERIALIZED_SIZE: std::option::Option<usize> = std::option::Option::None;
    #[inline]
    fn cairo_serialized_size(__rust: &Self::RustType) -> usize {
        match __rust {
            LargerBoardSize::TwelveByTwelve => 1,
            LargerBoardSize::FourteenByFourteen => 1,
            LargerBoardSize::TwentyByTwenty => 1,
            _ => 0,
        }
    }
    fn cairo_serialize(__rust: &Self::RustType) -> Vec<starknet_rust::core::types::Felt> {
        match __rust {
            LargerBoardSize::TwelveByTwelve => usize::cairo_serialize(&0usize),
            LargerBoardSize::FourteenByFourteen => usize::cairo_serialize(&1usize),
            LargerBoardSize::TwentyByTwenty => usize::cairo_serialize(&2usize),
            _ => vec![],
        }
    }
    fn cairo_deserialize(
        __felts: &[starknet_rust::core::types::Felt],
        __offset: usize,
    ) -> cainome::cairo_serde::Result<Self::RustType> {
        let __f = __felts[__offset];
        let __index = u128::from_be_bytes(__f.to_bytes_be()[16..].try_into().unwrap());
        match __index as usize {
            0usize => Ok(LargerBoardSize::TwelveByTwelve),
            1usize => Ok(LargerBoardSize::FourteenByFourteen),
            2usize => Ok(LargerBoardSize::TwentyByTwenty),
            _ => {
                return Err(
                    cainome::cairo_serde::Error::Deserialize(
                        format!("Index not handle for enum {}", "LargerBoardSize"),
                    ),
                );
            }
        }
    }
}
#[derive(Debug, Clone)]
pub enum Outcome {
    Fair(cainome::cairo_serde::ContractAddress),
    FailedToProvideProof(cainome::cairo_serde::ContractAddress),
    Null,
}
impl cainome::cairo_serde::CairoSerde for Outcome {
    type RustType = Self;
    const SERIALIZED_SIZE: std::option::Option<usize> = std::option::Option::None;
    #[inline]
    fn cairo_serialized_size(__rust: &Self::RustType) -> usize {
        match __rust {
            Outcome::Fair(val) => {
                cainome::cairo_serde::ContractAddress::cairo_serialized_size(val) + 1
            }
            Outcome::FailedToProvideProof(val) => {
                cainome::cairo_serde::ContractAddress::cairo_serialized_size(val) + 1
            }
            Outcome::Null => 1,
            _ => 0,
        }
    }
    fn cairo_serialize(__rust: &Self::RustType) -> Vec<starknet_rust::core::types::Felt> {
        match __rust {
            Outcome::Fair(val) => {
                let mut temp = vec![];
                temp.extend(usize::cairo_serialize(&0usize));
                temp.extend(cainome::cairo_serde::ContractAddress::cairo_serialize(val));
                temp
            }
            Outcome::FailedToProvideProof(val) => {
                let mut temp = vec![];
                temp.extend(usize::cairo_serialize(&1usize));
                temp.extend(cainome::cairo_serde::ContractAddress::cairo_serialize(val));
                temp
            }
            Outcome::Null => usize::cairo_serialize(&2usize),
            _ => vec![],
        }
    }
    fn cairo_deserialize(
        __felts: &[starknet_rust::core::types::Felt],
        __offset: usize,
    ) -> cainome::cairo_serde::Result<Self::RustType> {
        let __f = __felts[__offset];
        let __index = u128::from_be_bytes(__f.to_bytes_be()[16..].try_into().unwrap());
        match __index as usize {
            0usize => {
                Ok(
                    Outcome::Fair(
                        cainome::cairo_serde::ContractAddress::cairo_deserialize(
                            __felts,
                            __offset + 1,
                        )?,
                    ),
                )
            }
            1usize => {
                Ok(
                    Outcome::FailedToProvideProof(
                        cainome::cairo_serde::ContractAddress::cairo_deserialize(
                            __felts,
                            __offset + 1,
                        )?,
                    ),
                )
            }
            2usize => Ok(Outcome::Null),
            _ => {
                return Err(
                    cainome::cairo_serde::Error::Deserialize(
                        format!("Index not handle for enum {}", "Outcome"),
                    ),
                );
            }
        }
    }
}
#[derive(Debug, Clone)]
pub enum OwnableComponentEvent {
    OwnershipTransferred(OwnershipTransferred),
    OwnershipTransferStarted(OwnershipTransferStarted),
}
impl cainome::cairo_serde::CairoSerde for OwnableComponentEvent {
    type RustType = Self;
    const SERIALIZED_SIZE: std::option::Option<usize> = std::option::Option::None;
    #[inline]
    fn cairo_serialized_size(__rust: &Self::RustType) -> usize {
        match __rust {
            OwnableComponentEvent::OwnershipTransferred(val) => {
                OwnershipTransferred::cairo_serialized_size(val) + 1
            }
            OwnableComponentEvent::OwnershipTransferStarted(val) => {
                OwnershipTransferStarted::cairo_serialized_size(val) + 1
            }
            _ => 0,
        }
    }
    fn cairo_serialize(__rust: &Self::RustType) -> Vec<starknet_rust::core::types::Felt> {
        match __rust {
            OwnableComponentEvent::OwnershipTransferred(val) => {
                let mut temp = vec![];
                temp.extend(usize::cairo_serialize(&0usize));
                temp.extend(OwnershipTransferred::cairo_serialize(val));
                temp
            }
            OwnableComponentEvent::OwnershipTransferStarted(val) => {
                let mut temp = vec![];
                temp.extend(usize::cairo_serialize(&1usize));
                temp.extend(OwnershipTransferStarted::cairo_serialize(val));
                temp
            }
            _ => vec![],
        }
    }
    fn cairo_deserialize(
        __felts: &[starknet_rust::core::types::Felt],
        __offset: usize,
    ) -> cainome::cairo_serde::Result<Self::RustType> {
        let __f = __felts[__offset];
        let __index = u128::from_be_bytes(__f.to_bytes_be()[16..].try_into().unwrap());
        match __index as usize {
            0usize => {
                Ok(
                    OwnableComponentEvent::OwnershipTransferred(
                        OwnershipTransferred::cairo_deserialize(__felts, __offset + 1)?,
                    ),
                )
            }
            1usize => {
                Ok(
                    OwnableComponentEvent::OwnershipTransferStarted(
                        OwnershipTransferStarted::cairo_deserialize(
                            __felts,
                            __offset + 1,
                        )?,
                    ),
                )
            }
            _ => {
                return Err(
                    cainome::cairo_serde::Error::Deserialize(
                        format!("Index not handle for enum {}", "OwnableComponentEvent"),
                    ),
                );
            }
        }
    }
}
impl TryFrom<&starknet_rust::core::types::EmittedEvent> for OwnableComponentEvent {
    type Error = String;
    fn try_from(
        event: &starknet_rust::core::types::EmittedEvent,
    ) -> Result<Self, Self::Error> {
        use cainome::cairo_serde::CairoSerde;
        if event.keys.is_empty() {
            return Err("Event has no key".to_string());
        }
        let selector = event.keys[0];
        if selector
            == starknet_rust::core::utils::get_selector_from_name("OwnershipTransferred")
                .unwrap_or_else(|_| {
                    panic!("Invalid selector for {}", "OwnershipTransferred")
                })
        {
            let mut key_offset = 0 + 1;
            let mut data_offset = 0;
            let previous_owner = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.keys,
                key_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}",
                            "previous_owner", "OwnershipTransferred", e
                        ),
                    );
                }
            };
            key_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                    &previous_owner,
                );
            let new_owner = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.keys,
                key_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "new_owner",
                            "OwnershipTransferred", e
                        ),
                    );
                }
            };
            key_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                    &new_owner,
                );
            return Ok(
                OwnableComponentEvent::OwnershipTransferred(OwnershipTransferred {
                    previous_owner,
                    new_owner,
                }),
            );
        }
        let selector = event.keys[0];
        if selector
            == starknet_rust::core::utils::get_selector_from_name("OwnershipTransferStarted")
                .unwrap_or_else(|_| {
                    panic!("Invalid selector for {}", "OwnershipTransferStarted")
                })
        {
            let mut key_offset = 0 + 1;
            let mut data_offset = 0;
            let previous_owner = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.keys,
                key_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}",
                            "previous_owner", "OwnershipTransferStarted", e
                        ),
                    );
                }
            };
            key_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                    &previous_owner,
                );
            let new_owner = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.keys,
                key_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "new_owner",
                            "OwnershipTransferStarted", e
                        ),
                    );
                }
            };
            key_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                    &new_owner,
                );
            return Ok(
                OwnableComponentEvent::OwnershipTransferStarted(OwnershipTransferStarted {
                    previous_owner,
                    new_owner,
                }),
            );
        }
        Err(format!("Could not match any event from keys {:?}", event.keys))
    }
}
impl TryFrom<&starknet_rust::core::types::Event> for OwnableComponentEvent {
    type Error = String;
    fn try_from(event: &starknet_rust::core::types::Event) -> Result<Self, Self::Error> {
        use cainome::cairo_serde::CairoSerde;
        if event.keys.is_empty() {
            return Err("Event has no key".to_string());
        }
        let selector = event.keys[0];
        if selector
            == starknet_rust::core::utils::get_selector_from_name("OwnershipTransferred")
                .unwrap_or_else(|_| {
                    panic!("Invalid selector for {}", "OwnershipTransferred")
                })
        {
            let mut key_offset = 0 + 1;
            let mut data_offset = 0;
            let previous_owner = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.keys,
                key_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}",
                            "previous_owner", "OwnershipTransferred", e
                        ),
                    );
                }
            };
            key_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                    &previous_owner,
                );
            let new_owner = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.keys,
                key_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "new_owner",
                            "OwnershipTransferred", e
                        ),
                    );
                }
            };
            key_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                    &new_owner,
                );
            return Ok(
                OwnableComponentEvent::OwnershipTransferred(OwnershipTransferred {
                    previous_owner,
                    new_owner,
                }),
            );
        }
        let selector = event.keys[0];
        if selector
            == starknet_rust::core::utils::get_selector_from_name("OwnershipTransferStarted")
                .unwrap_or_else(|_| {
                    panic!("Invalid selector for {}", "OwnershipTransferStarted")
                })
        {
            let mut key_offset = 0 + 1;
            let mut data_offset = 0;
            let previous_owner = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.keys,
                key_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}",
                            "previous_owner", "OwnershipTransferStarted", e
                        ),
                    );
                }
            };
            key_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                    &previous_owner,
                );
            let new_owner = match cainome::cairo_serde::ContractAddress::cairo_deserialize(
                &event.keys,
                key_offset,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return Err(
                        format!(
                            "Could not deserialize field {} for {}: {:?}", "new_owner",
                            "OwnershipTransferStarted", e
                        ),
                    );
                }
            };
            key_offset
                += cainome::cairo_serde::ContractAddress::cairo_serialized_size(
                    &new_owner,
                );
            return Ok(
                OwnableComponentEvent::OwnershipTransferStarted(OwnershipTransferStarted {
                    previous_owner,
                    new_owner,
                }),
            );
        }
        Err(format!("Could not match any event from keys {:?}", event.keys))
    }
}
#[derive(Debug, Clone)]
pub enum ShipKind {
    Carrier,
    Battleship,
    Cruiser,
    Submarine,
    Destroyer,
    SuperCarrier,
}
impl cainome::cairo_serde::CairoSerde for ShipKind {
    type RustType = Self;
    const SERIALIZED_SIZE: std::option::Option<usize> = std::option::Option::None;
    #[inline]
    fn cairo_serialized_size(__rust: &Self::RustType) -> usize {
        match __rust {
            ShipKind::Carrier => 1,
            ShipKind::Battleship => 1,
            ShipKind::Cruiser => 1,
            ShipKind::Submarine => 1,
            ShipKind::Destroyer => 1,
            ShipKind::SuperCarrier => 1,
            _ => 0,
        }
    }
    fn cairo_serialize(__rust: &Self::RustType) -> Vec<starknet_rust::core::types::Felt> {
        match __rust {
            ShipKind::Carrier => usize::cairo_serialize(&0usize),
            ShipKind::Battleship => usize::cairo_serialize(&1usize),
            ShipKind::Cruiser => usize::cairo_serialize(&2usize),
            ShipKind::Submarine => usize::cairo_serialize(&3usize),
            ShipKind::Destroyer => usize::cairo_serialize(&4usize),
            ShipKind::SuperCarrier => usize::cairo_serialize(&5usize),
            _ => vec![],
        }
    }
    fn cairo_deserialize(
        __felts: &[starknet_rust::core::types::Felt],
        __offset: usize,
    ) -> cainome::cairo_serde::Result<Self::RustType> {
        let __f = __felts[__offset];
        let __index = u128::from_be_bytes(__f.to_bytes_be()[16..].try_into().unwrap());
        match __index as usize {
            0usize => Ok(ShipKind::Carrier),
            1usize => Ok(ShipKind::Battleship),
            2usize => Ok(ShipKind::Cruiser),
            3usize => Ok(ShipKind::Submarine),
            4usize => Ok(ShipKind::Destroyer),
            5usize => Ok(ShipKind::SuperCarrier),
            _ => {
                return Err(
                    cainome::cairo_serde::Error::Deserialize(
                        format!("Index not handle for enum {}", "ShipKind"),
                    ),
                );
            }
        }
    }
}
#[derive(Debug, Clone)]
pub enum SmallerBoardSize {
    SixBySix,
    EightByEight,
}
impl cainome::cairo_serde::CairoSerde for SmallerBoardSize {
    type RustType = Self;
    const SERIALIZED_SIZE: std::option::Option<usize> = std::option::Option::None;
    #[inline]
    fn cairo_serialized_size(__rust: &Self::RustType) -> usize {
        match __rust {
            SmallerBoardSize::SixBySix => 1,
            SmallerBoardSize::EightByEight => 1,
            _ => 0,
        }
    }
    fn cairo_serialize(__rust: &Self::RustType) -> Vec<starknet_rust::core::types::Felt> {
        match __rust {
            SmallerBoardSize::SixBySix => usize::cairo_serialize(&0usize),
            SmallerBoardSize::EightByEight => usize::cairo_serialize(&1usize),
            _ => vec![],
        }
    }
    fn cairo_deserialize(
        __felts: &[starknet_rust::core::types::Felt],
        __offset: usize,
    ) -> cainome::cairo_serde::Result<Self::RustType> {
        let __f = __felts[__offset];
        let __index = u128::from_be_bytes(__f.to_bytes_be()[16..].try_into().unwrap());
        match __index as usize {
            0usize => Ok(SmallerBoardSize::SixBySix),
            1usize => Ok(SmallerBoardSize::EightByEight),
            _ => {
                return Err(
                    cainome::cairo_serde::Error::Deserialize(
                        format!("Index not handle for enum {}", "SmallerBoardSize"),
                    ),
                );
            }
        }
    }
}
impl<A: starknet_rust::accounts::ConnectedAccount + Sync> Starkwaves<A> {
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn get_next_game_id(
        &self,
    ) -> cainome::cairo_serde::call::FCall<A::Provider, starknet_rust::core::types::Felt> {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        let __call = starknet_rust::core::types::FunctionCall {
            contract_address: self.address,
            entry_point_selector: starknet_rust::macros::selector!("get_next_game_id"),
            calldata: __calldata,
        };
        cainome::cairo_serde::call::FCall::new(__call, self.provider())
    }
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn owner(
        &self,
    ) -> cainome::cairo_serde::call::FCall<
        A::Provider,
        cainome::cairo_serde::ContractAddress,
    > {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        let __call = starknet_rust::core::types::FunctionCall {
            contract_address: self.address,
            entry_point_selector: starknet_rust::macros::selector!("owner"),
            calldata: __calldata,
        };
        cainome::cairo_serde::call::FCall::new(__call, self.provider())
    }
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn attack_getcall(
        &self,
        game_id: &starknet_rust::core::types::Felt,
        x: &u8,
        y: &u8,
    ) -> starknet_rust::core::types::Call {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        __calldata.extend(starknet_rust::core::types::Felt::cairo_serialize(game_id));
        __calldata.extend(u8::cairo_serialize(x));
        __calldata.extend(u8::cairo_serialize(y));
        starknet_rust::core::types::Call {
            to: self.address,
            selector: starknet_rust::macros::selector!("attack"),
            calldata: __calldata,
        }
    }
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn attack(
        &self,
        game_id: &starknet_rust::core::types::Felt,
        x: &u8,
        y: &u8,
    ) -> starknet_rust::accounts::ExecutionV3<A> {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        __calldata.extend(starknet_rust::core::types::Felt::cairo_serialize(game_id));
        __calldata.extend(u8::cairo_serialize(x));
        __calldata.extend(u8::cairo_serialize(y));
        let __call = starknet_rust::core::types::Call {
            to: self.address,
            selector: starknet_rust::macros::selector!("attack"),
            calldata: __calldata,
        };
        self.account.execute_v3(vec![__call])
    }
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn commit_board_getcall(
        &self,
        root: &starknet_rust::core::types::Felt,
        game_id: &starknet_rust::core::types::Felt,
    ) -> starknet_rust::core::types::Call {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        __calldata.extend(starknet_rust::core::types::Felt::cairo_serialize(root));
        __calldata.extend(starknet_rust::core::types::Felt::cairo_serialize(game_id));
        starknet_rust::core::types::Call {
            to: self.address,
            selector: starknet_rust::macros::selector!("commit_board"),
            calldata: __calldata,
        }
    }
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn commit_board(
        &self,
        root: &starknet_rust::core::types::Felt,
        game_id: &starknet_rust::core::types::Felt,
    ) -> starknet_rust::accounts::ExecutionV3<A> {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        __calldata.extend(starknet_rust::core::types::Felt::cairo_serialize(root));
        __calldata.extend(starknet_rust::core::types::Felt::cairo_serialize(game_id));
        let __call = starknet_rust::core::types::Call {
            to: self.address,
            selector: starknet_rust::macros::selector!("commit_board"),
            calldata: __calldata,
        };
        self.account.execute_v3(vec![__call])
    }
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn defend_getcall(
        &self,
        game_id: &starknet_rust::core::types::Felt,
        status: &FireStatus,
        proof: &Vec<starknet_rust::core::types::Felt>,
    ) -> starknet_rust::core::types::Call {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        __calldata.extend(starknet_rust::core::types::Felt::cairo_serialize(game_id));
        __calldata.extend(FireStatus::cairo_serialize(status));
        __calldata.extend(Vec::<starknet_rust::core::types::Felt>::cairo_serialize(proof));
        starknet_rust::core::types::Call {
            to: self.address,
            selector: starknet_rust::macros::selector!("defend"),
            calldata: __calldata,
        }
    }
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn defend(
        &self,
        game_id: &starknet_rust::core::types::Felt,
        status: &FireStatus,
        proof: &Vec<starknet_rust::core::types::Felt>,
    ) -> starknet_rust::accounts::ExecutionV3<A> {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        __calldata.extend(starknet_rust::core::types::Felt::cairo_serialize(game_id));
        __calldata.extend(FireStatus::cairo_serialize(status));
        __calldata.extend(Vec::<starknet_rust::core::types::Felt>::cairo_serialize(proof));
        let __call = starknet_rust::core::types::Call {
            to: self.address,
            selector: starknet_rust::macros::selector!("defend"),
            calldata: __calldata,
        };
        self.account.execute_v3(vec![__call])
    }
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn renounce_ownership_getcall(&self) -> starknet_rust::core::types::Call {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        starknet_rust::core::types::Call {
            to: self.address,
            selector: starknet_rust::macros::selector!("renounce_ownership"),
            calldata: __calldata,
        }
    }
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn renounce_ownership(&self) -> starknet_rust::accounts::ExecutionV3<A> {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        let __call = starknet_rust::core::types::Call {
            to: self.address,
            selector: starknet_rust::macros::selector!("renounce_ownership"),
            calldata: __calldata,
        };
        self.account.execute_v3(vec![__call])
    }
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn request_start_game_getcall(
        &self,
        board_size: &BoardSize,
    ) -> starknet_rust::core::types::Call {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        __calldata.extend(BoardSize::cairo_serialize(board_size));
        starknet_rust::core::types::Call {
            to: self.address,
            selector: starknet_rust::macros::selector!("request_start_game"),
            calldata: __calldata,
        }
    }
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn request_start_game(
        &self,
        board_size: &BoardSize,
    ) -> starknet_rust::accounts::ExecutionV3<A> {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        __calldata.extend(BoardSize::cairo_serialize(board_size));
        let __call = starknet_rust::core::types::Call {
            to: self.address,
            selector: starknet_rust::macros::selector!("request_start_game"),
            calldata: __calldata,
        };
        self.account.execute_v3(vec![__call])
    }
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn reset_getcall(&self) -> starknet_rust::core::types::Call {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        starknet_rust::core::types::Call {
            to: self.address,
            selector: starknet_rust::macros::selector!("reset"),
            calldata: __calldata,
        }
    }
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn reset(&self) -> starknet_rust::accounts::ExecutionV3<A> {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        let __call = starknet_rust::core::types::Call {
            to: self.address,
            selector: starknet_rust::macros::selector!("reset"),
            calldata: __calldata,
        };
        self.account.execute_v3(vec![__call])
    }
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn reveal_getcall(
        &self,
        game_id: &starknet_rust::core::types::Felt,
        board: &Vec<u8>,
        salt: &starknet_rust::core::types::Felt,
    ) -> starknet_rust::core::types::Call {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        __calldata.extend(starknet_rust::core::types::Felt::cairo_serialize(game_id));
        __calldata.extend(Vec::<u8>::cairo_serialize(board));
        __calldata.extend(starknet_rust::core::types::Felt::cairo_serialize(salt));
        starknet_rust::core::types::Call {
            to: self.address,
            selector: starknet_rust::macros::selector!("reveal"),
            calldata: __calldata,
        }
    }
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn reveal(
        &self,
        game_id: &starknet_rust::core::types::Felt,
        board: &Vec<u8>,
        salt: &starknet_rust::core::types::Felt,
    ) -> starknet_rust::accounts::ExecutionV3<A> {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        __calldata.extend(starknet_rust::core::types::Felt::cairo_serialize(game_id));
        __calldata.extend(Vec::<u8>::cairo_serialize(board));
        __calldata.extend(starknet_rust::core::types::Felt::cairo_serialize(salt));
        let __call = starknet_rust::core::types::Call {
            to: self.address,
            selector: starknet_rust::macros::selector!("reveal"),
            calldata: __calldata,
        };
        self.account.execute_v3(vec![__call])
    }
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn transfer_ownership_getcall(
        &self,
        new_owner: &cainome::cairo_serde::ContractAddress,
    ) -> starknet_rust::core::types::Call {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        __calldata
            .extend(cainome::cairo_serde::ContractAddress::cairo_serialize(new_owner));
        starknet_rust::core::types::Call {
            to: self.address,
            selector: starknet_rust::macros::selector!("transfer_ownership"),
            calldata: __calldata,
        }
    }
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn transfer_ownership(
        &self,
        new_owner: &cainome::cairo_serde::ContractAddress,
    ) -> starknet_rust::accounts::ExecutionV3<A> {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        __calldata
            .extend(cainome::cairo_serde::ContractAddress::cairo_serialize(new_owner));
        let __call = starknet_rust::core::types::Call {
            to: self.address,
            selector: starknet_rust::macros::selector!("transfer_ownership"),
            calldata: __calldata,
        };
        self.account.execute_v3(vec![__call])
    }
}
impl<P: starknet_rust::providers::Provider + Sync> StarkwavesReader<P> {
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn get_next_game_id(
        &self,
    ) -> cainome::cairo_serde::call::FCall<P, starknet_rust::core::types::Felt> {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        let __call = starknet_rust::core::types::FunctionCall {
            contract_address: self.address,
            entry_point_selector: starknet_rust::macros::selector!("get_next_game_id"),
            calldata: __calldata,
        };
        cainome::cairo_serde::call::FCall::new(__call, self.provider())
    }
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn owner(
        &self,
    ) -> cainome::cairo_serde::call::FCall<P, cainome::cairo_serde::ContractAddress> {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        let __call = starknet_rust::core::types::FunctionCall {
            contract_address: self.address,
            entry_point_selector: starknet_rust::macros::selector!("owner"),
            calldata: __calldata,
        };
        cainome::cairo_serde::call::FCall::new(__call, self.provider())
    }
}
