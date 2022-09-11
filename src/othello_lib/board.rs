#[derive(Clone, PartialEq, Debug)]
pub enum BoardState {
    Won, // to_move has won, so we don't need a `BoardState::Lost`
    Drawn,
    Ongoing,
}
#[derive(Clone)]
pub struct Pieces {
    pub bits: u64,
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

#[derive(Clone)]
pub struct Board {
    pub to_move: Pieces,
    pub waiting: Pieces,
    pub black_moving: bool,
    pub board_state: BoardState,
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
            use colored::Colorize;
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

impl Board {
    const DIRECTIONS: [(i8, u64); 8] = [
        (8, 0),
        (9, 0x8080808080808080),
        (1, 0x8080808080808080),
        (-7, 0x8080808080808080),
        (-8, 0),
        (-9, 0x0101010101010101),
        (-1, 0x0101010101010101),
        (7, 0x0101010101010101),
    ];
    pub fn each_move(&self) -> Pieces {
        let (friend, opponent) = (&self.to_move.bits, &self.waiting.bits);
        let safe_shl = |a: u64, b: i8| if b > 0 { a << b } else { a >> (-b) };
        let mut moves: u64 = 0;
        let empty = !(friend | opponent);
        for (dir, guard) in Board::DIRECTIONS {
            let mut candidates = opponent & safe_shl(*friend & !guard, dir);
            while candidates != 0 {
                moves |= empty & safe_shl(candidates & !guard, dir);
                candidates = opponent & safe_shl(candidates & !guard, dir);
            }
        }
        Pieces { bits: moves }
    }
    pub fn make_move(&mut self, bit: u64) -> Pieces {
        // Returning the number of pieces for optimization
        let safe_shl = |a: u64, b: i8| if b > 0 { a << b } else { a >> (-b) };
        for (dir, guard) in Board::DIRECTIONS {
            let mut line = 0u64;
            let mut moving_bit = safe_shl(bit & !guard, dir);
            while moving_bit & self.waiting.bits != 0 {
                line |= moving_bit;
                moving_bit = safe_shl(moving_bit & !guard, dir);
            }
            if moving_bit & self.to_move.bits != 0 {
                self.to_move.bits |= line;
                self.waiting.bits &= !line;
            }
        }
        self.to_move.bits |= bit;
        std::mem::swap(&mut self.to_move, &mut self.waiting);
        self.black_moving ^= true;
        let mut moves = self.each_move();
        self.board_state = if moves.bits == 0 {
            std::mem::swap(&mut self.to_move, &mut self.waiting);
            self.black_moving ^= true;
            moves = self.each_move();
            if moves.bits == 0 {
                use std::cmp::Ordering;
                // No possible moves for both sides
                match self
                    .to_move
                    .clone()
                    .count()
                    .cmp(&self.waiting.clone().count())
                {
                    Ordering::Greater => BoardState::Won,
                    Ordering::Less => {
                        std::mem::swap(&mut self.to_move, &mut self.waiting);
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
