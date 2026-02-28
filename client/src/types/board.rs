use super::orientation::Orientation;
use super::result::Result;
use super::ship::Ship;
use crate::merkle::board_merkle_tree::BoardMerkleTree;
use crate::types::board_size::BoardSize;
use crate::types::error::GameError::{AllShipsPlaced, BoardAlreadyCommitted, BoardNotReady, BombedAlready, GameOver, InvalidShipPlacementBounds, InvalidShipPlacementCollides, InvalidShipPlacementKind};
use crate::types::fire_report::FireReport;
use crate::types::{Cell, ShipKind};
use starknet::core::types::Felt;
use std::collections::{HashMap, HashSet};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    size: BoardSize,
    ships: Vec<Ship>,
    commitment_tree: Option<BoardMerkleTree>,
    received_fire: Vec<usize>,
    launched_fire: Vec<usize>,
}

impl Board {
    pub fn new(size: BoardSize) -> Board {
        Board {
            size,
            ships: Vec::new(),
            commitment_tree: None,
            received_fire: Vec::new(),
            launched_fire: Vec::new(),
        }
    }

    /// Places a ship on the board
    pub fn place_ship(&mut self, ship: Ship) -> Result<()> {
        if self.commitment_tree.is_some() {
            return Err(BoardAlreadyCommitted);
        }

        if self.is_board_ready() {
            return Err(AllShipsPlaced);
        }

        let counts = self.size.ship_kinds_count(&ship.kind);

        let mut ship_kind_occurrences = 0;
        self.ships.iter().for_each(|s| {
            if s.kind == ship.kind {
                ship_kind_occurrences += 1;
            }
        });

        if ship_kind_occurrences >= counts {
            Err(InvalidShipPlacementKind { kind: ship.kind })?;
        }

        let board_size = self.size.size();
        let length = ship.kind.length();

        // Check if ship fits on board
        match ship.orientation {
            Orientation::Horizontal => {
                if ship.y + length > board_size {
                    return Err(InvalidShipPlacementBounds {
                        ship: ship.kind,
                        x: ship.x,
                        y: ship.y,
                        orientation: ship.orientation,
                    });
                }
            }
            Orientation::Vertical => {
                if ship.x + length > board_size {
                    return Err(InvalidShipPlacementBounds {
                        ship: ship.kind,
                        x: ship.x,
                        y: ship.y,
                        orientation: ship.orientation,
                    });
                }
            }
        }

        let cells_before_insert = self.cells();

        // Check for overlaps and place ship
        for i in 0..length {
            let (x, y) = match ship.orientation {
                Orientation::Horizontal => (ship.x, ship.y + i),
                Orientation::Vertical => (ship.x + i, ship.y),
            };

            let offset = self.to_offset(x, y);
            if cells_before_insert[offset] != Cell::Water {
                return Err(InvalidShipPlacementCollides {
                    ship: ship.kind,
                    x: ship.x,
                    y: ship.y,
                    orientation: Orientation::Horizontal,
                    xc: x,
                    yc: y,
                });
            }
        }

        self.ships.push(ship);
        Ok(())
    }

    /// Generates a merkle tree that proves the state of the board on each cell in
    /// combination with the given salt.
    pub fn commit(&mut self, salt: u64) -> Result<Felt> {
        if self.commitment_tree.is_some() {
            return Err(BoardAlreadyCommitted);
        }

        let array = self.to_array()?;

        let tree = BoardMerkleTree::build(array, salt);
        self.commitment_tree = Some(tree.clone());
        Ok(tree.root())
    }

    pub fn receive_fire(&mut self, x: u8, y: u8) -> Result<FireReport> {
        let offset = self.to_offset(x, y);
        if self
            .received_fire
            .iter()
            .cloned()
            .find(|index| *index == offset)
            .is_some()
        {
            return Err(BombedAlready { x, y });
        }

        let hits = self.hit_ships();

        let destroyed = hits.clone().into_iter().filter(|(id, hits)| {
            let ship = self.ships.iter().find(|s| s.id == *id).expect("Ship should exist");
            ship.kind.length() == *hits
        }).map(|(id, _)| { id }).collect::<HashSet<_>>();

        let all_ships = self.ships.iter().map(|s| s.id).collect::<HashSet<_>>();
        if destroyed.symmetric_difference(&all_ships).count() == 0 {
            return Err(GameOver);
        }

        let proof = self
            .commitment_tree
            .clone()
            .map(|tree| tree.proof(offset))
            .ok_or(BoardNotReady)?;

        let cells = self.cells();
        let cell = cells[offset];

        let report = match cell {
            Cell::Water => FireReport::miss(proof),
            Cell::Ship(id) => {
                let ship = self
                    .ships
                    .iter()
                    .find(|ship| ship.id == id)
                    .expect("Ship should exist");
                let hit_count = hits.get(&id).unwrap_or(&0);

                if *hit_count == ship.kind.length() - 1 {
                    FireReport::hit_with_destruction(*ship, proof)
                } else {
                    FireReport::hit(ship.kind, proof)
                }
            }
        };

        self.received_fire.push(offset);

        Ok(report)
    }

    pub fn size(&self) -> BoardSize {
        self.size
    }

