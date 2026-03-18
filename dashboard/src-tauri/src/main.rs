// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Trigger tauri-clippy CI job to validate linter output
fn main() {
  app_lib::run();
}
