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
        let moves = self.each_move();
        print!("  1 2 3 4 5 6 7 8\n1");
        use std::io::Write;
        std::io::stdout().flush().expect("Flush failed");
        for i in 0..64 {
            let (bbit, wbit) = ((black >> i) & 1, (white >> i) & 1);
            use colored::Colorize;
            let next_str = if moves.clone().any(|x| (x >> i) & 1 == 1u64) {
                "██".color(colored::Color::TrueColor {
                    r: 3,
                    g: 240 + 10 * ((i + i / 8) & 1),
                    b: 252,
                })
            } else if bbit == 1 {
                "██".black()
            } else if wbit == 1 {
                "██".white()
            } else {
                "██".color(colored::Color::TrueColor {
                    r: 18,
                    g: 140 + 15 * ((i + i / 8) & 1),
                    b: 45,
                })
            };
            write!(f, "{}", next_str).expect("Formatting of Board failed");
            if (i + 1) & 7 == 0 && i != 63 {
                write!(f, "{}", format!("\n{}", i / 8 + 2)).expect("Formatting of Board failed");
            }
        }
        write!(f, "")
    }
}

impl Board {
    // The second item of each tuples are the squares where you shouldn't shl from for each dir
    pub const DIRECTIONS: [(i8, u64); 8] = [
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
        // Returning the moves because the caller often needs it anyway
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
                        self.black_moving ^= true;
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

    pub fn safe_make_move(&mut self, bit: u64) -> Result<Pieces, String> {
        let mut moves = self.each_move();
        match moves.any(|x| x == bit) {
            true => {
                self.make_move(bit);
                Ok(moves)
            }
            false => Err(String::from("Move is not legal")),
        }
    }

    pub fn children(&self, moves: &Pieces) -> Vec<(Board, Pieces)> {
        // Moves is a parameter as the caller will often already have calculated it
        // Returning both positions and the available moves as the latter
        // must be calculated and I don't want to waste the computation
        let kids: Vec<_> = moves
            .clone()
            .map(|bit| {
                let mut copy = self.clone();
                let next = copy.make_move(bit);
                (copy, next)
            })
            .collect();
        kids
    }
}