    pub fn is_board_ready(&self) -> bool {
        let mut kinds_placed = HashMap::<ShipKind, u8>::new();

        self.ships.iter().for_each(|ship| {
            let occurrences = kinds_placed.get(&ship.kind).unwrap_or(&0);
            kinds_placed.insert(ship.kind, occurrences + 1);
        });

        ShipKind::all().iter().all(|kind| {
            let occurrences = kinds_placed.get(&kind).unwrap_or(&0u8);
            kind.is_eligible(self.size, *occurrences)
        })
    }

    pub fn to_array(&self) -> Result<Vec<u8>> {
        if !self.is_board_ready() {
            Err(BoardNotReady)?;
        }

        let mut ids = Vec::<u8>::new();
        for cell in self.cells() {
            let id = cell
                .ship(&self.ships)
                .map(|ship| ship.kind.id())
                .unwrap_or(0);

            ids.push(id);
        }

        Ok(ids)
    }

    fn cells(&self) -> Vec<Cell> {
        let board_size = self.size.size();
        let mut cells = Vec::with_capacity((board_size * board_size) as usize);
        for _ in 0..cells.capacity() {
            cells.push(Cell::Water);
        }

        self.ships.iter().for_each(|ship| {
            let ship_len = ship.kind.length();

            for i in 0..ship_len {
                let (x, y) = match ship.orientation {
                    Orientation::Horizontal => (ship.x, ship.y + i),
                    Orientation::Vertical => (ship.x + i, ship.y),
                };

                let offset = self.to_offset(x, y);
                cells[offset] = Cell::Ship(ship.id);
            }
        });

        cells
    }

    pub fn track_launched_fire(&mut self, x: u8, y: u8) {
        let offset = self.to_offset(x, y);
        self.launched_fire.push(offset);
    }

    fn hit_ships(&mut self) -> HashMap<Uuid, u8> {
        let mut hits = HashMap::<Uuid, u8>::new();
        let cells = self.cells();

        self.received_fire.iter().for_each(|index| {
            let cell = cells[*index];
            if let Cell::Ship(ship_id) = cell {
                let bomb_count = hits.get(&ship_id).unwrap_or(&0);
                hits.insert(ship_id, bomb_count + 1);
            }
        });

        hits
    }

    fn to_offset(&self, x: u8, y: u8) -> usize {
        let size = self.size.size();
        let rows_offset = x * size;
        (rows_offset + y) as usize
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new(BoardSize::default())
    }
}

fn format_board_row(size: usize, items: &[&str], index: Option<usize>, bombed: &[bool]) -> String {
    let sanitize = |text: &str| {
        if text.len() != 2 && text.len() != 4 {
            panic!("Each text on grid should be 2 or 4 chars. Was `{}`.", text);
        }
        if text.len() == 2 {
            format!("  {}  ", text)
        } else {
            format!(" {} ", text)
        }
    };

    if items.len() > size {
        panic!("Items to draw should be at most {}", size);
    }

    let mut row = String::from("|");
    match index {
        None => row.push_str("      |"),
        Some(i) => {
            let label = if i + 1 < 10 {
                format!("0{}", i + 1)
            } else {
                (i + 1).to_string()
            };
            row = format!("{row}{}|", sanitize(&label));
        }
    }
    for i in 0..size {
        let cell = sanitize(items[i]);
        if bombed.get(i).copied().unwrap_or(false) {
            row = format!("{}\x1b[31m{}\x1b[0m|", row, cell);
        } else {
            row = format!("{}{}|", row, cell);
        }
    }
    row
}

fn write_board_grid(
    f: &mut fmt::Formatter<'_>,
    size: usize,
    rows: &[Vec<String>],
    fired: &[usize],
) -> fmt::Result {
    let divider_items = (0..size).map(|_| "----").collect::<Vec<_>>();
    let column_titles = (1..=size)
        .map(|i| if i < 10 { format!("0{i}") } else { format!("{i}") })
        .collect::<Vec<_>>();

    writeln!(f, "{}", format_board_row(size, divider_items.as_slice(), None, &[]))?;
    writeln!(f, "{}", format_board_row(
        size,
        column_titles.iter().map(|s| s.as_str()).collect::<Vec<_>>().as_slice(),
        None,
        &[],
    ))?;

    for (row_idx, row_cells) in rows.iter().enumerate() {
        let bombed_cols: Vec<bool> = (0..size)
            .map(|col_idx| fired.contains(&(row_idx * size + col_idx)))
            .collect();

        writeln!(f, "{}", format_board_row(
            size,
            row_cells.iter().map(|s| s.as_str()).collect::<Vec<_>>().as_slice(),
            Some(row_idx),
            &bombed_cols,
        ))?;
    }

    writeln!(f, "{}", format_board_row(size, divider_items.as_slice(), None, &[]))?;
    Ok(())
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let size = self.size.size() as usize;
        let cells = self.cells();
        let rows: Vec<Vec<String>> = cells
            .chunks(size)
            .map(|chunk| {
                chunk.iter().map(|cell| match cell {
                    Cell::Water => "~~".to_string(),
                    Cell::Ship(ship_id) => {
                        let ship = self
                            .ships
                            .iter()
                            .find(|s| s.id.as_bytes() == ship_id.as_bytes())
                            .expect(&format!("Ship id {} should exist but not found.", ship_id));
                        ship.kind.code().to_string()
                    }
                }).collect()
            })
            .collect();

        write_board_grid(f, size, &rows, &self.received_fire)
    }
}

