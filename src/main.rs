pub fn move_eval() {
    use othello::othello_lib::board::*;
    let waiting = Pieces {
        bits: 0b1000000001000000000000000000000000000,
    };
    let to_move = Pieces {
        bits: 0b100000010000000000000000000000000000,
    };
    use BoardState::*;
    let mut board = Board {
        to_move,
        waiting,
        black_moving: true,
        board_state: Ongoing,
    };

    println!("{:?}", board);
    while board.board_state == BoardState::Ongoing {
        match board.black_moving {
            true => {
                let bit = othello::othello_lib::minimax(othello::othello_lib::eval_func, &board, 6);
                board.make_move(bit);
            }
            false => {
                let mut line = String::new();
                std::io::stdin()
                    .read_line(&mut line)
                    .expect("stdin failed :(");
                let move_coords = line.trim().parse::<u32>().expect(
                    format!("give the coords properly moron, you entered {}", line).as_str(),
                );
                let bit: u64 = 1 << ((move_coords % 10 - 1) * 8 + (move_coords / 10 - 1));
                board.make_move(bit);
            }
        }
        println!("{:?}", board);
    }
}

fn main() {
    move_eval();
}
