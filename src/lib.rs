use std::io::{Read,Seek,SeekFrom};

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
    pub queen_side: bool,
    pub king_side: bool
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
struct PolyglotKey {
    pub pieces: Vec<Piece>,
    pub white_castle: CastleRights,
    pub black_castle: CastleRights,
    pub en_passant_file: Option<usize>,
    pub turn: Side
}

impl PolyglotKey {
    pub fn polyglot_hash(&self) -> u64 {
        let mut hash = 0;
        for piece in &self.pieces {
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
    #[cfg(feature = "chess_lib_helpers")]
    pub fn from_board(board: &chess::Board) -> Self {
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

        Self {
            pieces,
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
        }
    }
}

#[derive(Debug)]
pub struct Square {
    pub rank: usize,
    pub file: usize
}

#[derive(Debug)]
struct Move {
    pub source: Square,
    pub dest: Square,
    pub promotion: Option<PieceType>
}

impl Move {
    pub fn from_u16(mv: u16) -> Self {
        fn index(mv: u16, i: usize) -> usize {
            ((mv >> (i * 3)) & 0b111) as usize
        }
        Self {
            dest: Square {
                file: index(mv, 0),
                rank: index(mv, 1)
            },
            source: Square {
                file: index(mv, 2),
                rank: index(mv, 3)
            },
            promotion: match index(mv, 3) {
                0 => None,
                1 => Some(PieceType::Knight),
                2 => Some(PieceType::Bishop),
                3 => Some(PieceType::Rook),
                4 => Some(PieceType::Queen),
                p => unreachable!("Invalid promotion {}", p)
            }
        }
    }
}

#[derive(Debug)]
struct PolyglotEntry {
    pub mv: Move,
    pub weight: u16
}

impl PolyglotEntry {
    pub const SIZE: usize = 16;
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut mv = [0; 2];
        mv.copy_from_slice(&bytes[0..2]);

        let mut weight = [0; 2];
        weight.copy_from_slice(&bytes[2..4]);

        // The rest is the learn value, but it's not implemented.

        Self {
            mv: Move::from_u16(u16::from_be_bytes(mv)),
            weight: u16::from_be_bytes(weight)
        }
    }
}

#[derive(Debug)]
struct PolyglotReader<I> {
    inner: I,
    len: u64
}

impl <I: Seek + Read> PolyglotReader<I> {
    pub fn new(inner: I) -> Result<Self, std::io::Error> {
        let mut inner = inner;
        Ok(Self {
            len: inner.seek(SeekFrom::End(0))? / PolyglotEntry::SIZE as u64,
            inner
        })
    }
    pub fn get(&mut self, key: &PolyglotKey) -> Result<Vec<PolyglotEntry>, std::io::Error> {
        let key = key.polyglot_hash();
        
        let mut entry_exists = false;

        let mut left = 0;
        let mut right = self.len - 1;
        while left < right {
            let middle = (left + right) / 2;
            self.inner.seek(SeekFrom::Start(middle * PolyglotEntry::SIZE as u64))?;
            
            let mut entry_key = [0; 8];
            self.inner.read_exact(&mut entry_key)?;
            let entry_key = u64::from_be_bytes(entry_key);

            if entry_key < key {
                left = middle + 1;
            } else {
                if entry_key == key {
                    entry_exists = true;
                }
                right = middle;
            }
        }

        if !entry_exists {
            return Ok(Vec::new());
        }
        let lower_bound = left;
        
        left = 0;
        right = self.len - 1;
        while left < right {
            let middle = (left + right + 1) / 2;
            self.inner.seek(SeekFrom::Start(middle * PolyglotEntry::SIZE as u64))?;
            
            let mut entry_key = [0; 8];
            self.inner.read_exact(&mut entry_key)?;
            let entry_key = u64::from_be_bytes(entry_key);

            if entry_key > key {
                right = middle - 1;
            } else {
                left = middle;
            }
        }

        let upper_bound = right + 1;
        
        let mut entries = vec![0; (upper_bound - lower_bound) as usize * PolyglotEntry::SIZE];
        self.inner.seek(SeekFrom::Start(lower_bound * PolyglotEntry::SIZE as u64))?;
        self.inner.read_exact(&mut entries)?;
        
        Ok(entries.chunks(PolyglotEntry::SIZE).map(PolyglotEntry::from_bytes).collect())
    }
    pub fn len(&self) -> usize {
        self.len as usize
    }
}

#[cfg(all(test, feature = "chess_lib_helpers"))]
mod tests {
    use crate::*;
    use std::str::FromStr;

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

    #[test]
    fn test_keys() {
        for (i, &(fen, expected)) in TESTS.iter().enumerate() {
            let board = chess::Board::from_str(fen).unwrap();
            let hash = PolyglotKey::from_board(&board).polyglot_hash();
            assert_eq!(hash, expected, "Testing hash for '{}' (Test {})", fen, i + 1);
        }
    }

    #[test]
    fn test_reading() {
        use std::fs::File;

        let file = File::open("test_book.bin").unwrap();
        let mut reader = PolyglotReader::new(file).unwrap();

        for (i, &(fen, _)) in TESTS.iter().enumerate() {
            let board = chess::Board::from_str(fen).unwrap();
            let k = PolyglotKey::from_board(&board);
            let mv = reader.get(&k);
            assert!(mv.is_ok(), "Testing hash for '{}' (Test {})", fen, i + 1);
            println!("Got moves: {:?}", mv.unwrap());
        }
    }
}
