use super::orientation::Orientation;
use super::result::Result;
use super::ship::Ship;
use crate::types::board_size::BoardSize;
use crate::types::error::GameError::{InvalidShipPlacementBounds, InvalidShipPlacementCollides, InvalidShipPlacementKind};
use crate::types::{Cell, ShipKind};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    size: BoardSize,
    /// Cells stored in a single vector per rows
    cells: Vec<Cell>,

    ships_placed: Vec<ShipKind>,
}

impl Board {
    pub fn new (size: BoardSize) -> Board {
        Board {
            size,
            cells: vec![Cell::Water; size.size() * size.size()],
            ships_placed: Vec::new(),
        }
    }

    /// Places a ship on the board
    pub fn place_ship(&mut self, ship: Ship) -> Result<()> {
        let counts = self.size.ship_kinds_count(&ship.kind);

        let mut ship_kind_occurrences = 0;
        self.ships_placed.iter().for_each(|kind| {
            if kind == &ship.kind {
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
                    return Err(InvalidShipPlacementBounds { ship: ship.kind, x: ship.x, y: ship.y, orientation: ship.orientation });
                }
            }
            Orientation::Vertical => {
                if ship.x + length > board_size {
                    return Err(InvalidShipPlacementBounds { ship: ship.kind, x: ship.x, y: ship.y, orientation: ship.orientation });
                }
            }
        }

        // Check for overlaps and place ship
        for i in 0..length {
            let (x, y) = match ship.orientation {
                Orientation::Horizontal => (ship.x, ship.y + i),
                Orientation::Vertical => (ship.x + i, ship.y),
            };

            let offset = self.to_offset(x, y);
            if self.cells[offset] != Cell::Water {
                return Err(InvalidShipPlacementCollides{
                    ship: ship.kind,
                    x: ship.x,
                    y: ship.y,
                    orientation: Orientation::Horizontal,
                    xc: x,
                    yc: y,
                });
            }
        }

        // Place the ship
        for i in 0..length {
            let (x, y) = match ship.orientation {
                Orientation::Horizontal => (ship.x, ship.y + i),
                Orientation::Vertical => (ship.x + i, ship.y),
            };
            let offset = self.to_offset(x, y);
            self.cells[offset] = Cell::Ship(ship.kind);
        }
        self.ships_placed.push(ship.kind);

        Ok(())
    }

    /// Converts the board to a flat vec of u8 (for hashing)
    pub fn to_array(&self) -> Vec<u8> {
        let size = self.size.size();
        let mut arr = Vec::with_capacity(size * size);

        for i in 0..size * size {
            arr.push(match self.cells[i] {
                Cell::Water => 0,
                Cell::Ship(kind) => kind.id()
            })
        }

        arr
    }

