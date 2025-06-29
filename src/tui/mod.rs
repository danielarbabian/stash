pub mod app;
pub mod components;
pub mod handlers;
pub mod state;

pub use app::App;

pub fn run_tui() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();
    app.run()
}