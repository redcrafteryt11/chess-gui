use egui::{CentralPanel, Color32, Context, RichText, ScrollArea, SidePanel, TopBottomPanel};
use tokio::runtime::Runtime;
use crate::board::{BoardState, BoardWidget, piece_at_fen, side_to_move_white, sq_to_uci};
use crate::game::{GameMode, GameState, GameStatus};
use crate::uci::{UciEngine, UciOutput};

pub struct ChessApp {
    rt: Runtime,
    engine: Option<UciEngine>,
    engine_path: String,
    game: GameState,
    board: BoardState,
    selected_sq: Option<u8>,
    fen_input: String,
    depth: u32,
    engine_thinking: bool,
    move_history: Vec<String>,
    log: Vec<String>,
}

impl ChessApp {
    pub fn new() -> Self {
        let rt = Runtime::new().unwrap();
        Self {
            rt,
            engine: None,
            engine_path: String::from("ChessEngine.exe"),
            game: GameState::new(),
            board: BoardState::new(),
            selected_sq: None,
            fen_input: String::new(),
            depth: 6,
            engine_thinking: false,
            move_history: Vec::new(),
            log: Vec::new(),
        }
    }

    fn connect_engine(&mut self) {
        match self.rt.block_on(async {
            UciEngine::launch(&self.engine_path)
        }) {
            Ok(engine) => {
                engine.init();
                self.engine = Some(engine);
                self.log.push("Engine connected".to_string());
            }
            Err(e) => {
                self.log.push(format!("Failed to launch engine: {e}"));
            }
        }
    }

    fn poll_engine(&mut self) {
        let Some(engine) = &mut self.engine else { return };
        let mut messages = Vec::new();
        while let Ok(msg) = engine.output_rx.try_recv() {
            messages.push(msg);
        }
        for msg in messages {
            match msg {
                UciOutput::BestMove(mv) => {
                    if mv != "0000" && self.engine_thinking {
                        self.apply_move(&mv);
                    }
                    self.engine_thinking = false;
                }
                UciOutput::Info { depth, score, pv } => {
                    self.game.current_depth = depth;
                    self.game.current_score = score;
                    if let Some(line) = pv {
                        let info = format!("depth {depth} score {score:+} pv {line}");
                        if self.game.analysis_lines.last().map_or(true, |l| l != &info) {
                            self.game.analysis_lines.push(info);
                            if self.game.analysis_lines.len() > 20 {
                                self.game.analysis_lines.remove(0);
                            }
                        }
                    }
                }
                UciOutput::Ready => {}
            }
        }
    }

    fn apply_move(&mut self, mv: &str) {
        self.game.push_move(mv);
        self.board.last_move = Some((
            sq_from_uci(&mv[..2]),
            sq_from_uci(&mv[2..4]),
        ));
        self.move_history.push(mv.to_string());
        self.selected_sq = None;
        self.board.selected = None;
        self.board.legal_hints.clear();
    }

    fn request_engine_move(&mut self, depth: u32) {
        let Some(engine) = &self.engine else {
            self.log.push("No engine connected".to_string());
            return;
        };
        engine.go(&self.game.position_string(), depth);
        self.engine_thinking = true;
    }

    fn handle_square_click(&mut self, sq: u8) {
        if !self.game.is_human_turn() { return; }

        let white_turn = side_to_move_white(&fen_after_moves(&self.game));

        match self.selected_sq {
            Some(from) if from == sq => {
                self.selected_sq = None;
                self.board.selected = None;
                self.board.legal_hints.clear();
            }
            Some(from) => {
                let mv = sq_to_uci(from) + &sq_to_uci(sq);
                let mv = maybe_add_promotion(&mv, from, sq, white_turn, &self.game);
                self.apply_move(&mv);
                self.game.analysis_lines.clear();

                if !self.engine_thinking {
                    match self.game.mode {
                        GameMode::HumanVsEngine => self.request_engine_move(self.depth),
                        GameMode::Analysis => {
                            if let Some(engine) = &self.engine {
                                engine.go(&self.game.position_string(), self.depth);
                                self.engine_thinking = true;
                            }
                        }
                        _ => {}
                    }
                }
            }
            None => {
                let piece = piece_at_fen(&fen_after_moves(&self.game), sq);
                if piece.map_or(false, |(w, _)| w == white_turn) {
                    self.selected_sq = Some(sq);
                    self.board.selected = Some(sq);
                    self.board.legal_hints.clear();
                }
            }
        }
    }
}