    fn to_offset(&self, x: usize, y: usize) -> usize {
        let size = self.size.size();
        let rows_offset = x * size;
        rows_offset + y
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new(BoardSize::default())
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let size = self.size.size();
        let divider_items = (0..size).map(|_| "----").collect::<Vec<_>>();
        let row = | items: &[&str], index: Option<usize> | -> String {
            let sanitize = | text: &str | {
                if text.len() != 2 && text.len() != 4 {
                    panic!("Each text on grid should be 2 or 4 chars. Was `{}`.", &text);
                }

                if text.len() == 2 {
                    return format!("  {}  ", text);
                }

                return format!(" {} ", text);
            };

            if items.len() > size {
                panic!("Items to draw should be at most {}", size)
            }

            let mut row = String::from("|");
            match index {
                None => {
                    row.push_str("      |");
                }
                Some(index) => {
                    let index = index + 1;
                    let index_formatted = if index < 10 {
                        format!("0{index}")
                    } else {
                        index.to_string()
                    };

                    row = format!("{row}{}|", sanitize(index_formatted.as_str()));
                }
            }
            for i in 0..size {
                row = format!("{}{}|", row, sanitize(items[i]));
            }
            return row;
        };

        let column_titles = (1..size + 1).map(|i| {
            if i < 10 { format!("0{i}") } else { format!("{i}") }
        }).collect::<Vec<_>>();

        writeln!(f, "{}", row(divider_items.as_slice(), None))?;
        writeln!(f, "{}", row(column_titles.iter().map(|i| i.as_str()).collect::<Vec<_>>().as_slice(), None))?;
        let rows = self.cells.chunks(size).collect::<Vec<_>>();
        for (index, cells) in rows.iter().enumerate() {
            let cells_formatted = cells.into_iter().map(|cell| {
                match cell {
                    Cell::Water => "~~".to_string(),
                    Cell::Ship(ship) => format!("{}", ship.code()).to_string(),
                }
            }).collect::<Vec<_>>();

            writeln!(f, "{}", row(
                cells_formatted.iter().map(|cell| cell.as_str()).collect::<Vec<_>>().as_slice(),
                Some(index)
            ))?;
        }
        writeln!(f, "{}", row(divider_items.as_slice(), None))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::board_size::{BoardSize, LargerBoardSize, SmallerBoardSize};
    use crate::types::error::GameError;

    #[test]
    fn test_new_board_standard() {
        let board = Board::new(BoardSize::Standard);
        assert_eq!(board.size, BoardSize::Standard);
        assert_eq!(board.cells.len(), 100); // 10x10
        assert!(board.cells.iter().all(|c| c == &Cell::Water));
        assert_eq!(board.ships_placed.len(), 0);
    }

    #[test]
    fn test_new_board_smaller() {
        let board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        assert_eq!(board.cells.len(), 36); // 6x6
        assert!(board.cells.iter().all(|c| c == &Cell::Water));
    }

    #[test]
    fn test_new_board_larger() {
        let board = Board::new(BoardSize::Larger(LargerBoardSize::TwelveByTwelve));
        assert_eq!(board.cells.len(), 144); // 12x12
        assert!(board.cells.iter().all(|c| c == &Cell::Water));
    }

    #[test]
    fn test_default_board() {
        let board = Board::default();
        assert_eq!(board.size, BoardSize::Standard);
        assert_eq!(board.cells.len(), 100);
    }

    #[test]
    fn test_place_ship_horizontal() {
        let mut board = Board::new(BoardSize::Standard);
        let ship = Ship::new(ShipKind::Destroyer, 2, 3, Orientation::Horizontal);

        assert!(board.place_ship(ship).is_ok());

        // Verify ship is placed correctly (Destroyer has length 2)
        assert_eq!(board.cells[board.to_offset(2, 3)], Cell::Ship(ShipKind::Destroyer));
        assert_eq!(board.cells[board.to_offset(2, 4)], Cell::Ship(ShipKind::Destroyer));
        assert_eq!(board.ships_placed.len(), 1);
        assert_eq!(board.ships_placed[0], ShipKind::Destroyer);
    }

    #[test]
    fn test_place_ship_vertical() {
        let mut board = Board::new(BoardSize::Standard);
        let ship = Ship::new(ShipKind::Cruiser, 1, 1, Orientation::Vertical);

        assert!(board.place_ship(ship).is_ok());

        // Verify ship is placed correctly (Cruiser has length 3)
        assert_eq!(board.cells[board.to_offset(1, 1)], Cell::Ship(ShipKind::Cruiser));
        assert_eq!(board.cells[board.to_offset(2, 1)], Cell::Ship(ShipKind::Cruiser));
        assert_eq!(board.cells[board.to_offset(3, 1)], Cell::Ship(ShipKind::Cruiser));
    }

    #[test]
    fn test_place_ship_out_of_bounds_horizontal() {
        let mut board = Board::new(BoardSize::Standard); // 10x10
        let ship = Ship::new(ShipKind::Carrier, 0, 9, Orientation::Horizontal); // Carrier length 5, y=9 + 5 > 10

        let result = board.place_ship(ship);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GameError::InvalidShipPlacementBounds { .. }));
    }

    #[test]
    fn test_place_ship_out_of_bounds_vertical() {
        let mut board = Board::new(BoardSize::Standard); // 10x10
        let ship = Ship::new(ShipKind::Battleship, 8, 0, Orientation::Vertical); // Battleship length 4, x=8 + 4 > 10

        let result = board.place_ship(ship);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GameError::InvalidShipPlacementBounds { .. }));
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
        assert!(matches!(result.unwrap_err(), GameError::InvalidShipPlacementCollides { .. }));
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
        assert!(matches!(result.unwrap_err(), GameError::InvalidShipPlacementCollides { .. }));
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
        assert!(matches!(result.unwrap_err(), GameError::InvalidShipPlacementKind { .. }));
    }

    #[test]
    fn test_place_multiple_ships_larger_board() {
        let mut board = Board::new(BoardSize::Larger(LargerBoardSize::TwelveByTwelve));

        // Larger boards allow 2 Destroyers
        let ship1 = Ship::new(ShipKind::Destroyer, 0, 0, Orientation::Horizontal);
        let ship2 = Ship::new(ShipKind::Destroyer, 5, 5, Orientation::Horizontal);

        assert!(board.place_ship(ship1).is_ok());
        assert!(board.place_ship(ship2).is_ok());
        assert_eq!(board.ships_placed.len(), 2);

        // Third Destroyer should fail
        let ship3 = Ship::new(ShipKind::Destroyer, 8, 8, Orientation::Horizontal);
        let result = board.place_ship(ship3);
        assert!(result.is_err());
    }

    #[test]
    fn test_to_array_empty_board() {
        let board = Board::new(BoardSize::Standard);
        let array = board.to_array();

        assert_eq!(array.len(), 100);
        assert!(array.iter().all(|&v| v == 0)); // All water cells
    }

    #[test]
    fn test_to_array_with_ships() {
        let mut board = Board::new(BoardSize::Standard);
        let ship = Ship::new(ShipKind::Destroyer, 0, 0, Orientation::Horizontal);
        board.place_ship(ship).unwrap();

        let array = board.to_array();

        assert_eq!(array[0], ShipKind::Destroyer.id());
        assert_eq!(array[1], ShipKind::Destroyer.id());
        assert!(array[2..].iter().all(|&v| v == 0));
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
}
