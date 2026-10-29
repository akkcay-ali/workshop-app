// Windows'ta sürüm derlemesinde arkada konsol penceresi açılmasın
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    workshop_app_lib::run()
}
