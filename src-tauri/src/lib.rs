pub mod commands;
pub mod domain;
pub mod error;
pub mod events;
pub mod state;
pub mod storage;

use state::SharedState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(SharedState::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::tournament::create_tournament,
            commands::tournament::load_tournament,
            commands::tournament::save_tournament_as,
            commands::tournament::close_tournament,
            commands::tournament::get_tournament,
            commands::tournament::rename_tournament,
            commands::team::add_team,
            commands::team::rename_team,
            commands::team::remove_team,
            commands::round::generate_next_round,
            commands::round::reassign_table,
            commands::match_cmd::start_match,
            commands::match_cmd::start_all_assigned_matches,
            commands::match_cmd::submit_result,
            commands::match_cmd::edit_result,
            commands::match_cmd::cancel_match,
            commands::match_cmd::pause_match_timer,
            commands::match_cmd::resume_match_timer,
            commands::match_cmd::set_match_time_limit,
            commands::playoffs::start_playoffs,
            commands::playoffs::start_playoff_match,
            commands::playoffs::cancel_playoff_match,
            commands::playoffs::submit_playoff_result,
            commands::standings::get_standings,
            commands::standings::get_balance_report,
            commands::window::open_projector,
            commands::window::toggle_fullscreen_projector,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
