//! SP Football Life 2027 Playtools - native Linux / Steam Deck (iced).

mod theme;
mod ui;

use spfl27::core;

fn main() -> iced::Result {
    let args: Vec<String> = std::env::args().collect();
    let play = args.iter().any(|a| a == "--play");

    if play {
        // Headless mode for the Steam shortcut: launch straight into the game.
        let mut game = core::detect_game_dir();
        if let Some(i) = args.iter().position(|a| a == "--game-dir") {
            if let Some(p) = args.get(i + 1) {
                let p = std::path::PathBuf::from(p);
                if core::game_status(Some(&p)).game {
                    game = Some(p);
                }
            }
        }
        let Some(g) = game else {
            eprintln!("ERROR: game folder not found.");
            std::process::exit(1);
        };
        if !core::game_status(Some(&g)).wine {
            if core::game_status(Some(&g)).lutris {
                eprintln!("No system wine — launching through Lutris.");
                if let Err(e) = core::launch_via_lutris() {
                    eprintln!("ERROR: {e}");
                    std::process::exit(1);
                }
                return Ok(());
            }
            eprintln!("ERROR: wine is missing.");
            std::process::exit(1);
        }
        if let Err(e) = core::launch_game(&g) {
            eprintln!("ERROR: {e}");
            std::process::exit(1);
        }
        return Ok(());
    }

    iced::application(ui::App::new, ui::App::update, ui::App::view)
        .title(|_app: &ui::App| format!("{} v{}", ui::BRAND, env!("CARGO_PKG_VERSION")))
        .theme(|app: &ui::App| app.theme())
        .subscription(ui::App::subscription)
        .window(iced::window::Settings {
            // Content caps at 960px of page + 208 sidebar + 2x28 gutter = 1224,
            // so the default window sits just past it. Below that the page goes
            // fluid instead of clipping. 1280x720 (Steam Deck) has headroom.
            size: iced::Size::new(1264.0, 780.0),
            min_size: Some(iced::Size::new(1000.0, 620.0)),
            ..Default::default()
        })
        .run()
}
