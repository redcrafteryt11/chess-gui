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

#[derive(Debug, Clone)]
pub struct GameState {
    pub fen: String,
    pub moves: Vec<String>,
    pub mode: GameMode,
    pub human_color: PlayerColor,
    pub side_to_move: PlayerColor,
    pub status: GameStatus,
    pub analysis_lines: Vec<String>,
    pub current_depth: u32,
    pub current_score: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameStatus {
    Playing,
    Checkmate(PlayerColor),
    Stalemate,
    Waiting,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
            moves: Vec::new(),
            mode: GameMode::HumanVsEngine,
            human_color: PlayerColor::White,
            side_to_move: PlayerColor::White,
            status: GameStatus::Playing,
            analysis_lines: Vec::new(),
            current_depth: 0,
            current_score: 0,
        }
    }

    pub fn position_string(&self) -> String {
        if self.moves.is_empty() {
            format!("position fen {}", self.fen)
        } else {
            format!("position fen {} moves {}", self.fen, self.moves.join(" "))
        }
    }

    pub fn push_move(&mut self, mv: &str) {
        self.moves.push(mv.to_string());
        self.side_to_move = match self.side_to_move {
            PlayerColor::White => PlayerColor::Black,
            PlayerColor::Black => PlayerColor::White,
        };
    }

    pub fn reset(&mut self) {
        self.moves.clear();
        self.fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string();
        self.side_to_move = PlayerColor::White;
        self.status = GameStatus::Playing;
        self.analysis_lines.clear();
        self.current_depth = 0;
        self.current_score = 0;
    }

    pub fn set_fen(&mut self, fen: &str) {
        self.fen = fen.to_string();
        self.moves.clear();
        self.side_to_move = if fen.contains(" b ") {
            PlayerColor::Black
        } else {
            PlayerColor::White
        };
        self.status = GameStatus::Playing;
        self.analysis_lines.clear();
    }

    pub fn is_human_turn(&self) -> bool {
        match self.mode {
            GameMode::Analysis => true,
            GameMode::EngineVsEngine => false,
            GameMode::HumanVsEngine => self.side_to_move == self.human_color,
        }
    }
}