use crate::board::{self, Board};
use eframe::{self, egui};

pub fn run() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Othello?",
        options,
        Box::new(|_cc| Box::new(Board::default())),
    )
}

impl eframe::App for Board {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let (_response, mut painter) = match self.board_state {
                board::BoardState::Ongoing => {
                    let (response, painter) =
                        ui.allocate_painter(ui.min_size(), egui::Sense::click());
                    let cell_size = ui.min_size() / 8.0;
                    // println!(
                    //     "Cell width: {}, Cell height: {}",
                    //     ui.min_size().x,
                    //     ui.min_size().y
                    // );
                    if !self.black_moving {
                        self.make_move(crate::evaluation::best_move(
                            crate::evaluation::better_eval,
                            self,
                            8,
                        ));
                    } else if response.clicked() && self.black_moving {
                        if let Some(egui::Pos2 { x, y }) = response.hover_pos() {
                            let x = (x as u64) / cell_size.x as u64;
                            let y = (y as u64) / cell_size.y as u64;
                            let bit = 1 << (x + y * 8);
                            self.safe_make_move(bit);
                            // println!("x = {x}, y = {y}");
                        }
                    }
                    (response, painter)
                }
                board::BoardState::Drawn => {
                    ui.vertical_centered(|i| {
                        i.horizontal_centered(|inter| {
                            if inter.button("Draw - Retry?").clicked() {
                                *self = Board::default();
                            }
                        })
                    });
                    ui.allocate_painter(ui.min_size(), egui::Sense::click())
                }
                board::BoardState::Won => {
                    let s = match self.black_moving {
                        true => "Black",
                        false => "White",
                    };
                    ui.vertical_centered(|i| {
                        i.horizontal_centered(|inter| {
                            if inter
                                .button(format!(
                                    "{s} won! The winner had {} pieces, the loser had {} - Retry?",
                                    self.to_move.clone().count(),
                                    self.waiting.clone().count()
                                ))
                                .clicked()
                            {
                                *self = Board::default();
                            }
                        })
                    });
                    ui.allocate_painter(ui.min_size(), egui::Sense::click())
                }
            };
            self.draw(&mut painter);
        });
    }
}

impl Board {
    fn draw(&self, painter: &mut egui::Painter) {
        let rect = painter.clip_rect();
        let cell_size = (rect.max - rect.min) / 8.0;
        let (black, white) = match self.black_moving {
            true => (self.to_move.bits, self.waiting.bits),
            false => (self.waiting.bits, self.to_move.bits),
        };
        let moves = self.each_move();
        // println!("Rendering...");
        (0..64u64).for_each(|i| {
            painter.rect_filled(
                egui::Rect::from_center_size(
                    egui::Pos2::new(
                        (i % 8) as f32 * cell_size.x + cell_size.x / 2.0,
                        (i / 8) as f32 * cell_size.y + cell_size.y / 2.0,
                    ),
                    cell_size,
                ),
                egui::Rounding::none(),
                egui::Color32::from_rgb(18, 140 + 15 * ((i + i / 8) & 1) as u8, 45),
            );
            let (bbit, wbit) = ((black >> i) & 1, (white >> i) & 1);
            painter.circle_filled(
                egui::Pos2::new(
                    (i % 8) as f32 * cell_size.x + cell_size.x / 2.0,
                    (i / 8) as f32 * cell_size.y + cell_size.y / 2.0,
                ),
                cell_size.x.min(cell_size.y) / 3.0,
                if (moves.bits >> i) & 1 == 1u64 && self.black_moving {
                    egui::Color32::from_rgb(3, 240 + 10 * ((i + i / 8) & 1) as u8, 252)
                } else if bbit == 1 {
                    egui::Color32::BLACK
                } else if wbit == 1 {
                    egui::Color32::WHITE
                } else {
                    egui::Color32::from_rgb(18, 140 + 15 * ((i + i / 8) & 1) as u8, 45)
                },
            )
        })
    }
}
