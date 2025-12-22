use super::orientation::Orientation;
use super::ship::Ship;
use super::result::Result;
use crate::types::{Cell, ShipKind};
use std::fmt;
use std::fmt::format;
use crate::types::board_size::BoardSize;
use crate::types::error::GameError;
use crate::types::error::GameError::{InvalidShipPlacementBounds, InvalidShipPlacementCollides, InvalidShipPlacementKind};

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

    /// Gets the cell at the given coordinates
    pub fn get(&self, x: usize, y: usize) -> Result<Cell> {
        let size = self.size.size();
        if (x >= size || y >= size) {
            Err(GameError::OutOfBoardBounds { x, y })
        } else {
            let index = self.to_offset(x, y);
            Ok(self.cells[index])
        }
    }

    /// Sets the cell at the given coordinates
    pub fn set(&mut self, x: usize, y: usize, cell: Cell) -> Result<()> {
        let size = self.size.size();
        if (x >= size || y >= size) {
            Err(GameError::OutOfBoardBounds { x, y })
        } else {
            let index = self.to_offset(x, y);
            self.cells[index] = cell;
            Ok(())
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
                if ship.x + length > board_size {
                    return Err(InvalidShipPlacementBounds { ship: ship.kind, x: ship.x, y: ship.y, orientation: ship.orientation });
                }
            }
            Orientation::Vertical => {
                if ship.y + length > board_size {
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
