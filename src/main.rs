#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

use cozy_chess::Board;
use polyglot::{Polyglot, zobrist::Zobrist};

fn main() {
    let polyglot = Polyglot::load_book("book.bin").unwrap();

    let board = Board::startpos();

    for entry in polyglot.get_entries(dbg!(Zobrist::compute(&board))) {
        dbg!(entry);
        dbg!(entry.decode_move().unwrap().to_cozy());
    }
}
