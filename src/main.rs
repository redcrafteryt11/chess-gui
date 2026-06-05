mod app;
mod board;
mod game;
mod uci;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Chess")
            .with_inner_size([900.0, 620.0]),
        ..Default::default()
    };
    eframe::run_native("Chess", options, Box::new(|_cc| Ok(Box::new(app::ChessApp::new()))))
}