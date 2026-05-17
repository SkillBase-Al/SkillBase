mod api_client;
mod checker;
mod commands;
mod db;
mod dedup;
mod installer;
mod scanner;
mod utils;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let app_dir = app.path().app_local_data_dir().expect("failed to resolve app dir");
            utils::paths::ensure_base_dirs(&app_dir)?;
            let conn = db::repository::init_db(&app_dir).expect("failed to init DB");
            app.manage(conn);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::skill::get_installed_skills,
            commands::skill::install_skill,
            commands::skill::uninstall_skill,
            commands::skill::update_skill,
            commands::skill::toggle_skill_enabled,
            commands::skill::get_skill_content,
            commands::skill::apply_skill_to_agents,
            commands::skill::remove_skill_from_agents,
            commands::skill::scan_local_skills,
            commands::skill::import_skills,
            commands::skill::get_skill_conflicts,
            commands::skill::resolve_skill_conflict,
            commands::assessment::assess_format,
            commands::assessment::assess_security,
            commands::assessment::batch_assess,
            commands::assessment::get_assessment_results,
            commands::dedup::run_dedup,
            commands::dedup::get_dedup_groups,
            commands::dedup::delete_skill_from_group,
            commands::agent::get_agents,
            commands::agent::add_agent,
            commands::agent::update_agent,
            commands::agent::delete_agent,
            commands::market::search_market,
            commands::market::get_skill_detail,
            commands::market::get_categories,
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::feedback::submit_feedback,
            commands::settings::get_server_url,
            commands::settings::check_first_run,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
