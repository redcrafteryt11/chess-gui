use egui::{Color32, Painter, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};
use crate::game::PlayerColor;

const LIGHT: Color32 = Color32::from_rgb(240, 217, 181);
const DARK:  Color32 = Color32::from_rgb(181, 136, 99);
const SEL:   Color32 = Color32::from_rgba_premultiplied(20, 85, 30, 180);
const HINT:  Color32 = Color32::from_rgba_premultiplied(20, 85, 30, 100);
const LAST:  Color32 = Color32::from_rgba_premultiplied(205, 210, 106, 160);

#[derive(Debug, Clone, PartialEq)]
pub struct BoardState {
    pub selected: Option<u8>,
    pub legal_hints: Vec<u8>,
    pub last_move: Option<(u8, u8)>,
    pub flipped: bool,
}

impl BoardState {
    pub fn new() -> Self {
        Self { selected: None, legal_hints: Vec::new(), last_move: None, flipped: false }
    }
}

pub struct BoardWidget<'a> {
    pub fen: &'a str,
    pub state: &'a mut BoardState,
    pub interactive: bool,
    pub flip: bool,
}

impl<'a> BoardWidget<'a> {
    pub fn show(&mut self, ui: &mut Ui) -> Option<(u8, u8)> {
        let size = ui.available_width().min(ui.available_height());
        let sq_size = size / 8.0;
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
        let painter = ui.painter_at(rect);

        let pieces = parse_fen(self.fen);

        for rank in 0u8..8 {
            for file in 0u8..8 {
                let sq = rank * 8 + file;
                let (draw_file, draw_rank) = if self.flip {
                    (7 - file, rank)
                } else {
                    (file, 7 - rank)
                };

                let x = rect.left() + draw_file as f32 * sq_size;
                let y = rect.top() + draw_rank as f32 * sq_size;
                let sq_rect = Rect::from_min_size(Pos2::new(x, y), Vec2::splat(sq_size));

                let is_light = (file + rank) % 2 == 1;
                let mut bg = if is_light { LIGHT } else { DARK };

                if self.state.last_move.map_or(false, |(f, t)| f == sq || t == sq) {
                    bg = blend(bg, LAST);
                }

                painter.rect_filled(sq_rect, 0.0, bg);

                if self.state.selected == Some(sq) {
                    painter.rect_filled(sq_rect, 0.0, SEL);
                } else if self.state.legal_hints.contains(&sq) {
                    let center = sq_rect.center();
                    if pieces[sq as usize].is_some() {
                        painter.rect_stroke(sq_rect, 0.0, Stroke::new(3.0, SEL));
                    } else {
                        painter.circle_filled(center, sq_size * 0.15, HINT);
                    }
                }

                if let Some((color, piece)) = pieces[sq as usize] {
                    draw_piece(&painter, sq_rect, color, piece);
                }
            }
        }

        draw_coords(&painter, rect, sq_size, self.flip);

        if self.interactive {
            if let Some(pos) = response.interact_pointer_pos() {
                if response.clicked() {
                    let file = ((pos.x - rect.left()) / sq_size) as u8;
                    let rank_draw = ((pos.y - rect.top()) / sq_size) as u8;
                    let (file, rank) = if self.flip {
                        (7 - file, rank_draw)
                    } else {
                        (file, 7 - rank_draw)
                    };
                    if file < 8 && rank < 8 {
                        let clicked_sq = rank * 8 + file;
                        return Some((clicked_sq, clicked_sq));
                    }
                }
            }
        }

        None
    }
}

