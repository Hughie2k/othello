use crate::board::*;
pub fn eval_func(board: &Board) -> i8 {
    // maximizing bool for, now, should be changed
    // +ve for black winning
    use BoardState::*;
    let c = board.black_moving as i8 * 2 - 1;
    match board.board_state {
        Won => i8::MAX * c,
        Drawn => 4 * c, // completely arbitrary, can be improved
        Ongoing => {
            c * board.each_move().count() as i8
                + ((board.to_move.clone().count() as i8 - board.waiting.clone().count() as i8) >> 2)
        }
    }
}

pub fn minimax<T: Ord + Clone + num::Bounded>(
    eval: fn(&Board) -> T,
    board: &Board,
    depth: u8,
) -> u64 {
    fn recurse_eval<T: Ord + Clone + num::Bounded>(
        eval: fn(&Board) -> T,
        board: &Board,
        depth: u8,
        moves: Pieces,
        mut alpha: T,
        mut beta: T,
    ) -> T {
        if depth == 0 {
            eval(board)
        } else {
            if moves.bits == 0 {
                return eval(board);
            }
            let mut boards = vec![board.clone(); moves.clone().count()];
            let mut evaluations: Vec<T> = Vec::new();
            for (board, move_bit) in boards.iter_mut().zip(moves.clone()) {
                let moves = board.make_move(move_bit);
                let valuation =
                    recurse_eval(eval, board, depth - 1, moves, alpha.clone(), beta.clone());
                if board.black_moving {
                    alpha = alpha.max(valuation.clone());
                } else {
                    beta = beta.min(valuation.clone());
                }
                evaluations.push(valuation);
                if alpha > beta {
                    break;
                }
            }
            if board.black_moving {
                evaluations.iter().max().unwrap().clone()
            } else {
                evaluations.iter().min().unwrap().clone()
            }
        }
    }
    let moves = board.each_move();
    if moves.bits == 0 {
        panic!("{:?} - No moves. Fix the code!", board.board_state);
    }
    let mut boards = vec![board.clone(); moves.clone().count()];
    let mut evaluations: Vec<(T, u64)> = Vec::new();
    for (board, move_bit) in boards.iter_mut().zip(moves.clone()) {
        let moves = board.make_move(move_bit);
        evaluations.push((
            recurse_eval(eval, board, depth, moves, T::min_value(), T::max_value()),
            move_bit,
        ));
    }
    if board.black_moving {
        evaluations
            .iter()
            .max_by_key(|(evaluation, _)| evaluation)
            .unwrap()
            .1
    } else {
        evaluations
            .iter()
            .min_by_key(|(evaluation, _)| evaluation)
            .unwrap()
            .1
    }
}

pub fn better_eval(board: &Board) -> i8 {
    let c = board.black_moving as i8 * 2 - 1;
    use BoardState::*;
    match &board.board_state {
        Won => i8::MAX * c,
        Drawn => 4 * c, // completely arbitrary
        Ongoing => {
            c * (board.each_move().count() as i8
                + ((board.to_move.clone().count() as i8 - board.waiting.clone().count() as i8)
                    >> 2)
                + (((Pieces {
                    bits: board.to_move.bits & 0x8100000000000081,
                }
                .count() as i8)
                    << 2)
                    - (Pieces {
                        bits: board.waiting.bits & 0x8100000000000081,
                    }
                    .count() as i8)
                    << 2))
        }
    }
}

#[test]
fn cmp_eval() {
    use crate::board::*;
    use std::io::{self, Write};
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
    fn eval1(board: &Board) -> i8 {
        let c = board.black_moving as i8 * 2 - 1;
        match board.board_state {
            Won => i8::MAX * c,
            Drawn => 4 * c, // completely arbitrary, can be improved
            Ongoing => {
                c * (board.each_move().count() as i8
                    + ((board.to_move.clone().count() as i8 - board.waiting.clone().count() as i8)
                        >> 2))
            }
        }
    }
    fn eval2(board: &Board) -> i8 {
        let c = board.black_moving as i8 * 2 - 1;
        match board.board_state {
            Won => i8::MAX * c,
            Drawn => 4 * c, // completely arbitrary, can be improved
            Ongoing => {
                c * (board.each_move().count() as i8
                    + ((board.to_move.clone().count() as i8 - board.waiting.clone().count() as i8)
                        >> 2)
                    + (((Pieces {
                        bits: board.to_move.bits & 0x8100000000000081,
                    }
                    .count() as i8)
                        << 2)
                        - (Pieces {
                            bits: board.waiting.bits & 0x8100000000000081,
                        }
                        .count() as i8)
                        << 2))
            }
        }
    }
    loop {
        match board.board_state {
            Won => {
                match board.black_moving {
                    true => println!("Black wins!"),
                    false => println!("White wins!"),
                }
                let (win, loss) = (board.to_move.clone().count(), board.waiting.clone().count());
                println!("The winner had {win} pieces, the loser had {loss} pieces");
                break;
            }
            Drawn => {
                println!("It's a draw!");
                break;
            }
            Ongoing => match board.black_moving {
                true => {
                    board.make_move(minimax(eval2, &board, 3));
                }
                false => {
                    board.make_move(minimax(eval1, &board, 9));
                }
            },
        }
        println!("{board:#?}");
    }
}
