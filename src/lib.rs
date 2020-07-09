pub mod keys;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Side {
    White,
    Black
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King
}

impl PieceType {
    pub fn index(self) -> usize {
        match self {
            PieceType::Pawn => 0,
            PieceType::Knight => 1,
            PieceType::Bishop => 2,
            PieceType::Rook => 3,
            PieceType::Queen => 4,
            PieceType::King => 5
        }
    }
}

#[derive(Debug)]
pub struct Piece {
    pub piece_type: PieceType,
    pub side: Side,
    pub rank: usize,
    pub file: usize
}

impl Piece {
    pub fn polyglot_hash(&self) -> u64 {
        let kind = self.piece_type.index() * 2 + (self.side == Side::White) as usize;
        keys::RANDOM_PIECE[64 * kind + 8 * self.rank + self.file]
    }
}

#[derive(Debug)]
pub struct CastleRights {
    queen_side: bool,
    king_side: bool
}

impl CastleRights {
    pub fn polyglot_hash(&self, side: Side) -> u64 {
        let mut hash = 0;
        let base = if side == Side::White {
            0
        } else {
            2
        };
        if self.king_side {
            hash ^= keys::RANDOM_CASTLE[base];
        }
        if self.queen_side {
            hash ^= keys::RANDOM_CASTLE[base + 1];
        }
        hash
    }
}

#[derive(Debug)]
struct PolyglotKey<'a> {
    pieces: &'a [Piece],
    white_castle: CastleRights,
    black_castle: CastleRights,
    en_passant_file: Option<usize>,
    turn: Side
}

impl PolyglotKey<'_> {
    pub fn polyglot_hash(&self) -> u64 {
        let mut hash = 0;
        for piece in self.pieces {
            hash ^= piece.polyglot_hash();
        }
        hash ^= self.white_castle.polyglot_hash(Side::White);
        hash ^= self.black_castle.polyglot_hash(Side::Black);
        if let Some(file) = self.en_passant_file {
            hash ^= keys::RANDOM_EN_PASSANT[file];
        }
        if self.turn == Side::White {
            hash ^= keys::RANDOM_TURN;
        }
        hash
    }
}

struct PolyglotEntry {

}

struct PolyglotReader {

}

impl PolyglotReader {
    pub fn get(&self, key: &PolyglotKey) {

    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use std::str::FromStr;

    fn key(board: &chess::Board, f: impl FnOnce(&PolyglotKey) -> ()) {
        let pieces: Vec<_> = board.combined().into_iter().map(|sq| {
            let piece_type = match board.piece_on(sq).unwrap() {
                chess::Piece::Pawn => PieceType::Pawn,
                chess::Piece::Knight => PieceType::Knight,
                chess::Piece::Bishop => PieceType::Bishop,
                chess::Piece::Rook => PieceType::Rook,
                chess::Piece::Queen => PieceType::Queen,
                chess::Piece::King => PieceType::King
            };
            Piece {
                piece_type,
                rank: sq.get_rank().to_index(),
                file: sq.get_file().to_index(),
                side: if board.color_on(sq).unwrap() == chess::Color::White {
                    Side::White
                } else {
                    Side::Black
                }
            }
        }).collect();

        let white_castle = board.castle_rights(chess::Color::White);
        let black_castle = board.castle_rights(chess::Color::Black);

        let key = PolyglotKey {
            pieces: &pieces,
            white_castle: CastleRights {
                queen_side: white_castle.has_queenside(),
                king_side: white_castle.has_kingside()
            },
            black_castle: CastleRights {
                queen_side: black_castle.has_queenside(),
                king_side: black_castle.has_kingside()
            },
            en_passant_file: board.en_passant().and_then(|en_passant_sq| {
                [en_passant_sq.left(), en_passant_sq.right()]
                    .iter()
                    .flatten()
                    .find_map(|&sq| {
                        if board.piece_on(sq) == Some(chess::Piece::Pawn) &&
                            board.color_on(sq).unwrap() == board.side_to_move() {
                            Some(en_passant_sq.get_file().to_index())
                        } else {
                            None
                        }
                    })
            }),
            turn: if board.side_to_move() == chess::Color::White {
                Side::White
            } else {
                Side::Black
            }
        };
        f(&key);
    }

    #[test]
    fn test_keys() {
        const TESTS: &[(&str, u64)] = &[
            ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 0x463b96181691fc9c),
            ("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1", 0x823c9b50fd114196),
            ("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2", 0x0756b94461c50fb0),
            ("rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 2", 0x662fafb965db29d4),
            ("rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3", 0x22a48b5a8e47ff78),
            ("rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPPKPPP/RNBQ1BNR b kq - 0 3", 0x652a607ca3f242c1),
            ("rnbq1bnr/ppp1pkpp/8/3pPp2/8/8/PPPPKPPP/RNBQ1BNR w - - 0 4", 0x00fdd303c946bdd9),
            ("rnbqkbnr/p1pppppp/8/8/PpP4P/8/1P1PPPP1/RNBQKBNR b KQkq c3 0 3", 0x3c8123ea7b067637),
            ("rnbqkbnr/p1pppppp/8/8/P6P/R1p5/1P1PPPP1/1NBQKBNR b Kkq - 0 4", 0x5c3f9b829b279560)
        ];

        for (i, &(fen, expected)) in TESTS.iter().enumerate() {
            let board = chess::Board::from_str(fen).unwrap();
            key(&board, |k|  {
                assert_eq!(k.polyglot_hash(), expected, "Testing hash for '{}' (Test {})", fen, i + 1);
            });
        }
    }
}
