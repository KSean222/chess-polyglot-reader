use chess_polyglot_reader::*;
use std::str::FromStr;

const TESTS: &[&str] = &[
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
];

#[test]
fn test_reader() {
    use std::fs::File;

    let file = File::open("test-data/test_book.bin").unwrap();
    let mut reader = PolyglotReader::new(file).unwrap();

    for (i, fen) in TESTS.iter().enumerate() {
        let board = chess::Board::from_str(fen).unwrap();
        let k = PolyglotKey::from_board(&board);
        let mv = reader.get(&k).unwrap();
        assert!(!mv.is_empty(), "Testing reading openings for '{}' (Test {})", fen, i + 1);
    }
}