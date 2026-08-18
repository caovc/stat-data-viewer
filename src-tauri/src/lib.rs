mod commands;
mod export;
mod ingest;
mod query;
mod sqlutil;
mod state;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = AppState::new().expect("failed to create session database");
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::open_dataset,
            commands::reimport,
            commands::cancel_import,
            commands::query_page_cmd,
            commands::run_sql,
            commands::export,
            commands::column_distinct_cmd,
            commands::get_metadata,
            commands::list_datasets,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Stat Data Viewer");
}
