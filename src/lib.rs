#![deny(clippy::all)]

use std::{
    fs::File,
    io::{self, Read},
};

mod tests;

#[cfg(feature = "cozy-chess")]
pub mod zobrist;

mod zobrist_keys;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Promotion {
    None = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecodedMove {
    pub from_file: u8,
    pub from_rank: u8,
    pub to_file: u8,
    pub to_rank: u8,
    pub promotion: Promotion,
}

#[cfg(feature = "cozy-chess")]
impl DecodedMove {
    /// Converts the `DecodedMove` into a `cozy_chess::Move`.
    /// Does not check legality.
    pub const fn to_cozy(&self) -> cozy_chess::Move {
        use cozy_chess::{Move, Piece, Square};

        let from = Square::new(
            cozy_chess::File::ALL[self.from_file as usize],
            cozy_chess::Rank::ALL[self.from_rank as usize],
        );
        let to = Square::new(
            cozy_chess::File::ALL[self.to_file as usize],
            cozy_chess::Rank::ALL[self.to_rank as usize],
        );

        let promotion = match self.promotion {
            Promotion::None => None,
            Promotion::Knight => Some(Piece::Knight),
            Promotion::Bishop => Some(Piece::Bishop),
            Promotion::Rook => Some(Piece::Rook),
            Promotion::Queen => Some(Piece::Queen),
        };

        Move {
            from,
            to,
            promotion,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Entry {
    pub key: u64,
    pub book_move: u16,
    pub weight: u16,
    pub learn: u32,
}

impl Entry {
    #[must_use]
    pub fn decode_move(&self) -> Option<DecodedMove> {
        if self.book_move == 0 {
            return None;
        }

        let to_file = (self.book_move & 0x7) as u8; // Bits 0-2
        let to_row = ((self.book_move >> 3) & 0x7) as u8; // Bits 3-5
        let from_file = ((self.book_move >> 6) & 0x7) as u8; // Bits 6-8
        let from_row = ((self.book_move >> 9) & 0x7) as u8; // Bits 9-11
        let promotion_code = ((self.book_move >> 12) & 0x7) as u8; // Bits 12-14

        let promotion = match promotion_code {
            0 => Promotion::None,
            1 => Promotion::Knight,
            2 => Promotion::Bishop,
            3 => Promotion::Rook,
            4 => Promotion::Queen,
            _ => panic!("Invalid promotion"),
        };

        Some(DecodedMove {
            from_file,
            from_rank: from_row,
            to_file,
            to_rank: to_row,
            promotion,
        })
    }
}

pub struct Polyglot {
    book: Vec<Entry>,
}

impl Polyglot {
    /// Load the Polyglot opening book from a file
    pub fn load_book(file_path: &str) -> io::Result<Self> {
        let mut book = Vec::new();
        let mut prev_key: Option<u64> = None;

        let mut file = File::open(file_path)?;
        let mut buffer = [0u8; 16]; // 16 bytes per entry

        while file.read_exact(&mut buffer).is_ok() {
            let key = u64::from_be_bytes(buffer[0..8].try_into().unwrap());
            let book_move = u16::from_be_bytes(buffer[8..10].try_into().unwrap());
            let weight = u16::from_be_bytes(buffer[10..12].try_into().unwrap());
            let learn = u32::from_be_bytes(buffer[12..16].try_into().unwrap());

            if let Some(prev_key) = prev_key {
                if key < prev_key {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Book entries are not sorted by key",
                    ));
                }
            }

            let entry = Entry {
                key,
                book_move,
                weight,
                learn,
            };
            book.push(entry);

            prev_key = Some(key);
        }

        Ok(Self { book })
    }

    /// Probe the Polyglot book for a move based on the Zobrist key
    #[must_use]
    pub fn get_entries(&self, zobrist_key: u64) -> &[Entry] {
        let start = self.book.partition_point(|e| e.key < zobrist_key);
        let end = self.book.partition_point(|e| e.key <= zobrist_key);

        if start < end {
            &self.book[start..end]
        } else {
            &[]
        }
    }
}
