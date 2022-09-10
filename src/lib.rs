#[allow(dead_code)]
#[cfg(test)]
pub mod othello {
    use colored::Colorize;
    #[derive(Clone, PartialEq, Debug)]
    enum BoardState {
        Won, // to_move has won
        Drawn,
        Ongoing,
    }
    #[derive(Clone)]
    struct Pieces {
        bits: u64,
    }
    #[derive(Clone)]
    struct Board {
        to_move: Pieces,
        waiting: Pieces,
        black_moving: bool,
    }

    trait Player {
        fn get_move(&self, _: &GameState) -> u64;
    }
    #[derive(Clone)]
    struct GameState {
        board: Board,
        board_state: BoardState,
    }

    struct CPUPlayer {
        strategy: fn(&GameState) -> u64,
    }

    impl Player for CPUPlayer {
        fn get_move(&self, state: &GameState) -> u64 {
            (self.strategy)(state)
        }
    }
    impl GameState {
        fn each_move(&self) -> Pieces {
            let (friend, opponent) = (&self.board.to_move.bits, &self.board.waiting.bits);
            let safe_shl = |a: u64, b: i8| if b > 0 { a << b } else { a >> (-b) };
            let directions: [(i8, u64); 8] = [
                (8, 0),
                (9, 0x8080808080808080),
                (1, 0x8080808080808080),
                (-7, 0x8080808080808080),
                (-8, 0),
                (-9, 0x0101010101010101),
                (-1, 0x0101010101010101),
                (7, 0x0101010101010101),
            ];
            let mut moves: u64 = 0;
            let empty = !(friend | opponent);
            for (dir, guard) in directions {
                let mut candidates = opponent & safe_shl(*friend & !guard, dir);
                while candidates != 0 {
                    moves |= empty & safe_shl(candidates & !guard, dir);
                    candidates = opponent & safe_shl(candidates & !guard, dir);
                }
            }
            Pieces { bits: moves }
        }
        fn make_move(&mut self, bit: u64) -> Pieces {
            // Returning the number of pieces for optimization
            let directions: [(i8, u64); 8] = [
                (8, 0),
                (9, 0x8080808080808080),
                (1, 0x8080808080808080),
                (-7, 0x8080808080808080),
                (-8, 0),
                (-9, 0x0101010101010101),
                (-1, 0x0101010101010101),
                (7, 0x0101010101010101),
            ];
            let safe_shl = |a: u64, b: i8| if b > 0 { a << b } else { a >> (-b) };
            for (dir, guard) in directions {
                let mut line = 0u64;
                let mut moving_bit = safe_shl(bit & !guard, dir);
                while moving_bit & self.board.waiting.bits != 0 {
                    line |= moving_bit;
                    moving_bit = safe_shl(moving_bit & !guard, dir);
                }
                if moving_bit & self.board.to_move.bits != 0 {
                    self.board.to_move.bits |= line;
                    self.board.waiting.bits &= !line;
                }
            }
            self.board.to_move.bits |= bit;
            std::mem::swap(&mut self.board.to_move, &mut self.board.waiting);
            self.board.black_moving ^= true;
            let mut moves = self.each_move();
            self.board_state = if moves.bits == 0 {
                std::mem::swap(&mut self.board.to_move, &mut self.board.waiting);
                self.board.black_moving ^= true;
                moves = self.each_move();
                if moves.bits == 0 {
                    use std::cmp::Ordering;
                    // No possible moves for both sides
                    match self
                        .board
                        .to_move
                        .clone()
                        .count()
                        .cmp(&self.board.waiting.clone().count())
                    {
                        Ordering::Greater => BoardState::Won,
                        Ordering::Less => {
                            std::mem::swap(&mut self.board.to_move, &mut self.board.waiting);
                            BoardState::Won
                        }
                        Ordering::Equal => BoardState::Drawn,
                    }
                } else {
                    BoardState::Ongoing
                }
            } else {
                BoardState::Ongoing
            };
            moves
        }
    }

    impl Iterator for Pieces {
        type Item = u64;
        fn next(&mut self) -> Option<Self::Item> {
            if self.bits == 0 {
                None
            } else {
                let bit = self.bits ^ (self.bits & (self.bits - 1));
                self.bits &= self.bits - 1;
                Some(bit)
            }
        }
    }