impl eframe::App for ChessApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.poll_engine();

        if self.engine_thinking || matches!(self.game.mode, GameMode::EngineVsEngine) {
            ctx.request_repaint();
        }

        TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Engine:");
                ui.text_edit_singleline(&mut self.engine_path);
                if ui.button("Browse…").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Executable", &["exe", ""])
                        .set_title("Select chess engine executable")
                        .pick_file()
                    {
                        self.engine_path = path.to_string_lossy().to_string();
                    }
                }
                if ui.button("Connect").clicked() {
                    self.connect_engine();
                }
                ui.separator();
                if ui.button("New game").clicked() {
                    self.game.reset();
                    self.move_history.clear();
                    self.board = BoardState::new();
                    self.selected_sq = None;
                    self.engine_thinking = false;
                    if let Some(e) = &self.engine { e.new_game(); }
                }
                ui.separator();
                ui.label("Mode:");
                egui::ComboBox::from_id_salt("mode")
                    .selected_text(mode_label(&self.game.mode))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.game.mode, GameMode::HumanVsEngine, "Human vs Engine");
                        ui.selectable_value(&mut self.game.mode, GameMode::EngineVsEngine, "Engine vs Engine");
                        ui.selectable_value(&mut self.game.mode, GameMode::Analysis, "Analysis");
                    });
                ui.separator();
                ui.label("Depth:");
                ui.add(egui::Slider::new(&mut self.depth, 1..=20));
                ui.separator();
                if ui.button("Flip").clicked() {
                    self.board.flipped = !self.board.flipped;
                }
            });
        });

        SidePanel::right("analysis").min_width(220.0).show(ctx, |ui| {
            ui.heading("Analysis");
            ui.separator();

            let score = self.game.current_score;
            let depth = self.game.current_depth;
            let thinking = self.engine_thinking;

            ui.horizontal(|ui| {
                ui.label("Eval:");
                let color = if score > 0 { Color32::GREEN } else if score < 0 { Color32::RED } else { Color32::GRAY };
                ui.label(RichText::new(format!("{score:+}")).color(color));
                ui.label(format!("  depth {depth}"));
                if thinking { ui.spinner(); }
            });

            ui.separator();
            ui.label("PV lines:");
            ScrollArea::vertical().id_salt("pv").max_height(200.0).show(ui, |ui| {
                for line in self.game.analysis_lines.iter().rev().take(10) {
                    ui.label(RichText::new(line).monospace().size(11.0));
                }
            });

            ui.separator();
            ui.label("Move history:");
            ScrollArea::vertical().id_salt("hist").max_height(200.0).show(ui, |ui| {
                for (i, mv) in self.move_history.iter().enumerate() {
                    if i % 2 == 0 {
                        ui.label(RichText::new(format!("{}. {}", i / 2 + 1, mv)).monospace());
                    } else {
                        ui.label(RichText::new(format!("   {mv}")).monospace());
                    }
                }
            });

            ui.separator();
            ui.label("FEN:");
            ui.text_edit_singleline(&mut self.fen_input);
            if ui.button("Set position").clicked() {
                let fen = self.fen_input.clone();
                self.game.set_fen(&fen);
                self.board = BoardState::new();
                self.move_history.clear();
                self.selected_sq = None;
                self.engine_thinking = false;
                if let Some(e) = &self.engine { e.new_game(); }
            }

            ui.separator();
            if ui.button("Go (engine move)").clicked() {
                self.request_engine_move(self.depth);
            }

            ui.separator();
            ui.label("Log:");
            ScrollArea::vertical().id_salt("log").max_height(100.0).stick_to_bottom(true).show(ui, |ui| {
                for entry in &self.log {
                    ui.label(RichText::new(entry).size(11.0));
                }
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            let interactive = self.game.is_human_turn() && self.game.status == GameStatus::Playing;
            let flip = self.board.flipped;
            let fen = fen_after_moves(&self.game);

            let mut widget = BoardWidget {
                fen: &fen,
                state: &mut self.board,
                interactive,
                flip,
            };

            if let Some((sq, _)) = widget.show(ui) {
                self.handle_square_click(sq);
            }
        });

        if matches!(self.game.mode, GameMode::EngineVsEngine)
            && !self.engine_thinking
            && self.game.status == GameStatus::Playing
        {
            self.request_engine_move(self.depth);
        }
    }
}

fn mode_label(mode: &GameMode) -> &'static str {
    match mode {
        GameMode::HumanVsEngine  => "Human vs Engine",
        GameMode::EngineVsEngine => "Engine vs Engine",
        GameMode::Analysis       => "Analysis",
    }
}

fn sq_from_uci(s: &str) -> u8 {
    let b = s.as_bytes();
    let file = b[0] - b'a';
    let rank = b[1] - b'1';
    rank * 8 + file
}

fn fen_after_moves(game: &GameState) -> String {
    game.fen.clone()
}

fn maybe_add_promotion(mv: &str, from: u8, to: u8, white_turn: bool, game: &GameState) -> String {
    let from_rank = from / 8;
    let to_rank = to / 8;
    let is_pawn = piece_at_fen(&game.fen, from).map_or(false, |(_, p)| p == 0);
    let is_promotion = is_pawn && ((white_turn && from_rank == 6 && to_rank == 7)
        || (!white_turn && from_rank == 1 && to_rank == 0));
    if is_promotion {
        format!("{mv}q")
    } else {
        mv.to_string()
    }
}