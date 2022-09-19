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
    eval: fn(&Board, &Pieces) -> T, // Static evaluation function, although static is not enforcable
    (board, moves): &(Board, Pieces),
    depth: u8,    // How many moves ahead to we want to consider?
    mut alpha: T, // The evaluation of the best move found so far for black
    mut beta: T,  // alpha, but for white
) -> T {
    if depth == 0 || moves.bits == 0 {
        return eval(board, moves);
    }
    let children = board.children(moves);
    // This match statement avoids code duplication, where the same function is written almost identically for both colours
    let (mut best_eval, cmp) = match board.black_moving {
        true => (alpha.clone(), T::max as fn(T, T) -> T),
        false => (beta.clone(), T::min as fn(T, T) -> T),
    };
    for (next_board, next_moves) in children {
        best_eval = cmp(
            best_eval,
            minimax(
                eval,
                &(next_board, next_moves),
                depth - 1,
                alpha.clone(),
                beta.clone(),
            ),
        );
        if board.black_moving {
            alpha = best_eval.clone();
        } else {
            beta = best_eval.clone();
        }
        // Good positions for black give greater evaluations, opposite for white
        // Consider alpha and beta, the best evaluation for black so far and the best for white so far
        // If we compare the bes&mut t moves found so far for black and white and we find that the opposition
        // Can force a better position for them than the one we are currently considering, we need not consider
        // Any more children of this position as the opposition can and should avoid this position entirely
        // If they happen not to do that, that's all the better for us as they are playing sub-optimally
        if beta <= alpha {
            //println!("Prune!");
            break;
        }
    }
    best_eval
}

pub fn best_move<T: Ord + Clone + num::Bounded>(
    eval: fn(&Board, &Pieces) -> T,
    board: &Board,
    depth: u8,
) -> u64 {
    let moves = board.each_move();
    if board.black_moving {
        board
            .children(&moves)
            .iter()
            .zip(moves)
            .map(|(pos, move_bit)| {
                (
                    minimax(eval, pos, depth - 1, T::min_value(), T::max_value()),
                    move_bit,
                )
            })
            .collect::<Vec<_>>()
            .iter()
            .max_by_key(|(value, _)| value)
            .unwrap()
            .1
    } else {
        board
            .children(&moves)
            .iter()
            .zip(moves)
            .map(|(pos, move_bit)| {
                (
                    minimax(eval, pos, depth - 1, T::min_value(), T::max_value()),
                    move_bit,
                )
            })
            .collect::<Vec<_>>()
            .iter()
            .min_by_key(|(value, _)| value)
            .unwrap()
            .1
    }
}

fn frontier(board: &Board) -> (Pieces, Pieces) {
    let (mut moving_front, mut waiting_front) = (0u64, 0u64);
    let empty = !(board.to_move.bits | board.waiting.bits);
    let safe_shl = |a: u64, b: i8| if b > 0 { a << b } else { a >> (-b) };
    for (dir, guard) in Board::DIRECTIONS {
        moving_front |= safe_shl(board.to_move.bits & !guard, dir) & empty;
        waiting_front |= safe_shl(board.waiting.bits & !guard, dir) & empty;
    }
    (
        Pieces { bits: moving_front },
        Pieces {
            bits: waiting_front,
        },
    )
}

pub fn better_eval(board: &Board, moves: &Pieces) -> i16 {
    let c = board.black_moving as i16 * 2 - 1;
    use BoardState::*;
    match &board.board_state {
        Won => i16::MAX * c,
        Drawn => 4 * c, // completely arbitrary
        Ongoing => {
            let corner_balance = c
                * (Pieces {
                    bits: board.to_move.bits & 0x8100000000000081,
                }
                .count() as i16
                    - Pieces {
                        bits: board.waiting.bits & 0x8100000000000081,
                    }
                    .count() as i16);
            let material_balance =
                c * (board.to_move.clone().count() as i16 - board.waiting.clone().count() as i16);
            let (moving_front, waiting_front) = frontier(board);
            let frontier_balance = c * (moving_front.count() as i16 - waiting_front.count() as i16);
            let move_count = c * moves.clone().count() as i16;
            corner_balance * 4000 + material_balance - frontier_balance * 10 + move_count * 1000
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
    fn eval1(board: &Board, moves: &Pieces) -> i8 {
        let c = board.black_moving as i8 * 2 - 1;
        match board.board_state {
            Won => i8::MAX * c,
            Drawn => 4 * c, // completely arbitrary, can be improved
            Ongoing => {
                c * (moves.clone().count() as i8
                    + ((board.to_move.clone().count() as i8 - board.waiting.clone().count() as i8)
                        >> 2))
            }
        }
    }
    fn eval2(board: &Board, moves: &Pieces) -> i8 {
        let c = board.black_moving as i8 * 2 - 1;
        match board.board_state {
            Won => i8::MAX * c,
            Drawn => 4 * c, // completely arbitrary, can be improved
            Ongoing => {
                c * (moves.clone().count() as i8
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
                    board.make_move(best_move(eval2, &board, 5));
                }
                false => {
                    board.make_move(best_move(better_eval, &board, 4));
                }
            },
        }
        println!("{board:#?}");
    }
}
