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

pub fn minimax<T: Ord + Clone>(eval: fn(&Board) -> T, board: &Board, depth: u8) -> u64 {
    fn recurse_eval<T: Ord + Clone>(
        eval: fn(&Board) -> T,
        board: &Board,
        depth: u8,
        moves: Pieces,
    ) -> T {
        if depth == 0 {
            eval(board)
        } else {
            let bound_recurse_eval = |x, m| -> T { recurse_eval(eval, x, depth - 1, m) };
            if moves.bits == 0 {
                return eval(board);
            }
            let mut boards = vec![board.clone(); moves.clone().count()];
            let mut evaluations: Vec<T> = Vec::new();
            for (board, move_bit) in boards.iter_mut().zip(moves.clone()) {
                let moves = board.make_move(move_bit);
                evaluations.push(bound_recurse_eval(board, moves));
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
        evaluations.push((recurse_eval(eval, board, depth, moves), move_bit));
    }
    evaluations
        .iter()
        .max_by_key(|(evaluation, _)| evaluation)
        .unwrap()
        .1
}
