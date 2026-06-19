mod file_handling;
mod helpers;
mod interface;
mod utils;

use directories::ProjectDirs;
use ratatui::{
    crossterm::event::{self, Event},
    widgets::ListState,
};

use std::fs::create_dir_all;

use file_handling::FileHandler;
use helpers::{get_notes, handle_input};
use interface::{configure_text_areas_style, render};
use utils::{AppState, Mode::*, TextAreas};

fn main() {
    let res = run();
    // Return the terminal back to its normal state after the program finishes.
    ratatui::restore();
    match res {
        Ok(_) => {}
        Err(error) => {
            let mut output = String::from("quick-note: ");
            if error.downcast_ref::<std::io::Error>().is_some() {
                output.push_str("an error related to file or render operations took place");
            } else if error.downcast_ref::<serde_json::error::Error>().is_some() {
                output.push_str("an error occurred while trying to deserialize or serialize notes");
            }
            println!("{}", output);
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Get access to the path of the directory where persistent data is stored.
    let path = ProjectDirs::from("", "daniel-valencia-ts", "quick-note")
        .unwrap()
        .data_dir()
        .to_path_buf();
    create_dir_all(&path)?;
    // Create a new struct to keep track of the current state of the program.
    let mut app = AppState {
        exit: false,
        current_mode: Display,
        file_handler: FileHandler::new(path, String::from("notes.json"))?,
        current_note: None,
        selected_item: ListState::default(),
        display_offset: 0,
        notes: None,
        text_areas: TextAreas::new(),
    };

    // Initialize a terminal struct, needed to render the program on screen
    // as well as to activate the raw mode and alternate screen buffer.
    let mut terminal = ratatui::init();
    app.notes = get_notes(&mut app)?;
    if let Some(notes) = &app.notes {
        app.current_note = Some(notes[0].clone());
    }
    // Give text areas their initial configuration.
    configure_text_areas_style(&mut app);

    while !app.exit {
        render(&mut terminal, &mut app)?;

        if let Event::Key(key) = event::read()? {
            // Avoid registering duplicate, unneeded input, since in some cases an event
            // is read both when a key is pressed and once again when it's released.
            if key.is_press() {
                handle_input(&mut app, key)?;
            }
        }
    }

    Ok(())
}