pub struct LaunchedFireView<'a>(&'a Board);

impl Board {
    pub fn launched_fire_view(&self) -> LaunchedFireView<'_> {
        LaunchedFireView(self)
    }
}

impl fmt::Display for LaunchedFireView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let board = self.0;
        let size = board.size.size() as usize;
        let rows: Vec<Vec<String>> = (0..size)
            .map(|_| (0..size).map(|_| "??".to_string()).collect())
            .collect();

        write_board_grid(f, size, &rows, &board.launched_fire)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::board_size::{BoardSize, LargerBoardSize, SmallerBoardSize};
    use crate::types::fire_report::FireStatus;

    #[test]
    fn test_new_board_standard() {
        let board = Board::new(BoardSize::Standard);
        let cells = board.cells();
        assert_eq!(board.size, BoardSize::Standard);
        assert_eq!(cells.len(), 100); // 10x10
        assert!(cells.iter().all(|c| c == &Cell::Water));
        assert_eq!(board.ships.len(), 0);
    }

    #[test]
    fn test_new_board_smaller() {
        let board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        let cells = board.cells();
        assert_eq!(cells.len(), 36); // 6x6
        assert!(cells.iter().all(|c| c == &Cell::Water));
    }

    #[test]
    fn test_new_board_larger() {
        let board = Board::new(BoardSize::Larger(LargerBoardSize::TwelveByTwelve));
        let cells = board.cells();
        assert_eq!(cells.len(), 144); // 12x12
        assert!(cells.iter().all(|c| c == &Cell::Water));
    }

    #[test]
    fn test_default_board() {
        let board = Board::default();
        let cells = board.cells();
        assert_eq!(board.size, BoardSize::Standard);
        assert_eq!(cells.len(), 100);
    }

    #[test]
    fn test_place_ship_horizontal() {
        let mut board = Board::new(BoardSize::Standard);
        let ship = Ship::new(ShipKind::Destroyer, 2, 3, Orientation::Horizontal);

        assert!(board.place_ship(ship).is_ok());
        let cells = board.cells();

        // Verify ship is placed correctly (Destroyer has length 2)
        cells[board.to_offset(2, 3)].assert_kind(&board.ships, ShipKind::Destroyer);
        cells[board.to_offset(2, 4)].assert_kind(&board.ships, ShipKind::Destroyer);
        assert_eq!(board.ships.len(), 1);
        assert_eq!(board.ships[0].kind, ShipKind::Destroyer);
    }

    #[test]
    fn test_place_ship_vertical() {
        let mut board = Board::new(BoardSize::Standard);
        let ship = Ship::new(ShipKind::Cruiser, 1, 1, Orientation::Vertical);

        assert!(board.place_ship(ship).is_ok());
        let cells = board.cells();

        // Verify ship is placed correctly (Cruiser has length 3)
        cells[board.to_offset(1, 1)].assert_kind(&board.ships, ShipKind::Cruiser);
        cells[board.to_offset(2, 1)].assert_kind(&board.ships, ShipKind::Cruiser);
        cells[board.to_offset(3, 1)].assert_kind(&board.ships, ShipKind::Cruiser);
    }

    #[test]
    fn test_place_ship_out_of_bounds_horizontal() {
        let mut board = Board::new(BoardSize::Standard); // 10x10
        let ship = Ship::new(ShipKind::Carrier, 0, 9, Orientation::Horizontal); // Carrier length 5, y=9 + 5 > 10

        let result = board.place_ship(ship);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            InvalidShipPlacementBounds { .. }
        ));
    }

    #[test]
    fn test_place_ship_out_of_bounds_vertical() {
        let mut board = Board::new(BoardSize::Standard); // 10x10
        let ship = Ship::new(ShipKind::Battleship, 8, 0, Orientation::Vertical); // Battleship length 4, x=8 + 4 > 10

        let result = board.place_ship(ship);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            InvalidShipPlacementBounds { .. }
        ));
    }

    #[test]
    fn test_place_ship_collision_horizontal() {
        let mut board = Board::new(BoardSize::Standard);

        // Place first ship
        let ship1 = Ship::new(ShipKind::Destroyer, 2, 2, Orientation::Horizontal);
        assert!(board.place_ship(ship1).is_ok());

        // Try to place second ship that overlaps
        let ship2 = Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Horizontal);
        let result = board.place_ship(ship2);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            InvalidShipPlacementCollides { .. }
        ));
    }

    #[test]
    fn test_place_ship_collision_vertical() {
        let mut board = Board::new(BoardSize::Standard);

        // Place first ship
        let ship1 = Ship::new(ShipKind::Cruiser, 3, 3, Orientation::Vertical);
        assert!(board.place_ship(ship1).is_ok());

        // Try to place second ship that overlaps
        let ship2 = Ship::new(ShipKind::Destroyer, 4, 3, Orientation::Vertical);
        let result = board.place_ship(ship2);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            InvalidShipPlacementCollides { .. }
        ));
    }

    #[test]
    fn test_place_ship_too_many_of_same_kind() {
        let mut board = Board::new(BoardSize::Standard);

        // Place first Destroyer
        let ship1 = Ship::new(ShipKind::Destroyer, 0, 0, Orientation::Horizontal);
        assert!(board.place_ship(ship1).is_ok());

        // Try to place second Destroyer (Standard board only allows 1)
        let ship2 = Ship::new(ShipKind::Destroyer, 5, 5, Orientation::Horizontal);
        let result = board.place_ship(ship2);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            InvalidShipPlacementKind { .. }
        ));
    }

    #[test]
    fn test_place_multiple_ships_larger_board() {
        let mut board = Board::new(BoardSize::Larger(LargerBoardSize::TwelveByTwelve));

        // Larger boards allow 2 Destroyers
        let ship1 = Ship::new(ShipKind::Destroyer, 0, 0, Orientation::Horizontal);
        let ship2 = Ship::new(ShipKind::Destroyer, 5, 5, Orientation::Horizontal);

        assert!(board.place_ship(ship1).is_ok());
        assert!(board.place_ship(ship2).is_ok());
        assert_eq!(board.ships.len(), 2);

        // Third Destroyer should fail
        let ship3 = Ship::new(ShipKind::Destroyer, 8, 8, Orientation::Horizontal);
        let result = board.place_ship(ship3);
        assert!(result.is_err());
    }

    #[test]
    fn test_to_array_empty_board_fails() {
        let board = Board::new(BoardSize::Standard);
        let result = board.to_array();

        assert!(result.is_err(), "Empty board should not be ready");
    }

    #[test]
    fn test_to_array_with_ships() {
        let mut board = Board::new(BoardSize::Standard);
        // Complete 10x10 board: needs Carrier, Battleship, Cruiser, Submarine, Destroyer (1 each)
        board
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                0,
                0,
                Orientation::Horizontal,
            ))
            .unwrap();
        board
            .place_ship(Ship::new(ShipKind::Carrier, 2, 0, Orientation::Horizontal))
            .unwrap();
        board
            .place_ship(Ship::new(
                ShipKind::Battleship,
                4,
                0,
                Orientation::Horizontal,
            ))
            .unwrap();
        board
            .place_ship(Ship::new(ShipKind::Cruiser, 6, 0, Orientation::Horizontal))
            .unwrap();
        board
            .place_ship(Ship::new(
                ShipKind::Submarine,
                8,
                0,
                Orientation::Horizontal,
            ))
            .unwrap();

        let array = board.to_array().expect("Board should be ready");

        // Verify Destroyer at position 0, 1
        assert_eq!(array[0], ShipKind::Destroyer.id());
        assert_eq!(array[1], ShipKind::Destroyer.id());

        // Verify array has correct size
        assert_eq!(array.len(), 100); // 10x10
    }

    #[test]
    fn test_to_offset() {
        let board = Board::new(BoardSize::Standard); // 10x10

        assert_eq!(board.to_offset(0, 0), 0);
        assert_eq!(board.to_offset(0, 1), 1);
        assert_eq!(board.to_offset(0, 9), 9);
        assert_eq!(board.to_offset(1, 0), 10);
        assert_eq!(board.to_offset(1, 1), 11);
        assert_eq!(board.to_offset(9, 9), 99);
    }

    #[test]
    fn test_place_adjacent_ships_no_collision() {
        let mut board = Board::new(BoardSize::Standard);

        // Place ships adjacent but not overlapping
        let ship1 = Ship::new(ShipKind::Destroyer, 0, 0, Orientation::Horizontal); // (0,0) to (0,1)
        let ship2 = Ship::new(ShipKind::Destroyer, 1, 0, Orientation::Horizontal); // (1,0) to (1,1)

        assert!(board.place_ship(ship1).is_ok());
        // This should fail because we can only place 1 Destroyer on Standard board
        assert!(board.place_ship(ship2).is_err());
    }

    #[test]
    fn test_place_ships_at_board_edges() {
        let mut board = Board::new(BoardSize::Standard); // 10x10

        // Place ship at top-right corner
        let ship1 = Ship::new(ShipKind::Destroyer, 0, 8, Orientation::Horizontal);
        assert!(board.place_ship(ship1).is_ok());

        // Place ship at bottom-left corner
        let ship2 = Ship::new(ShipKind::Cruiser, 7, 0, Orientation::Vertical);
        assert!(board.place_ship(ship2).is_ok());
    }

    #[test]
    fn test_smaller_board_ship_limits() {
        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));

        // Only Destroyer and Cruiser allowed on smaller boards
        let destroyer = Ship::new(ShipKind::Destroyer, 0, 0, Orientation::Horizontal);
        let cruiser = Ship::new(ShipKind::Cruiser, 2, 2, Orientation::Horizontal);

        assert!(board.place_ship(destroyer).is_ok());
        assert!(board.place_ship(cruiser).is_ok());

        // Try to place another Destroyer (should fail - only 1 allowed)
        let destroyer2 = Ship::new(ShipKind::Destroyer, 4, 4, Orientation::Horizontal);
        assert!(board.place_ship(destroyer2).is_err());
    }

    // ===== Commitment Tests =====

    #[test]
    fn test_board_commitment_deterministic() {
        // Board 1
        let mut board1 = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board1
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                0,
                0,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");
        board1
            .place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        // Board 2 with identical setup
        let mut board2 = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board2
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                0,
                0,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");
        board2
            .place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        let salt = 12345u64;

        let commitment1 = board1.commit(salt).expect("Board should be ready");
        let commitment2 = board2.commit(salt).expect("Board should be ready");

        assert_eq!(
            commitment1, commitment2,
            "Same board and salt should produce same commitment"
        );
    }

    #[test]
    fn test_board_commitment_empty_board_fails() {
        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        let salt = 99999u64;

        let result = board.commit(salt);

        assert!(
            result.is_err(),
            "Empty board should not be ready for commitment"
        );
    }

    #[test]
    fn test_board_commitment_different_salt() {
        // Board 1 with salt 11111
        let mut board1 = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board1
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                0,
                0,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");
        board1
            .place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        // Board 2 with salt 22222 (same ship placement)
        let mut board2 = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board2
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                0,
                0,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");
        board2
            .place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        let commitment1 = board1.commit(11111).expect("Board should be ready");
        let commitment2 = board2.commit(22222).expect("Board should be ready");

        assert_ne!(
            commitment1, commitment2,
            "Different salts should produce different commitments"
        );
    }

    #[test]
    fn test_board_commitment_different_ships() {
        // Board 1: Complete board with Destroyer and Cruiser at one position
        let mut board1 = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board1
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                0,
                0,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");
        board1
            .place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        // Board 2: Complete board with Destroyer and Cruiser at different positions
        let mut board2 = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board2
            .place_ship(Ship::new(ShipKind::Cruiser, 0, 0, Orientation::Horizontal))
            .expect("Ship placement should succeed");
        board2
            .place_ship(Ship::new(ShipKind::Destroyer, 3, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        let salt = 12345u64;

        assert_ne!(
            board1.commit(salt).unwrap(),
            board2.commit(salt).unwrap(),
            "Different ship arrangements should produce different commitments"
        );
    }

    #[test]
    fn test_board_commitment_different_positions() {
        let salt = 12345u64;

        // Board 1: Complete board with ships at position 1
        let mut board1 = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board1
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                0,
                0,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");
        board1
            .place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        // Board 2: Complete board with ships at position 2
        let mut board2 = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board2
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                3,
                3,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");
        board2
            .place_ship(Ship::new(ShipKind::Cruiser, 0, 4, Orientation::Vertical))
            .expect("Ship placement should succeed");

        assert_ne!(
            board1.commit(salt).unwrap(),
            board2.commit(salt).unwrap(),
            "Same ship at different positions should produce different commitments"
        );
    }

    #[test]
    fn test_board_commitment_multiple_ships() {
        let salt = 12345u64;

        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                0,
                0,
                Orientation::Horizontal,
            ))
            .expect("Destroyer placement should succeed");
        board
            .place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Cruiser placement should succeed");

        let commitment = board.commit(salt).expect("Board should be ready");

        assert_ne!(
            commitment,
            Felt::ZERO,
            "Board with multiple ships should produce non-zero commitment"
        );
    }

    #[test]
    fn test_board_commitment_order_matters() {
        let salt = 12345u64;

        // Board 1: Destroyer first, then Cruiser
        let mut board1 = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board1
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                0,
                0,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");
        board1
            .place_ship(Ship::new(ShipKind::Cruiser, 2, 2, Orientation::Vertical))
            .expect("Ship placement should succeed");

        // Board 2: Cruiser first, then Destroyer
        let mut board2 = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board2
            .place_ship(Ship::new(ShipKind::Cruiser, 2, 2, Orientation::Vertical))
            .expect("Ship placement should succeed");
        board2
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                0,
                0,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");

        // Commitments should be the same (only final board state matters, not placement order)
        assert_eq!(
            board1.commit(salt).unwrap(),
            board2.commit(salt).unwrap(),
            "Placement order should not affect commitment (only final board state matters)"
        );
    }

    #[test]
    fn test_board_commitment_standard_size() {
        let mut board = Board::new(BoardSize::Standard);
        // Complete 10x10 board: needs Carrier, Battleship, Cruiser, Submarine, Destroyer (1 each)
        board
            .place_ship(Ship::new(ShipKind::Carrier, 0, 0, Orientation::Horizontal))
            .expect("Ship placement should succeed");
        board
            .place_ship(Ship::new(
                ShipKind::Battleship,
                2,
                0,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");
        board
            .place_ship(Ship::new(ShipKind::Cruiser, 4, 0, Orientation::Horizontal))
            .expect("Ship placement should succeed");
        board
            .place_ship(Ship::new(
                ShipKind::Submarine,
                6,
                0,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");
        board
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                8,
                0,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");

        let salt = 12345u64;
        let commitment = board.commit(salt).expect("Board should be ready");

        assert_ne!(
            commitment,
            Felt::ZERO,
            "Standard board should produce non-zero commitment"
        );
    }

    // ===== Fire and Hit Tests =====

    #[test]
    fn test_hit_ships_no_received_fire() {
        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                0,
                0,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");
        board
            .place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        board.commit(12345).expect("Board should be ready");

        let hits = board.hit_ships();

        assert_eq!(
            hits.len(),
            0,
            "No ships should be hit when no fire has been received"
        );
    }

    #[test]
    fn test_hit_ships_single_hit() {
        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        let destroyer = Ship::new(ShipKind::Destroyer, 0, 0, Orientation::Horizontal);
        let destroyer_id = destroyer.id;
        board
            .place_ship(destroyer)
            .expect("Ship placement should succeed");
        board
            .place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        board.commit(12345).expect("Board should be ready");

        // Hit the destroyer once at (0, 0)
        board.receive_fire(0, 0).expect("Fire should succeed");

        let hits = board.hit_ships();

        assert_eq!(hits.len(), 1, "One ship should be hit");
        assert_eq!(
            *hits.get(&destroyer_id).unwrap(),
            1,
            "Destroyer should have 1 hit"
        );
    }

    #[test]
    fn test_hit_ships_multiple_hits_same_ship() {
        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        let destroyer = Ship::new(ShipKind::Destroyer, 0, 0, Orientation::Horizontal);
        let destroyer_id = destroyer.id;
        board
            .place_ship(destroyer)
            .expect("Ship placement should succeed");
        board
            .place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        board.commit(12345).expect("Board should be ready");

        // Hit the destroyer twice at (0, 0) and (0, 1)
        board.receive_fire(0, 0).expect("Fire should succeed");
        board.receive_fire(0, 1).expect("Fire should succeed");

        let hits = board.hit_ships();

        assert_eq!(hits.len(), 1, "One ship should be hit");
        assert_eq!(
            *hits.get(&destroyer_id).unwrap(),
            2,
            "Destroyer should have 2 hits"
        );
    }

    #[test]
    fn test_hit_ships_multiple_ships() {
        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        let destroyer = Ship::new(ShipKind::Destroyer, 0, 0, Orientation::Horizontal);
        let cruiser = Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical);
        let destroyer_id = destroyer.id;
        let cruiser_id = cruiser.id;
        board
            .place_ship(destroyer)
            .expect("Ship placement should succeed");
        board
            .place_ship(cruiser)
            .expect("Ship placement should succeed");

        board.commit(12345).expect("Board should be ready");

        // Hit destroyer at (0, 0) and cruiser at (2, 1)
        board.receive_fire(0, 0).expect("Fire should succeed");
        board.receive_fire(2, 1).expect("Fire should succeed");

        let hits = board.hit_ships();

        assert_eq!(hits.len(), 2, "Two ships should be hit");
        assert_eq!(
            *hits.get(&destroyer_id).unwrap(),
            1,
            "Destroyer should have 1 hit"
        );
        assert_eq!(
            *hits.get(&cruiser_id).unwrap(),
            1,
            "Cruiser should have 1 hit"
        );
    }

    #[test]
    fn test_hit_ships_with_misses() {
        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        let destroyer = Ship::new(ShipKind::Destroyer, 0, 0, Orientation::Horizontal);
        let destroyer_id = destroyer.id;
        board
            .place_ship(destroyer)
            .expect("Ship placement should succeed");
        board
            .place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        board.commit(12345).expect("Board should be ready");

        // Hit water and ship
        board.receive_fire(5, 5).expect("Fire should succeed"); // Miss
        board.receive_fire(0, 0).expect("Fire should succeed"); // Hit

        let hits = board.hit_ships();

        assert_eq!(hits.len(), 1, "Only one ship should be hit");
        assert_eq!(
            *hits.get(&destroyer_id).unwrap(),
            1,
            "Destroyer should have 1 hit"
        );
    }

    #[test]
    fn test_receive_fire_miss() {
        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                0,
                0,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");
        board
            .place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        board.commit(12345).expect("Board should be ready");

        let report = board.receive_fire(5, 5).expect("Fire should succeed");

        assert_eq!(FireStatus::Miss, report.status, "Should be a miss");
        assert!(
            report.ship_destroyed.is_none(),
            "No ship should be destroyed"
        );
        assert!(!report.proof.is_empty(), "Proof should be provided");
    }

    #[test]
    fn test_receive_fire_hit() {
        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                0,
                0,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");
        board
            .place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        board.commit(12345).expect("Board should be ready");

        let report = board.receive_fire(0, 0).expect("Fire should succeed");

        assert_eq!(FireStatus::Hit(ShipKind::Destroyer), report.status, "Should be a hit");
        assert!(
            report.ship_destroyed.is_none(),
            "Ship should not be destroyed yet"
        );
        assert!(!report.proof.is_empty(), "Proof should be provided");
    }

    #[test]
    fn test_receive_fire_destroy_destroyer() {
        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        let destroyer = Ship::new(ShipKind::Destroyer, 0, 0, Orientation::Horizontal);
        board
            .place_ship(destroyer)
            .expect("Ship placement should succeed");
        board
            .place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        board.commit(12345).expect("Board should be ready");

        // Hit first cell
        let report1 = board.receive_fire(0, 0).expect("Fire should succeed");
        assert_eq!(FireStatus::Hit(ShipKind::Destroyer), report1.status, "First hit should succeed");
        assert!(report1.ship_destroyed.is_none(), "Ship not destroyed yet");

        // Hit second cell - should destroy destroyer (length 2)
        let report2 = board.receive_fire(0, 1).expect("Fire should succeed");
        assert_eq!(FireStatus::Hit(ShipKind::Destroyer), report2.status, "Second hit should succeed");
        assert!(report2.ship_destroyed.is_some(), "Ship should be destroyed");
        assert_eq!(
            report2.ship_destroyed.unwrap().kind,
            ShipKind::Destroyer,
            "Destroyed ship should be Destroyer"
        );
    }

    #[test]
    fn test_receive_fire_destroy_cruiser() {
        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                0,
                0,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");
        let cruiser = Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical);
        board
            .place_ship(cruiser)
            .expect("Ship placement should succeed");

        board.commit(12345).expect("Board should be ready");

        // Hit cruiser 3 times (length 3)
        board.receive_fire(2, 1).expect("Fire should succeed");
        board.receive_fire(3, 1).expect("Fire should succeed");
        let report = board.receive_fire(4, 1).expect("Fire should succeed");

        assert_eq!(FireStatus::Hit(ShipKind::Cruiser), report.status, "Should be a hit");
        assert!(report.ship_destroyed.is_some(), "Ship should be destroyed");
        assert_eq!(
            report.ship_destroyed.unwrap().kind,
            ShipKind::Cruiser,
            "Destroyed ship should be Cruiser"
        );
    }

    #[test]
    fn test_receive_fire_already_bombed() {
        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                0,
                0,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");
        board
            .place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        board.commit(12345).expect("Board should be ready");

        // First fire should succeed
        board.receive_fire(0, 0).expect("First fire should succeed");

        // Second fire at same location should fail
        let result = board.receive_fire(0, 0);
        assert!(
            result.is_err(),
            "Should not be able to bomb same location twice"
        );
    }

    #[test]
    fn test_receive_fire_without_commitment() {
        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                0,
                0,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");
        board
            .place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        // Don't commit - board not ready

        let result = board.receive_fire(0, 0);
        assert!(
            result.is_err(),
            "Should not be able to fire without commitment"
        );
    }

    #[test]
    fn test_receive_fire_vertical_ship() {
        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                0,
                0,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");
        board
            .place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        board.commit(12345).expect("Board should be ready");

        // Hit vertical cruiser at different x coordinates
        let report1 = board.receive_fire(2, 1).expect("Fire should succeed");
        assert_eq!(FireStatus::Hit(ShipKind::Cruiser), report1.status, "Should be a hit");

        let report2 = board.receive_fire(3, 1).expect("Fire should succeed");
        assert_eq!(FireStatus::Hit(ShipKind::Cruiser), report2.status, "Should be a hit");

        let report3 = board.receive_fire(4, 1).expect("Fire should succeed");
        assert_eq!(FireStatus::Hit(ShipKind::Cruiser), report3.status, "Should be a hit");
        assert!(
            report3.ship_destroyed.is_some(),
            "Cruiser should be destroyed"
        );
    }

    #[test]
    fn test_receive_fire_multiple_ships_destroyed() {
        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        let destroyer = Ship::new(ShipKind::Destroyer, 0, 0, Orientation::Horizontal);
        let cruiser = Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical);
        board
            .place_ship(destroyer)
            .expect("Ship placement should succeed");
        board
            .place_ship(cruiser)
            .expect("Ship placement should succeed");

        board.commit(12345).expect("Board should be ready");

        // Destroy destroyer
        board.receive_fire(0, 0).expect("Fire should succeed");
        let destroyer_destroyed = board.receive_fire(0, 1).expect("Fire should succeed");
        assert!(
            destroyer_destroyed.ship_destroyed.is_some(),
            "Destroyer should be destroyed"
        );
        assert_eq!(
            destroyer_destroyed.ship_destroyed.unwrap().kind,
            ShipKind::Destroyer
        );

        // Destroy cruiser
        board.receive_fire(2, 1).expect("Fire should succeed");
        board.receive_fire(3, 1).expect("Fire should succeed");
        let cruiser_destroyed = board.receive_fire(4, 1).expect("Fire should succeed");
        assert!(
            cruiser_destroyed.ship_destroyed.is_some(),
            "Cruiser should be destroyed"
        );
        assert_eq!(
            cruiser_destroyed.ship_destroyed.unwrap().kind,
            ShipKind::Cruiser
        );
    }

    #[test]
    fn test_receive_fire_proof_always_present() {
        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                0,
                0,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");
        board
            .place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        board.commit(12345).expect("Board should be ready");

        // Test hit
        let hit_report = board.receive_fire(0, 0).expect("Fire should succeed");
        assert!(!hit_report.proof.is_empty(), "Hit should include proof");

        // Test miss
        let miss_report = board.receive_fire(5, 5).expect("Fire should succeed");
        assert!(!miss_report.proof.is_empty(), "Miss should include proof");

        // Test destruction
        let destroy_report = board.receive_fire(0, 1).expect("Fire should succeed");
        assert!(
            !destroy_report.proof.is_empty(),
            "Destruction should include proof"
        );
    }

    #[test]
    fn test_receive_fire_game_over() {
        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                0,
                0,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");
        board
            .place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        board.commit(12345).expect("Board should be ready");

        // Destroy all ships
        // Destroyer: (0,0) and (0,1)
        board.receive_fire(0, 0).expect("Fire should succeed");
        board.receive_fire(0, 1).expect("Fire should succeed");

        // Cruiser: (2,1), (3,1), (4,1)
        board.receive_fire(2, 1).expect("Fire should succeed");
        board.receive_fire(3, 1).expect("Fire should succeed");
        board.receive_fire(4, 1).expect("Fire should succeed");

        // Try to fire again - should return GameOver error
        let result = board.receive_fire(5, 5);
        assert!(result.is_err(), "Should return error when game is over");
        assert!(
            matches!(result.unwrap_err(), GameOver),
            "Error should be GameOver"
        );
    }

    #[test]
    fn test_place_ship_after_board_ready() {
        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                0,
                0,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");
        board
            .place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        // Board is now ready (all required ships placed)
        assert!(board.is_board_ready(), "Board should be ready");

        // Try to place another ship
        let result = board.place_ship(Ship::new(ShipKind::Destroyer, 4, 4, Orientation::Horizontal));
        assert!(result.is_err(), "Should not be able to place ship when board is ready");
        assert!(
            matches!(result.unwrap_err(), AllShipsPlaced),
            "Error should be AllShipsPlaced"
        );
    }

    #[test]
    fn test_place_ship_after_commitment() {
        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                0,
                0,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");
        board
            .place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        // Commit the board
        board.commit(12345).expect("Commit should succeed");

        // Try to place another ship after commitment
        let result = board.place_ship(Ship::new(ShipKind::Destroyer, 4, 4, Orientation::Horizontal));
        assert!(
            result.is_err(),
            "Should not be able to place ship after commitment"
        );
        println!("{:?}", result.clone().unwrap_err());
        assert!(
            matches!(result.unwrap_err(), BoardAlreadyCommitted),
            "Error should be BoardAlreadyCommitted"
        );
    }

    #[test]
    fn test_commit_twice() {
        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                0,
                0,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");
        board
            .place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        // First commit should succeed
        let commitment1 = board.commit(12345).expect("First commit should succeed");
        assert_ne!(commitment1, Felt::ZERO, "Commitment should be non-zero");

        // Second commit should fail
        let result = board.commit(67890);
        assert!(result.is_err(), "Should not be able to commit twice");
        assert!(
            matches!(result.unwrap_err(), BoardAlreadyCommitted),
            "Error should be BoardAlreadyCommitted"
        );
    }

    #[test]
    fn test_commit_incomplete_board() {
        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        // Only place one ship (need Destroyer + Cruiser for 6x6)
        board
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                0,
                0,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");

        // Try to commit incomplete board
        let result = board.commit(12345);
        assert!(result.is_err(), "Should not be able to commit incomplete board");
        assert!(
            matches!(result.unwrap_err(), BoardNotReady),
            "Error should be BoardNotReady"
        );
    }

    #[test]
    fn test_receive_fire_game_over_with_partial_hits() {
        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                0,
                0,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");
        board
            .place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        board.commit(12345).expect("Board should be ready");

        // Partially hit destroyer (only 1 of 2 cells)
        board.receive_fire(0, 0).expect("Fire should succeed");

        // Game should not be over yet - can still fire
        let result = board.receive_fire(5, 5);
        assert!(result.is_ok(), "Should be able to fire when game not over");
    }

    #[test]
    fn test_place_ship_validates_ship_kind_count() {
        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));

        // Place Destroyer - should succeed
        board
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                0,
                0,
                Orientation::Horizontal,
            ))
            .expect("First Destroyer placement should succeed");

        // Try to place another Destroyer - should fail (6x6 only allows 1)
        let result = board.place_ship(Ship::new(
            ShipKind::Destroyer,
            2,
            2,
            Orientation::Horizontal,
        ));
        assert!(result.is_err(), "Should not allow more Destroyers than permitted");
        assert!(
            matches!(result.unwrap_err(), InvalidShipPlacementKind { .. }),
            "Error should be InvalidShipPlacementKind"
        );
    }

    #[test]
    fn test_place_ship_validates_ineligible_ship_kind() {
        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));

        // Try to place Carrier on 6x6 board (not allowed)
        let result = board.place_ship(Ship::new(
            ShipKind::Carrier,
            0,
            0,
            Orientation::Horizontal,
        ));
        assert!(result.is_err(), "Should not allow ineligible ship kinds");
        assert!(
            matches!(result.unwrap_err(), InvalidShipPlacementKind { .. }),
            "Error should be InvalidShipPlacementKind"
        );
    }

    #[test]
    fn test_commit_preserves_board_state() {
        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                0,
                0,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");
        board
            .place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        // Get cells before commit
        let cells_before = board.cells();

        // Commit
        board.commit(12345).expect("Commit should succeed");

        // Get cells after commit - should be identical
        let cells_after = board.cells();

        assert_eq!(
            cells_before.len(),
            cells_after.len(),
            "Cell count should not change after commit"
        );

        for (i, (before, after)) in cells_before.iter().zip(cells_after.iter()).enumerate() {
            assert_eq!(
                before, after,
                "Cell {} should be identical before and after commit",
                i
            );
        }
    }

    #[test]
    fn test_receive_fire_sequence_tracking() {
        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                0,
                0,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");
        board
            .place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        board.commit(12345).expect("Board should be ready");

        // Fire multiple times and verify hits are tracked correctly
        board.receive_fire(0, 0).expect("Fire 1 should succeed"); // Hit destroyer
        board.receive_fire(5, 5).expect("Fire 2 should succeed"); // Miss
        board.receive_fire(2, 1).expect("Fire 3 should succeed"); // Hit cruiser

        let hits = board.hit_ships();

        // Should have 2 ships hit (Destroyer and Cruiser), each with 1 hit
        assert_eq!(hits.len(), 2, "Should have 2 ships with hits");
    }

    #[test]
    fn test_to_array_after_commit() {
        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                0,
                0,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");
        board
            .place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        // Get array before commit
        let array_before = board.to_array().expect("Should get array");

        // Commit
        board.commit(12345).expect("Commit should succeed");

        // Get array after commit - should be identical
        let array_after = board.to_array().expect("Should get array after commit");

        assert_eq!(
            array_before, array_after,
            "Array should be identical before and after commit"
        );
    }
}
