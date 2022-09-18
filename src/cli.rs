pub fn play() {
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
    let input = |prompt: &str| -> String {
        let mut line = String::new();
        print!("{prompt}");
        io::stdout().flush().expect("Flush failed");
        io::stdin().read_line(&mut line).expect("stdin failed");
        String::from(line.trim())
    };
    println!("{:#?}\n", board);
    loop {
        match board.board_state {
            Ongoing => {
                match board.black_moving {
                    true => {
                        let bit =
                            crate::evaluation::minimax(crate::evaluation::better_eval, &board, 10);
                        board.make_move(bit);
                    }
                    false => {
                        // I use a closure here in order to access the `input(&str) -> String
                        let get_coords = || -> u8 {
                            let msg =
                                "Please enter coordinates indexed from 1, in the form `xy`.\n\
                               x increases to the right, y increases downwards: ";
                            let move_coords = loop {
                                match input(msg).parse::<u8>() {
                                    Ok(x) => break x,
                                    Err(e) => println!("{e}\nYou did not enter valid coordinates"),
                                };
                            };
                            move_coords
                        };
                        let move_coords = get_coords();
                        let mut bit: u64 =
                            1 << ((move_coords % 10 - 1) * 8 + (move_coords / 10 - 1));
                        loop {
                            match board.safe_make_move(bit) {
                                Ok(_) => break,
                                Err(_) => {
                                    println!("The move you entered is illegal - try again");
                                    let move_coords = get_coords();
                                    bit =
                                        1 << ((move_coords % 10 - 1) * 8 + (move_coords / 10 - 1));
                                }
                            }
                        }
                    }
                };
                println!("{board:?}\n");
            }
            Drawn => {
                println!("It's a draw!");
                break;
            }
            Won => {
                match board.black_moving {
                    true => println!("Black wins!"),
                    false => println!("White wins!"),
                }
                let (win, loss) = (board.to_move.clone().count(), board.waiting.clone().count());
                println!("The winner had {win} pieces, the loser had {loss} pieces");
                break;
            }
        }
    }
}
