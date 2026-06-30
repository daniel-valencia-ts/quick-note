use ratatui::crossterm::event::{self, KeyEvent};

use crate::utils::{
    AppState,
    Mode::{self, *},
    Note, RelevanceLevel,
};

use crate::Result;

pub fn get_notes(app: &mut AppState) -> Result<Option<Vec<Note>>> {
    let notes: Vec<Note> = app.file_handler.deserialize_notes()?;
    // Return the notes that match the input provided by the user so far, either in
    // their title or numerical id.
    let query: Vec<Note> = notes
        .into_iter()
        .rev()
        .filter(|x| {
            x.title
                .to_lowercase()
                .starts_with(&app.text_areas.search_bar.lines()[0].to_lowercase())
                || x.id
                    .to_string()
                    .to_lowercase()
                    .starts_with(&app.text_areas.search_bar.lines()[0].to_lowercase())
        })
        .collect();
    match query.len() {
        n if n >= 1 => Ok(Some(query)),
        _ => Ok(None),
    }
}

pub fn execute_command(app: &mut AppState) -> Result<()> {
    match app.text_areas.command_prompt.lines()[0].as_str() {
        "select" | "add" | "remove" | "modify" | "display" => {
            let new_mode = Mode::from(&app.text_areas.command_prompt.lines()[0]).unwrap();
            change_mode(app, new_mode)?;
        }
        "exit" => app.exit = true,
        _ => {}
    }
    app.text_areas.command_prompt.clear();
    Ok(())
}

pub fn change_mode(app: &mut AppState, new_mode: Mode) -> Result<()> {
    // Change from the current mode to the target one, executing a specific
    // set of actions based on the former.
    match &app.current_mode {
        Select => {
            if let Some(index) = app.selected_item.selected() {
                if let Some(notes) = &app.notes {
                    if index < notes.len() {
                        app.current_note = Some(notes[index].clone());
                    }
                }
            }
            app.selected_item.select(None);
            app.text_areas.search_bar.clear();

            app.notes = get_notes(app)?;
        }
        Remove => {
            if let Some(index) = app.selected_item.selected() {
                if let Some(notes) = &app.notes {
                    if index < notes.len() {
                        let target_id = notes[index].id;
                        app.file_handler.remove_note(target_id)?;

                        if let Some(note) = &app.current_note {
                            if note.id == target_id {
                                app.current_note = None;
                            }
                        }
                    }
                }
            }
            app.selected_item.select(None);
            app.text_areas.search_bar.clear();

            app.notes = get_notes(app)?;
        }
        AddNoteRelevance => {
            if let Some(index) = app.selected_item.selected() {
                let new_note = Note::new(
                    String::from(app.text_areas.note_body.lines().join("\n")),
                    String::from(app.text_areas.note_title.lines().join("\n")),
                    RelevanceLevel::VARIANTS[index].clone(),
                );
                app.file_handler.add_note(new_note)?;
                app.text_areas.note_title.clear();
                app.text_areas.note_body.clear();
                app.selected_item.select(None);

                app.notes = get_notes(app)?;
                if app.current_note.is_none() {
                    app.current_note = Some(app.notes.as_ref().unwrap()[0].clone());
                }
            }
        }
        ModifyNoteSelect => {
            if let Some(index) = app.selected_item.selected() {
                if let Some(notes) = &app.notes {
                    if index < notes.len() {
                        app.current_note = Some(notes[index].clone());
                    }
                }
            } else {
                return Ok(());
            }
            app.selected_item.select(None);
            app.text_areas.search_bar.clear();

            app.text_areas
                .note_title
                .insert_str(app.current_note.as_ref().unwrap().title.as_str());
            app.text_areas
                .note_body
                .insert_str(app.current_note.as_ref().unwrap().body.as_str());
        }
        ModifyNoteBody => {
            let modified_note = Note::new(
                String::from(app.text_areas.note_body.lines().join("\n")),
                String::from(app.text_areas.note_title.lines().join("\n")),
                app.current_note.as_ref().unwrap().relevance.clone(),
            );

            let target_id = app.current_note.as_ref().unwrap().id;
            app.file_handler.remove_note(target_id)?;

            app.file_handler.add_note(modified_note)?;
            app.text_areas.note_title.clear();
            app.text_areas.note_body.clear();

            app.notes = get_notes(app)?;
            app.current_note = Some(app.notes.as_ref().unwrap()[0].clone());
        }
        Display => app.display_offset = 0,
        _ => {}
    }

    app.current_mode = new_mode;
    Ok(())
}

fn quit_mode(app: &mut AppState) -> Result<()> {
    match app.current_mode {
        AddNoteTitle | AddNoteBody | AddNoteRelevance | ModifyNoteTitle | ModifyNoteBody => {
            app.text_areas.note_title.clear();
            app.text_areas.note_body.clear();
            app.selected_item.select(None);
            app.current_mode = Display;
        }
        Select | Remove | ModifyNoteSelect => {
            app.selected_item.select(None);
            app.text_areas.search_bar.clear();
            app.notes = get_notes(app)?;
            app.current_mode = Display;
        }
        Display => app.current_note = None,
    }

    Ok(())
}

