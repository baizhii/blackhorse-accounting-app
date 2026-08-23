mod db;
mod models;

use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::Manager;

/// 全局数据库连接与数据文件路径
pub struct Db {
    pub conn: Mutex<Connection>,
    pub path: PathBuf,
}

#[tauri::command]
fn list_categories(state: tauri::State<'_, Db>) -> Result<Vec<models::Category>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::list_categories(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_expenses(state: tauri::State<'_, Db>, month: String) -> Result<models::MonthSummary, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::list_expenses(&conn, &month)
}

#[tauri::command]
fn create_expense(
    state: tauri::State<'_, Db>,
    payload: models::NewExpense,
) -> Result<models::Expense, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::create_expense(&conn, &payload)
}

#[tauri::command]
fn update_expense(
    state: tauri::State<'_, Db>,
    id: i64,
    payload: models::NewExpense,
) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::update_expense(&conn, id, &payload)
}

#[tauri::command]
fn delete_expense(state: tauri::State<'_, Db>, id: i64) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::delete_expense(&conn, id)
}

#[tauri::command]
fn add_category(
    state: tauri::State<'_, Db>,
    name: String,
    parent_id: Option<i64>,
) -> Result<models::Category, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::add_category(&conn, &name, parent_id)
}

#[tauri::command]
fn rename_category(state: tauri::State<'_, Db>, id: i64, name: String) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::rename_category(&conn, id, &name)
}

#[tauri::command]
fn delete_category(state: tauri::State<'_, Db>, id: i64) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::delete_category(&conn, id)
}

#[tauri::command]
fn month_trend(
    state: tauri::State<'_, Db>,
    end_month: String,
) -> Result<Vec<models::MonthTotal>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::month_trend(&conn, &end_month, 12)
}

#[tauri::command]
fn category_stats(
    state: tauri::State<'_, Db>,
    month: String,
) -> Result<Vec<models::CategoryStat>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::category_stats(&conn, &month)
}

#[tauri::command]
fn child_ranking(
    state: tauri::State<'_, Db>,
    month: String,
    limit: i64,
) -> Result<Vec<models::CategoryStat>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::child_ranking(&conn, &month, limit)
}

#[tauri::command]
fn year_report(
    state: tauri::State<'_, Db>,
    year: String,
) -> Result<models::YearReport, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::year_report(&conn, &year)
}

#[tauri::command]
fn data_file_path(state: tauri::State<'_, Db>) -> String {
    state.path.display().to_string()
}

#[tauri::command]
fn export_expenses_csv(
    state: tauri::State<'_, Db>,
    path: String,
    month: Option<String>,
) -> Result<usize, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::export_expenses_csv(&conn, Path::new(&path), month.as_deref())
}

#[tauri::command]
fn backup_db(state: tauri::State<'_, Db>, target: String) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::backup_db(&conn, Path::new(&target))
}

#[tauri::command]
fn restore_db(state: tauri::State<'_, Db>, source: String) -> Result<(), String> {
    let mut conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::restore_db(&mut conn, &state.path, Path::new(&source))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // 数据文件存放于系统应用数据目录（如 %APPDATA%\com.blackhorse.accounting）
            let data_dir = app.path().app_data_dir().expect("无法解析应用数据目录");
            std::fs::create_dir_all(&data_dir).expect("无法创建应用数据目录");
            let db_path = data_dir.join("blackhorse.db");
            let conn = db::init(&db_path).expect("数据库初始化失败");
            app.manage(Db {
                conn: Mutex::new(conn),
                path: db_path,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_categories,
            list_expenses,
            create_expense,
            update_expense,
            delete_expense,
            add_category,
            rename_category,
            delete_category,
            month_trend,
            category_stats,
            child_ranking,
            year_report,
            data_file_path,
            export_expenses_csv,
            backup_db,
            restore_db
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
