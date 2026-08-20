mod commands;
mod export;
mod ingest;
mod open_files;
mod query;
mod sqlutil;
mod state;

use open_files::PendingOpenPaths;
use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = AppState::new().expect("failed to create session database");
    let builder = tauri::Builder::default();

    #[cfg(any(windows, target_os = "linux"))]
    let builder = builder.plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
        let files = open_files::paths_from_args(argv.into_iter().skip(1));
        open_files::enqueue(app, files);
    }));

    builder
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .manage(PendingOpenPaths::new())
        .invoke_handler(tauri::generate_handler![
            commands::open_dataset,
            commands::reimport,
            commands::cancel_import,
            commands::drop_dataset,
            commands::query_page_cmd,
            commands::run_sql,
            commands::export,
            commands::column_distinct_cmd,
            commands::get_metadata,
            commands::list_datasets,
            open_files::take_pending_open_paths,
        ])
        .setup(|app| {
            let files = open_files::paths_from_args(std::env::args().skip(1));
            open_files::enqueue(app.handle(), files);
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running Stat Data Viewer")
        .run(|app, event| {
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            if let tauri::RunEvent::Opened { urls } = event {
                let files = urls
                    .into_iter()
                    .filter_map(|url| url.to_file_path().ok())
                    .collect();
                open_files::enqueue(app, files);
            }
        });
}
