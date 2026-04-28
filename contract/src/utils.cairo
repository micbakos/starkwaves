use core::dict::Felt252Dict;
use core::nullable::{FromNullableResult, match_nullable};
use core::pedersen::pedersen;
use crate::types::ship::HullSection;
use crate::types::{BoardSize, BoardSizeTrait, ShipKindTrait};

/// Converts a linear board offset back to (x, y) cartesian coordinates
/// given the board size. Inverse of `cartesian_to_offset`.
pub fn offset_to_cartesian(size: @BoardSize, offset: u32) -> (u8, u8) {
    let board_size: u32 = size.size().into();
    let x: u8 = (offset / board_size).try_into().unwrap();
    let y: u8 = (offset % board_size).try_into().unwrap();
    (x, y)
}

/// Converts (x, y) cartesian coordinates to a linear offset using row-major order.
/// `offset = x * board_size + y`
pub fn cartesian_to_offset(size: @BoardSize, x: u8, y: u8) -> u32 {
    let rows_offset: u32 = x.into() * size.size().into();
    rows_offset + y.into()
}

/// Encodes (x, y) coordinates as a two-byte pair (high, low) suitable for storage
/// in a `ByteArray`. The linear offset is split into `high = offset / 256` and
/// `low = offset % 256`, supporting boards up to ~256x256.
pub fn cartesian_as_bytes(size: @BoardSize, x: u8, y: u8) -> (u8, u8) {
    let offset = cartesian_to_offset(size, x, y);
    let high_byte: u8 = (offset / 256).try_into().unwrap();
    let low_byte: u8 = (offset % 256).try_into().unwrap();

    (high_byte, low_byte)
}

/// Decodes a two-byte pair (high, low) back into a linear board offset.
/// Inverse of the encoding in `cartesian_as_bytes`.
pub fn bytes_to_offset(size: @BoardSize, high: u8, low: u8) -> u32 {
    let mut offset: u32 = 0;

    offset = high.into() * 256;
    offset += low.into();
    offset
}

/// Appends a bomb at (x, y) to the given `ByteArray` by encoding the coordinates
/// as a two-byte pair and appending both bytes.
pub fn append_bomb_at(ref self: ByteArray, size: @BoardSize, x: u8, y: u8) {
    let (high, low) = cartesian_as_bytes(size, x, y);

    self.append_byte(high);
    self.append_byte(low);
}

/// Checks whether a bomb at (x, y) exists in the given `ByteArray`.
/// Scans the byte array in two-byte strides looking for a matching (high, low) pair.
pub fn contains_bomb_at(self: @ByteArray, size: @BoardSize, x: u8, y: u8) -> bool {
    let (high, low) = cartesian_as_bytes(size, x, y);

    let len = self.len();
    let mut i = 0;

    while i < len {
        if i + 1 < len {
            let _high = self.at(i).unwrap();
            let _low = self.at(i + 1).unwrap();
            if high == _high && low == _low {
                return true;
            }
        }
        i = i + 2;
    }

    false
}

/// Returns the linear board offset of the bomb placed at a given turn index,
/// or `None` if the turn has no recorded bomb. Each turn occupies two bytes
/// in the `ByteArray` at positions `[turn*2, turn*2+1]`.
pub fn get_bomb_offset_at_turn(self: @ByteArray, size: @BoardSize, turn: u32) -> Option<u32> {
    if let (Some(high), Some(low)) = (self.at(turn * 2), self.at(turn * 2 + 1)) {
        Some(bytes_to_offset(size, high, low))
    } else {
        None
    }
}

pub fn verify_destructions(
    ref hulls: Felt252Dict<Nullable<HullSection>>,
    board_size: @BoardSize,
    destruction_hash: felt252,
    bombs: @ByteArray,
) -> bool {
    let mut confirmed_hits: Felt252Dict<u8> = Default::default();
    let mut reconstructed_hash: felt252 = 0;

    let all_turns = bombs.len() / 2;
    for turn in 0_u32..all_turns {
        let maybe_offset = get_bomb_offset_at_turn(bombs, board_size, turn);

        if let Some(offset) = maybe_offset {
            match match_nullable(hulls.get(offset.into())) {
                FromNullableResult::Null => { // Miss: water cell, nothing to track
                },
                FromNullableResult::NotNull(section) => {
                    let ship_id = section.ship_id;
                    let potential_hits = section.ship_kind.length();

                    let current_hits = confirmed_hits.get(ship_id) + 1;

                    confirmed_hits.insert(ship_id, current_hits);
                    if current_hits == potential_hits {
                        reconstructed_hash =
                            pedersen(reconstructed_hash, section.ship_kind.id().into());
                    }
                },
            }
        } else {
            return false;
        }
    }

    reconstructed_hash == destruction_hash
}
