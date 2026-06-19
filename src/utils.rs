use chrono::Local;
use rand::random;
use ratatui::widgets::ListState;
use ratatui_textarea::TextArea;
use serde::{Deserialize, Serialize};

use crate::file_handling::FileHandler;

// State the values a Note struct can hold in its "relevance" field.
#[derive(Deserialize, Serialize, Clone)]
pub enum RelevanceLevel {
    Normal,
    Relevant,
    Important,
    Crucial,
}
impl RelevanceLevel {
    pub const VARIANTS: [RelevanceLevel; 4] = [
        RelevanceLevel::Normal,
        RelevanceLevel::Relevant,
        RelevanceLevel::Important,
        RelevanceLevel::Crucial,
    ];
}

impl ToString for RelevanceLevel {
    fn to_string(&self) -> String {
        match self {
            RelevanceLevel::Normal => String::from("normal"),
            RelevanceLevel::Relevant => String::from("relevant"),
            RelevanceLevel::Important => String::from("important"),
            RelevanceLevel::Crucial => String::from("crucial"),
        }
    }
}

// Describe all the modes the app can be set to, which modify the
// data shown on screen and the way in which input is processed.
#[derive(PartialEq)]
pub enum Mode {
    Select,
    AddNoteTitle,
    AddNoteBody,
    AddNoteRelevance,
    Remove,
    ModifyNoteSelect,
    ModifyNoteTitle,
    ModifyNoteBody,
    Display,
}

pub fn to_mode(mode_as_string: String) -> Option<Mode> {
    match mode_as_string.as_str() {
        "select" => Some(Mode::Select),
        "add" => Some(Mode::AddNoteTitle),
        "remove" => Some(Mode::Remove),
        "modify" => Some(Mode::ModifyNoteSelect),
        "display" => Some(Mode::Display),
        _ => None,
    }
}

impl ToString for Mode {
    fn to_string(&self) -> String {
        match self {
            Mode::Select => String::from("select"),
            Mode::AddNoteTitle | Mode::AddNoteBody | Mode::AddNoteRelevance => String::from("add"),
            Mode::Remove => String::from("remove"),
            Mode::ModifyNoteSelect | Mode::ModifyNoteTitle | Mode::ModifyNoteBody => {
                String::from("modify")
            }
            Mode::Display => String::from("display"),
        }
    }
}

pub struct AppState<'a> {
    pub exit: bool,
    pub current_mode: Mode,
    pub file_handler: FileHandler,
    pub selected_item: ListState,
    pub current_note: Option<Note>,
    pub display_offset: u16,
    pub notes: Option<Vec<Note>>,
    pub text_areas: TextAreas<'a>,
}

// Keep track of all of the TextArea widgets in the app, in such a way that the
// input provided by the user isn't lost when the screen is re-rendered.
pub struct TextAreas<'a> {
    pub command_prompt: TextArea<'a>,
    pub search_bar: TextArea<'a>,
    pub note_title: TextArea<'a>,
    pub note_body: TextArea<'a>,
}

impl<'a> TextAreas<'a> {
    pub fn new() -> Self {
        TextAreas {
            command_prompt: TextArea::default(),
            search_bar: TextArea::default(),
            note_title: TextArea::default(),
            note_body: TextArea::default(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Note {
    pub id: u32,
    #[serde(rename = "timeStamp")]
    pub timestamp: String,
    pub title: String,
    pub body: String,
    pub relevance: RelevanceLevel,
}

impl Note {
    pub fn new(body: String, title: String, relevance: RelevanceLevel) -> Self {
        Note {
            id: random::<u32>(),
            timestamp: Local::now().format("%Y-%b-%d-%H:%M").to_string(),
            title: title,
            body: body,
            relevance: relevance,
        }
    }
}
