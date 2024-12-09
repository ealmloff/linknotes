/// The main function initializes a tracing subscriber and then runs the contextual_lib module.
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tracing_subscriber::fmt::init(); // Initialize a tracing subscriber.
    contextual_lib::run() // Run the contextual_lib module.
}
