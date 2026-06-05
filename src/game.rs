#[derive(Debug, Clone, PartialEq)]
pub enum GameMode {
    HumanVsEngine,
    EngineVsEngine,
    Analysis,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlayerColor {
    White,
    Black,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameStatus {
    Playing,
    Checkmate(PlayerColor),
    Stalemate,
    Waiting,
}

#[derive(Debug, Clone)]
pub struct GameState {
    pub start_fen: String,
    pub current_fen: String,
    pub moves: Vec<String>,
    pub mode: GameMode,
    pub human_color: PlayerColor,
    pub status: GameStatus,
    pub analysis_lines: Vec<String>,
    pub current_depth: u32,
    pub current_score: i32,
}

impl GameState {
    pub fn new() -> Self {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string();
        Self {
            start_fen: fen.clone(),
            current_fen: fen,
            moves: Vec::new(),
            mode: GameMode::HumanVsEngine,
            human_color: PlayerColor::White,
            status: GameStatus::Playing,
            analysis_lines: Vec::new(),
            current_depth: 0,
            current_score: 0,
        }
    }

    pub fn position_string(&self) -> String {
        if self.moves.is_empty() {
            format!("position fen {}", self.start_fen)
        } else {
            format!("position fen {} moves {}", self.start_fen, self.moves.join(" "))
        }
    }

    pub fn side_to_move_white(&self) -> bool {
        self.current_fen.split_whitespace().nth(1).map_or(true, |s| s == "w")
    }

    pub fn push_move(&mut self, mv: &str, new_fen: String) {
        self.moves.push(mv.to_string());
        self.current_fen = new_fen;
    }

    pub fn reset(&mut self) {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string();
        self.start_fen = fen.clone();
        self.current_fen = fen;
        self.moves.clear();
        self.status = GameStatus::Playing;
        self.analysis_lines.clear();
        self.current_depth = 0;
        self.current_score = 0;
    }

    pub fn set_fen(&mut self, fen: &str) {
        self.start_fen = fen.to_string();
        self.current_fen = fen.to_string();
        self.moves.clear();
        self.status = GameStatus::Playing;
        self.analysis_lines.clear();
    }

    pub fn is_human_turn(&self) -> bool {
        match self.mode {
            GameMode::Analysis => true,
            GameMode::EngineVsEngine => false,
            GameMode::HumanVsEngine => {
                let white = self.side_to_move_white();
                match self.human_color {
                    PlayerColor::White => white,
                    PlayerColor::Black => !white,
                }
            }
        }
    }
}