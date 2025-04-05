use crate::zobrist_keys::ZOBRIST_KEYS;
use cozy_chess::{Board, Color, Piece, Rank, Square};

enum ColoredPiece {
    BlackPawn = 0,
    WhitePawn = 1,
    BlackKnight = 2,
    WhiteKnight = 3,
    BlackBishop = 4,
    WhiteBishop = 5,
    BlackRook = 6,
    WhiteRook = 7,
    BlackQueen = 8,
    WhiteQueen = 9,
    BlackKing = 10,
    WhiteKing = 11,
}

pub struct Zobrist;
impl Zobrist {
    const fn piece_hash(piece: ColoredPiece, square: Square) -> u64 {
        ZOBRIST_KEYS[64 * (piece as usize) + square as usize]
    }

    fn castle_hash(board: &Board) -> u64 {
        let castle_rights = [
            board.castle_rights(Color::White).short.is_some(),
            board.castle_rights(Color::White).long.is_some(),
            board.castle_rights(Color::Black).short.is_some(),
            board.castle_rights(Color::Black).long.is_some(),
        ];

        castle_rights
            .iter()
            .enumerate()
            .fold(0, |acc, (i, &right)| {
                if right {
                    acc ^ ZOBRIST_KEYS[768 + i]
                } else {
                    acc
                }
            })
    }

    fn en_passant_hash(board: &Board) -> u64 {
        if let Some(file) = board.en_passant() {
            let side_to_move = board.side_to_move();

            let opponent_pawn_rank = match side_to_move {
                Color::White => Rank::Fifth,
                Color::Black => Rank::Fourth,
            };

            let adjacent_files = file.adjacent();

            let player_pawns = board.colored_pieces(side_to_move, Piece::Pawn);

            if !adjacent_files.is_disjoint(player_pawns & opponent_pawn_rank.bitboard()) {
                return ZOBRIST_KEYS[772 + file as usize];
            }
        }
        0
    }

    fn turn_hash(is_white_to_move: bool) -> u64 {
        if is_white_to_move {
            ZOBRIST_KEYS[780]
        } else {
            0
        }
    }

    #[must_use]
    pub fn compute(board: &Board) -> u64 {
        let mut piece_hash = 0;
        for color in [Color::White, Color::Black] {
            for piece in Piece::ALL {
                for square in board.colored_pieces(color, piece) {
                    let colored_piece = match (color, piece) {
                        (Color::White, Piece::Pawn) => ColoredPiece::WhitePawn,
                        (Color::White, Piece::Knight) => ColoredPiece::WhiteKnight,
                        (Color::White, Piece::Bishop) => ColoredPiece::WhiteBishop,
                        (Color::White, Piece::Rook) => ColoredPiece::WhiteRook,
                        (Color::White, Piece::Queen) => ColoredPiece::WhiteQueen,
                        (Color::White, Piece::King) => ColoredPiece::WhiteKing,
                        (Color::Black, Piece::Pawn) => ColoredPiece::BlackPawn,
                        (Color::Black, Piece::Knight) => ColoredPiece::BlackKnight,
                        (Color::Black, Piece::Bishop) => ColoredPiece::BlackBishop,
                        (Color::Black, Piece::Rook) => ColoredPiece::BlackRook,
                        (Color::Black, Piece::Queen) => ColoredPiece::BlackQueen,
                        (Color::Black, Piece::King) => ColoredPiece::BlackKing,
                    };
                    piece_hash ^= Self::piece_hash(colored_piece, square);
                }
            }
        }

        let castle_hash = Self::castle_hash(board);
        let en_passant_hash = Self::en_passant_hash(board);
        let turn_hash = Self::turn_hash(board.side_to_move() == Color::White);

        piece_hash ^ castle_hash ^ en_passant_hash ^ turn_hash
    }
}