pub fn handle_input(app: &mut AppState, key: KeyEvent) -> Result<()> {
    if key.code == event::KeyCode::Esc {
        app.exit = true;
    } else if key.code == event::KeyCode::Char('c')
        && key.modifiers.contains(event::KeyModifiers::CONTROL)
    {
        quit_mode(app)?;
    } else {
        match &app.current_mode {
            Select => match key.code {
                event::KeyCode::Down => app.selected_item.select_next(),
                event::KeyCode::Up => app.selected_item.select_previous(),

                event::KeyCode::Char(_)
                | event::KeyCode::Backspace
                | event::KeyCode::Left
                | event::KeyCode::Right => {
                    app.text_areas.search_bar.input(key);
                    app.notes = get_notes(app)?;
                }
                event::KeyCode::Enter => {
                    change_mode(app, Display)?;
                }
                _ => {}
            },
            AddNoteTitle => {
                if key.code == event::KeyCode::Char('j')
                    && key.modifiers.contains(event::KeyModifiers::CONTROL)
                {
                    if app.text_areas.note_title.lines()[0].len() >= 1 {
                        change_mode(app, AddNoteBody)?;
                    }
                } else {
                    match key.code {
                        event::KeyCode::Char(_)
                        | event::KeyCode::Backspace
                        | event::KeyCode::Left
                        | event::KeyCode::Right => {
                            app.text_areas.note_title.input(key);
                        }
                        _ => {}
                    }
                }
            }

            AddNoteBody => {
                if key.code == event::KeyCode::Char('j')
                    && key.modifiers.contains(event::KeyModifiers::CONTROL)
                {
                    if app
                        .text_areas
                        .note_body
                        .lines()
                        .iter()
                        .map(|line| line.len())
                        .sum::<usize>()
                        >= 1
                    {
                        app.selected_item.select(Some(0));
                        change_mode(app, AddNoteRelevance)?;
                    }
                } else {
                    app.text_areas.note_body.input(key);
                }
            }
            AddNoteRelevance => match key.code {
                event::KeyCode::Down => app.selected_item.select_next(),
                event::KeyCode::Up => app.selected_item.select_previous(),
                event::KeyCode::Enter => {
                    change_mode(app, Display)?;
                }
                _ => {}
            },
            Remove => match key.code {
                event::KeyCode::Down => app.selected_item.select_next(),
                event::KeyCode::Up => app.selected_item.select_previous(),

                event::KeyCode::Char(_)
                | event::KeyCode::Backspace
                | event::KeyCode::Left
                | event::KeyCode::Right => {
                    app.text_areas.search_bar.input(key);
                    app.notes = get_notes(app)?;
                }
                event::KeyCode::Enter => {
                    change_mode(app, Display)?;
                }
                _ => {}
            },
            ModifyNoteSelect => match key.code {
                event::KeyCode::Down => app.selected_item.select_next(),
                event::KeyCode::Up => app.selected_item.select_previous(),

                event::KeyCode::Char(_)
                | event::KeyCode::Backspace
                | event::KeyCode::Left
                | event::KeyCode::Right => {
                    app.text_areas.search_bar.input(key);
                    app.notes = get_notes(app)?;
                }
                event::KeyCode::Enter => {
                    change_mode(app, ModifyNoteTitle)?;
                }
                _ => {}
            },
            ModifyNoteTitle => {
                if key.code == event::KeyCode::Char('j')
                    && key.modifiers.contains(event::KeyModifiers::CONTROL)
                {
                    if app.text_areas.note_title.lines()[0].len() >= 1 {
                        change_mode(app, ModifyNoteBody)?;
                    }
                } else {
                    match key.code {
                        event::KeyCode::Char(_)
                        | event::KeyCode::Backspace
                        | event::KeyCode::Left
                        | event::KeyCode::Right => {
                            app.text_areas.note_title.input(key);
                        }
                        _ => {}
                    }
                }
            }
            ModifyNoteBody => {
                if key.code == event::KeyCode::Char('j')
                    && key.modifiers.contains(event::KeyModifiers::CONTROL)
                {
                    if app
                        .text_areas
                        .note_body
                        .lines()
                        .iter()
                        .map(|line| line.len())
                        .sum::<usize>()
                        >= 1
                    {
                        change_mode(app, Display)?;
                    }
                } else {
                    app.text_areas.note_body.input(key);
                }
            }
            Display => match key.code {
                event::KeyCode::Down => app.display_offset += 1,
                event::KeyCode::Up => {
                    if app.display_offset >= 1 {
                        app.display_offset -= 1
                    }
                }

                event::KeyCode::Char(_)
                | event::KeyCode::Backspace
                | event::KeyCode::Left
                | event::KeyCode::Right => {
                    app.text_areas.command_prompt.input(key);
                }
                event::KeyCode::Enter => {
                    execute_command(app)?;
                }
                _ => {}
            },
        }
    }
    Ok(())
}
