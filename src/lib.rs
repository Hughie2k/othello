pub mod board;
pub mod cli;
pub mod evaluation;
pub mod gui;
pub mod net;
#[allow(dead_code)]
#[cfg(test)]
#[test]
fn move_generation() {
    use board::*;
    // Use https://tearth.dev/bitboard-viewer/ to view the positions
    let to_move = Pieces {
        bits: 0b1000000001000000000000000000000000000,
    };
    let waiting = Pieces {
        bits: 0b100000010000000000000000000000000000,
    };
    use BoardState::*;
    let board = Board {
        to_move,
        waiting,
        black_moving: true,
        board_state: Ongoing,
    };
    assert_eq!(
        board.each_move().bits,
        0b10000000010000100000000100000000000000000000
    );
    let to_move = Pieces {
        bits: 0b11000000000000001100000011000000000000000000000000000,
    };
    let waiting = Pieces {
        bits: 0b110000000000000000000000000000000000000000000,
    };
    let board = Board {
        to_move,
        waiting,
        black_moving: true,
        board_state: Ongoing,
    };
    assert_eq!(
        board.each_move().bits,
        0b100100000000000010010000000000000000000000000000000000
    );
    let board = Board {
        to_move: Pieces {
            bits: 0xfe00c80808000000,
        },
        waiting: Pieces {
            bits: 0xfe101010000000,
        },
        black_moving: true,
        board_state: Ongoing,
    };

    println!("{:?}", board);
    assert_eq!(board.each_move().bits, 0x272020300000);
}
