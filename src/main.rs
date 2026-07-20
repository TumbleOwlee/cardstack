mod app;
mod command;
mod dialog;
mod filter;
mod focus;
mod model;
mod storage;
mod ui;

use std::io::Stdout;

use app::App;
use ferrowl_ui::AlternateScreen;

/// UI-R-001 — restore the terminal on normal exit, error exit, and panic.
fn main() {
    let handler = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        AlternateScreen::<Stdout>::release();
        handler(panic);
    }));

    let config_dir = storage::config_dir().expect("resolve config directory");
    let _lock = match storage::acquire_lock(&config_dir) {
        Ok(lock) => lock,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };

    let dir = storage::boards_dir().expect("resolve boards directory");
    let mut app = App::load(dir).expect("load boards");

    let mut screen = match AlternateScreen::<Stdout>::new() {
        Ok(screen) => screen,
        Err(e) => {
            eprintln!("Failed to set up screen: {e}");
            return;
        }
    };
    if let Err(e) = app.run(&mut screen) {
        AlternateScreen::<Stdout>::release();
        eprintln!("UI error: {e}");
    }
}
