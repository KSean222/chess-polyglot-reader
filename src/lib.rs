use std::io::{Read,Seek,SeekFrom};

pub mod keys;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Side {
    White,
    Black
}

#[cfg(feature = "chess_lib_helpers")]
impl From<chess::Color> for Side {
    fn from(color: chess::Color) -> Side {
        if color == chess::Color::White {
            Side::White
        } else {
            Side::Black
        }
    }
}

#[cfg(feature = "chess_lib_helpers")]
impl From<Side> for chess::Color {
    fn from(side: Side) -> chess::Color {
        if side == Side::White {
            chess::Color::White
        } else {
            chess::Color::Black
        }
    }
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

#[cfg(feature = "chess_lib_helpers")]
impl From<chess::Piece> for PieceType {
    fn from(piece: chess::Piece) -> PieceType {
        match piece {
            chess::Piece::Pawn => PieceType::Pawn,
            chess::Piece::Knight => PieceType::Knight,
            chess::Piece::Bishop => PieceType::Bishop,
            chess::Piece::Rook => PieceType::Rook,
            chess::Piece::Queen => PieceType::Queen,
            chess::Piece::King => PieceType::King
        }
    }
}

#[cfg(feature = "chess_lib_helpers")]
impl From<PieceType> for chess::Piece {
    fn from(piece: PieceType) -> chess::Piece {
        match piece {
            PieceType::Pawn => chess::Piece::Pawn,
            PieceType::Knight => chess::Piece::Knight,
            PieceType::Bishop => chess::Piece::Bishop,
            PieceType::Rook => chess::Piece::Rook,
            PieceType::Queen => chess::Piece::Queen,
            PieceType::King => chess::Piece::King
        }
    }
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
    pub square: Square
}

impl Piece {
    pub fn polyglot_hash(&self) -> u64 {
        let kind = self.piece_type.index() * 2 + (self.side == Side::White) as usize;
        keys::RANDOM_PIECE[64 * kind + 8 * self.square.rank + self.square.file]
    }
}

#[derive(Debug)]
pub struct CastleRights {
    pub queen_side: bool,
    pub king_side: bool
}

#[cfg(feature = "chess_lib_helpers")]
impl From<chess::CastleRights> for CastleRights {
    fn from(rights: chess::CastleRights) -> CastleRights {
        CastleRights {
            queen_side: rights.has_queenside(),
            king_side: rights.has_kingside()
        }
    }
}

#[cfg(feature = "chess_lib_helpers")]
impl From<CastleRights> for chess::CastleRights {
    fn from(rights: CastleRights) -> chess::CastleRights {
        match (rights.queen_side, rights.king_side) {
            (false, false) => chess::CastleRights::NoRights,
            (false, true) => chess::CastleRights::KingSide,
            (true, false) => chess::CastleRights::QueenSide,
            (true, true) => chess::CastleRights::Both
        }
    }
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
pub struct PolyglotKey {
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
        let pieces: Vec<_> = board.combined().into_iter().map(|sq| Piece {
            piece_type: board.piece_on(sq).unwrap().into(),
            square: sq.into(),
            side: board.color_on(sq).unwrap().into()
        }).collect();

        Self {
            pieces,
            white_castle: board.castle_rights(chess::Color::White).into(),
            black_castle: board.castle_rights(chess::Color::Black).into(),
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
            turn: board.side_to_move().into()
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Square {
    pub rank: usize,
    pub file: usize
}

#[cfg(feature = "chess_lib_helpers")]
impl From<chess::Square> for Square {
    fn from(sq: chess::Square) -> Square {
        Square {
            rank: sq.get_rank().to_index(),
            file: sq.get_file().to_index()
        }
    }
}

#[cfg(feature = "chess_lib_helpers")]
impl From<Square> for chess::Square {
    fn from(sq: Square) -> chess::Square {
        chess::Square::make_square(chess::Rank::from_index(sq.rank), chess::File::from_index(sq.file))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Move {
    pub source: Square,
    pub dest: Square,
    pub promotion: Option<PieceType>
}

#[cfg(feature = "chess_lib_helpers")]
impl From<chess::ChessMove> for Move {
    fn from(mv: chess::ChessMove) -> Move {
        Move {
            source: mv.get_source().into(),
            dest: mv.get_dest().into(),
            promotion: mv.get_promotion().map(|p| p.into())
        }
    }
}

#[cfg(feature = "chess_lib_helpers")]
impl From<Move> for chess::ChessMove {
    fn from(mv: Move) -> chess::ChessMove {
        chess::ChessMove::new(mv.source.into(), mv.dest.into(), mv.promotion.map(|p| p.into()))
    }
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
            promotion: match index(mv, 4) {
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
pub struct PolyglotEntry {
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
pub struct PolyglotReader<I> {
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
        let hash = key.polyglot_hash();
        
        let mut entry_exists = false;

        let mut left = 0;
        let mut right = self.len - 1;
        while left < right {
            let middle = (left + right) / 2;
            self.inner.seek(SeekFrom::Start(middle * PolyglotEntry::SIZE as u64))?;
            
            let mut entry_key = [0; 8];
            self.inner.read_exact(&mut entry_key)?;
            let entry_key = u64::from_be_bytes(entry_key);

            if entry_key < hash {
                left = middle + 1;
            } else {
                if entry_key == hash {
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

            if entry_key > hash {
                right = middle - 1;
            } else {
                left = middle;
            }
        }

        let upper_bound = right + 1;
        
        let mut entries = vec![0; (upper_bound - lower_bound) as usize * PolyglotEntry::SIZE];
        self.inner.seek(SeekFrom::Start(lower_bound * PolyglotEntry::SIZE as u64))?;
        self.inner.read_exact(&mut entries)?;
        
        let entries = entries.chunks(PolyglotEntry::SIZE)
            .map(|entry| {
                let mut entry = PolyglotEntry::from_bytes(&entry[8..]);
                if entry.mv.source.file == 4 && entry.mv.source.rank == entry.mv.dest.rank {
                    let is_castle = match entry.mv.dest {
                        Square { file: 7, rank: 0 } => key.white_castle.king_side,
                        Square { file: 0, rank: 0 } => key.white_castle.queen_side,
                        Square { file: 7, rank: 7 } => key.black_castle.king_side,
                        Square { file: 0, rank: 7 } => key.black_castle.queen_side,
                        _ => false
                    };
                    if is_castle {
                        if entry.mv.dest.file < entry.mv.source.file {
                            entry.mv.dest.file += 1;
                        } else {
                            entry.mv.dest.file -= 1;
                        }
                    }
                }
                entry
            })
            .collect();

        Ok(entries)
    }
    pub fn len(&self) -> usize {
        self.len as usize
    }
}