    impl std::fmt::Debug for Board {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if self.to_move.bits & self.waiting.bits != 0 {
                panic!("Squares aren't for sharing!");
            }
            let (black, white) = match self.black_moving {
                true => (self.to_move.bits, self.waiting.bits),
                false => (self.waiting.bits, self.to_move.bits),
            };
            for i in 0..64 {
                let (bbit, wbit) = ((black >> i) & 1, (white >> i) & 1);
                let next_str = if bbit == 1 {
                    "██".black()
                } else if wbit == 1 {
                    "██".white()
                } else {
                    "██".green()
                };
                write!(f, "{}", next_str).expect("Formatting of Board failed");
                if (i + 1) & 7 == 0 && i != 63 {
                    write!(f, "\n").expect("Formatting of Board failed");
                }
            }
            write!(f, "")
        }
    }
    fn mobility(state: &GameState) -> i8 {
        // maximizing bool for, now, should be changed
        // +ve for black winning
        use BoardState::*;
        let c = state.board.black_moving as i8 * 2 - 1;
        match state.board_state {
            Won => i8::MAX * c,
            Drawn => 4 * c, // completely arbitrary, can be improved
            Ongoing => c * state.each_move().count() as i8,
        }
    }

    fn minimax<T: Ord + Clone>(eval: fn(&GameState) -> T, state: &GameState, depth: u8) -> u64 {
        fn recurse_eval<T: Ord + Clone>(
            eval: fn(&GameState) -> T,
            state: &GameState,
            depth: u8,
        ) -> T {
            if depth == 0 {
                eval(state)
            } else {
                let min_or_max = if state.board.black_moving {
                    Iterator::max
                } else {
                    Iterator::min
                };
                let bound_recurse_eval = |x| -> T { recurse_eval(eval, x, depth - 1) };
                let moves = state.each_move();
                if moves.bits == 0 {
                    return eval(state);
                }
                let mut states = vec![state.clone(); moves.clone().count()];
                states
                    .iter_mut()
                    .zip(moves)
                    .for_each(|(a, b)| a.make_move(b));
                min_or_max(states.iter().map(bound_recurse_eval)).unwrap()
            }
        }
        let moves = state.each_move();
        if moves.bits == 0 {
            panic!("{:?} - No moves. Fix the code!", state.board_state);
        }
        let mut states = vec![state.clone(); moves.clone().count()];
        states
            .iter_mut()
            .zip(moves.clone())
            .for_each(|(a, b)| GameState::make_move(a, b));
        let (mut best_move, mut lowest_score): (Option<u64>, Option<T>) = (None, None);
        for (state, bit) in states.iter().zip(moves.clone()) {
            let score = recurse_eval(eval, state, depth);
            match &lowest_score {
                None => {
                    (best_move, lowest_score) = (Some(bit), Some(recurse_eval(eval, state, depth)))
                }
                Some(other_score) => {
                    (best_move, lowest_score) = if score < *other_score {
                        (Some(bit), Some(score))
                    } else {
                        (best_move, lowest_score)
                    }
                }
            }
        }
        best_move.unwrap()
    }
    #[test]
    fn move_generation() {
        // Use https://tearth.dev/bitboard-viewer/ to view the positions
        let to_move = Pieces {
            bits: 0b1000000001000000000000000000000000000,
        };
        let waiting = Pieces {
            bits: 0b100000010000000000000000000000000000,
        };
        let board = Board {
            to_move,
            waiting,
            black_moving: true,
        };
        use BoardState::*;
        let state = GameState {
            board,
            board_state: Ongoing,
        };
        assert_eq!(
            state.each_move().bits,
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
        };
        let state = GameState {
            board,
            board_state: Ongoing,
        };
        assert_eq!(
            state.each_move().bits,
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
        };
        let state = GameState {
            board,
            board_state: Ongoing,
        };
        println!("{:?}", state.board);
        assert_eq!(state.each_move().bits, 0x272020300000);
    }
    #[test] // Not a proper test, just using it to check out the minimax
    fn move_eval() {
        let to_move = Pieces {
            bits: 0b1000000001000000000000000000000000000,
        };
        let waiting = Pieces {
            bits: 0b100000010000000000000000000000000000,
        };
        let board = Board {
            to_move,
            waiting,
            black_moving: true,
        };
        use BoardState::*;
        let mut state = GameState {
            board,
            board_state: Ongoing,
        };
        println!("{:?}", state.board);
        while state.board_state == BoardState::Ongoing {
            match state.board.black_moving {
                true => {
                    let bit = minimax(mobility, &state, 6);
                    state.make_move(bit);
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
                    state.make_move(bit);
                }
            }
            println!("{:?}", state.board);
        }
    }
}