fn draw_coords(painter: &Painter, rect: Rect, sq_size: f32, flip: bool) {
    let font = egui::FontId::proportional(sq_size * 0.18);
    for i in 0u8..8 {
        let rank_char = if flip { (b'1' + i) as char } else { (b'8' - i) as char };
        let file_char = if flip { (b'h' - i) as char } else { (b'a' + i) as char };
        let is_light_rank = if flip { i % 2 == 0 } else { i % 2 == 1 };
        let is_light_file = if flip { i % 2 == 1 } else { i % 2 == 0 };
        let rank_color = if is_light_rank { DARK } else { LIGHT };
        let file_color = if is_light_file { DARK } else { LIGHT };

        painter.text(
            Pos2::new(rect.left() + 2.0, rect.top() + i as f32 * sq_size + 2.0),
            egui::Align2::LEFT_TOP,
            rank_char,
            font.clone(),
            rank_color,
        );
        painter.text(
            Pos2::new(rect.left() + (i as f32 + 1.0) * sq_size - 2.0, rect.bottom() - 2.0),
            egui::Align2::RIGHT_BOTTOM,
            file_char,
            font.clone(),
            file_color,
        );
    }
}

fn draw_piece(painter: &Painter, rect: Rect, color: bool, piece: u8) {
    let symbol = match (color, piece) {
        (true,  0) => "♙", (true,  1) => "♘", (true,  2) => "♗",
        (true,  3) => "♖", (true,  4) => "♕", (true,  5) => "♔",
        (false, 0) => "♟", (false, 1) => "♞", (false, 2) => "♝",
        (false, 3) => "♜", (false, 4) => "♛", (false, 5) => "♚",
        _ => "?",
    };

    let outline_color = if color { Color32::from_rgb(30, 30, 30) } else { Color32::from_rgb(220, 220, 220) };
    let fill_color    = if color { Color32::WHITE } else { Color32::from_rgb(20, 20, 20) };

    let font = egui::FontId::proportional(rect.width() * 0.72);
    let center = rect.center();

    for dx in [-1.0f32, 0.0, 1.0] {
        for dy in [-1.0f32, 0.0, 1.0] {
            if dx != 0.0 || dy != 0.0 {
                painter.text(
                    Pos2::new(center.x + dx, center.y + dy),
                    egui::Align2::CENTER_CENTER,
                    symbol,
                    font.clone(),
                    outline_color,
                );
            }
        }
    }
    painter.text(center, egui::Align2::CENTER_CENTER, symbol, font, fill_color);
}

fn blend(base: Color32, overlay: Color32) -> Color32 {
    let a = overlay.a() as f32 / 255.0;
    Color32::from_rgb(
        (base.r() as f32 * (1.0 - a) + overlay.r() as f32 * a) as u8,
        (base.g() as f32 * (1.0 - a) + overlay.g() as f32 * a) as u8,
        (base.b() as f32 * (1.0 - a) + overlay.b() as f32 * a) as u8,
    )
}

fn parse_fen(fen: &str) -> [Option<(bool, u8)>; 64] {
    let mut board = [None; 64];
    let board_part = fen.split_whitespace().next().unwrap_or("");
    let mut sq: usize = 56;

    for ch in board_part.chars() {
        match ch {
            '/' => { sq = sq.saturating_sub(16); }
            '1'..='8' => { sq += (ch as u8 - b'0') as usize; }
            _ => {
                let white = ch.is_uppercase();
                let piece = match ch.to_ascii_lowercase() {
                    'p' => 0, 'n' => 1, 'b' => 2,
                    'r' => 3, 'q' => 4, 'k' => 5,
                    _ => { sq += 1; continue; }
                };
                if sq < 64 { board[sq] = Some((white, piece)); }
                sq += 1;
            }
        }
    }
    board
}

pub fn sq_to_uci(sq: u8) -> String {
    format!("{}{}", (b'a' + sq % 8) as char, (b'1' + sq / 8) as char)
}

pub fn piece_at_fen(fen: &str, sq: u8) -> Option<(bool, u8)> {
    parse_fen(fen)[sq as usize]
}

pub fn side_to_move_white(fen: &str) -> bool {
    fen.split_whitespace().nth(1).map_or(true, |s| s == "w")
}