use std::{env, vec};

use crate::utils::{
    AppState,
    Mode::*,
    RelevanceLevel::{self, *},
};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, List, ListItem, Padding, Paragraph, Widget, Wrap},
};
use ratatui_textarea::WrapMode::WordOrGlyph;

pub fn configure_text_areas_style(app: &mut AppState) {
    app.text_areas
        .command_prompt
        .set_cursor_line_style(Style::default());
    app.text_areas
        .search_bar
        .set_cursor_line_style(Style::default());
    app.text_areas
        .note_title
        .set_cursor_line_style(Style::default());
    app.text_areas
        .note_body
        .set_cursor_line_style(Style::default());

    app.text_areas.note_title.set_block(
        Block::bordered()
            .border_type(BorderType::Rounded)
            .title(Line::from("Note Title").fg(Color::LightRed))
            .fg(Color::Yellow),
    );

    app.text_areas.note_body.set_block(
        Block::bordered()
            .border_type(BorderType::Rounded)
            .title(Line::from("Note Body").fg(Color::Green))
            .fg(Color::White),
    );

    app.text_areas.note_body.set_wrap_mode(WordOrGlyph);

    app.text_areas
        .command_prompt
        .set_cursor_style(Style::default());
    app.text_areas
        .command_prompt
        .set_style(Style::new().fg(Color::LightYellow).bold());
}

pub fn render(terminal: &mut DefaultTerminal, app: &mut AppState) -> std::io::Result<()> {
    terminal.draw(|f| render_outer_layout(f, app))?;
    Ok(())
}

fn render_outer_layout(frame: &mut Frame, app: &mut AppState) {
    let [program_area] = Layout::default()
        .constraints([Constraint::Fill(1)])
        .margin(1)
        .areas(frame.area());

    let [header, body, footer] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .margin(1)
    .areas(program_area);

    Block::bordered()
        .border_type(BorderType::Rounded)
        .fg(Color::Green)
        .render(program_area, frame.buffer_mut());

    Block::bordered()
        .border_type(BorderType::Rounded)
        .title(Line::from("QuickNote TUI"))
        .fg(Color::White)
        .render(body, frame.buffer_mut());

    let mut header_text = Line::default();
    match &app.current_mode {
        Select => header_text.spans.push(Span::styled(
            "Select.",
            Style::default().fg(Color::LightGreen).underlined().italic(),
        )),
        AddNoteTitle | AddNoteBody | AddNoteRelevance => header_text.spans.push(Span::styled(
            "Add.",
            Style::default()
                .fg(Color::LightYellow)
                .underlined()
                .italic(),
        )),
        Remove => header_text.spans.push(Span::styled(
            "Remove.",
            Style::default().fg(Color::Red).bold().underlined().italic(),
        )),
        ModifyNoteSelect | ModifyNoteTitle | ModifyNoteBody => {
            header_text.spans.push(Span::styled(
                "Modify.",
                Style::default().fg(Color::LightCyan).underlined().italic(),
            ))
        }
        Display => header_text.spans.push(Span::styled(
            "Display.",
            Style::default().fg(Color::LightGreen).underlined().italic(),
        )),
    }
    header_text.spans.push(Span::raw(" "));

    match &app.current_mode {
        Select | Remove | ModifyNoteSelect | AddNoteRelevance => {
            header_text.spans.push(Span::styled(
                "up & down arrows: move, Ctrl + c: quit mode",
                Style::default().fg(Color::LightGreen),
            ))
        }
        AddNoteTitle | AddNoteBody => header_text.spans.push(Span::styled(
            "Ctrl + j: submit, Ctrl + c: quit mode",
            Style::default().fg(Color::Gray).bold(),
        )),
        ModifyNoteTitle | ModifyNoteBody => header_text.spans.push(Span::styled(
            "Ctrl + j: submit, Ctrl + c: quit mode",
            Style::default().fg(Color::LightCyan),
        )),
        Display => header_text.spans.push(Span::styled(
            "commands: select, add, remove, modify, display, exit",
            Style::default().fg(Color::LightYellow),
        )),
    }

    Paragraph::new(header_text)
        .wrap(Wrap { trim: false })
        .render(header, frame.buffer_mut());

    let username = env::var("USER").unwrap_or_else(|_| "user".to_string());
    let footer_prompt = Line::from(vec![
        Span::styled(username, Style::default().fg(Color::White)),
        Span::raw(" $: "),
        if app.current_mode == Display {
            Span::styled(
                ">",
                Style::default()
                    .fg(Color::White)
                    .bold()
                    .underlined()
                    .slow_blink(),
            )
        } else {
            Span::raw("")
        },
        Span::raw(" "),
    ]);
    let footer_prompt_length: usize = footer_prompt.spans.iter().map(|span| span.width()).sum();
    let [left_side_command_prompt, right_side_command_prompt] = Layout::horizontal([
        Constraint::Max(footer_prompt_length.try_into().unwrap()),
        Constraint::Fill(1),
    ])
    .areas(footer);

    frame.render_widget(footer_prompt, left_side_command_prompt);
    frame.render_widget(&app.text_areas.command_prompt, right_side_command_prompt);

    match &app.current_mode {
        Select => render_select(frame, app, body),
        AddNoteTitle | AddNoteBody | AddNoteRelevance => render_add(frame, app, body),
        Remove => render_remove(frame, app, body),
        ModifyNoteSelect => render_modify_select(frame, app, body),
        ModifyNoteTitle | ModifyNoteBody => render_modify(frame, app, body),
        Display => render_display(frame, app, body),
    }
}

fn render_select(frame: &mut Frame, app: &mut AppState, body: Rect) {
    let [search_bar, select_menu] = Layout::vertical([Constraint::Length(3), Constraint::Fill(1)])
        .margin(2)
        .areas(body);

    let [symbol, text_area] = Layout::horizontal([Constraint::Length(2), Constraint::Fill(1)])
        .margin(1)
        .areas(search_bar);

    Block::bordered()
        .border_type(BorderType::Rounded)
        .title(Line::from("Search Bar").fg(Color::LightRed))
        .fg(Color::White)
        .render(search_bar, frame.buffer_mut());

    Line::from(vec![
        Span::styled(
            ">",
            Style::default()
                .fg(Color::White)
                .bold()
                .underlined()
                .slow_blink(),
        ),
        Span::raw(" "),
    ])
    .render(symbol, frame.buffer_mut());

    frame.render_widget(&app.text_areas.search_bar, text_area);

    match &app.notes {
        Some(notes) => {
            let list = List::new(
                notes
                    .iter()
                    .map(|x| ListItem::from(x.title.as_str()).fg(Color::Gray)),
            )
            .highlight_symbol(">")
            .highlight_style(Style::default().bg(Color::Green).fg(Color::White))
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title(
                        Line::from("Select any Note to Display it")
                            .fg(Color::LightYellow)
                            .bold(),
                    )
                    .fg(Color::White),
            );

            frame.render_stateful_widget(list, select_menu, &mut app.selected_item);
        }
        None => {
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title("No Available Notes")
                .fg(Color::Red)
                .render(select_menu, frame.buffer_mut());
        }
    }
}

fn render_add(frame: &mut Frame, app: &mut AppState, body: Rect) {
    let [left_side, right_side] =
        Layout::horizontal([Constraint::Percentage(80), Constraint::Percentage(20)])
            .margin(1)
            .areas(body);

    let [title_area, body_area] = Layout::vertical([Constraint::Length(3), Constraint::Fill(1)])
        .margin(1)
        .areas(left_side);

    if app.current_mode == AddNoteTitle {
        app.text_areas
            .note_title
            .set_cursor_style(Style::default().reversed().slow_blink());
        app.text_areas.note_body.set_cursor_style(Style::default());
    } else if app.current_mode == AddNoteBody {
        app.text_areas
            .note_body
            .set_cursor_style(Style::default().reversed().slow_blink());
        app.text_areas.note_title.set_cursor_style(Style::default());
    } else {
        app.text_areas.note_title.set_cursor_style(Style::default());
        app.text_areas.note_body.set_cursor_style(Style::default());
    }

    frame.render_widget(&app.text_areas.note_title, title_area);
    frame.render_widget(&app.text_areas.note_body, body_area);

    let list = List::new([
        ListItem::new(RelevanceLevel::Normal.to_string()).style(Style::new().fg(Color::Gray)),
        ListItem::new(RelevanceLevel::Relevant.to_string()).style(Style::new().fg(Color::Green)),
        ListItem::new(RelevanceLevel::Important.to_string())
            .style(Style::new().fg(Color::LightYellow)),
        ListItem::new(RelevanceLevel::Crucial.to_string()).style(Style::new().fg(Color::LightRed)),
    ])
    .highlight_symbol(">")
    .highlight_style(
        Style::default()
            .bg(Color::Green)
            .fg(Color::White)
            .slow_blink(),
    )
    .block(
        Block::bordered()
            .title(Line::from("Relevance").fg(Color::LightYellow))
            .border_type(BorderType::Rounded)
            .padding(Padding::vertical(1))
            .fg(Color::White),
    );
    frame.render_stateful_widget(list, right_side, &mut app.selected_item);
}

fn render_remove(frame: &mut Frame, app: &mut AppState, body: Rect) {
    let [search_bar, select_menu] = Layout::vertical([Constraint::Length(3), Constraint::Fill(1)])
        .margin(2)
        .areas(body);

    let [symbol, text_area] = Layout::horizontal([Constraint::Length(2), Constraint::Fill(1)])
        .margin(1)
        .areas(search_bar);

    Block::bordered()
        .border_type(BorderType::Rounded)
        .title(Line::from("Search Bar").fg(Color::LightRed))
        .fg(Color::White)
        .render(search_bar, frame.buffer_mut());

    Line::from(vec![
        Span::styled(
            ">",
            Style::default()
                .fg(Color::Gray)
                .bold()
                .underlined()
                .slow_blink(),
        ),
        Span::raw(" "),
    ])
    .render(symbol, frame.buffer_mut());

    frame.render_widget(&app.text_areas.search_bar, text_area);

    match &app.notes {
        Some(notes) => {
            let list = List::new(
                notes
                    .iter()
                    .map(|x| ListItem::from(x.title.as_str()).fg(Color::Gray)),
            )
            .highlight_symbol(">")
            .highlight_style(Style::default().bg(Color::Red).fg(Color::White))
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title(
                        Line::from("Select a Note to Permanently Remove it")
                            .fg(Color::LightRed)
                            .bold(),
                    )
                    .fg(Color::White),
            );

            frame.render_stateful_widget(list, select_menu, &mut app.selected_item);
        }
        None => {
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title("No Available Notes")
                .fg(Color::Red)
                .render(select_menu, frame.buffer_mut());
        }
    }
}

fn render_modify_select(frame: &mut Frame, app: &mut AppState, body: Rect) {
    let [search_bar, select_menu] = Layout::vertical([Constraint::Length(3), Constraint::Fill(1)])
        .margin(2)
        .areas(body);

    let [symbol, text_area] = Layout::horizontal([Constraint::Length(2), Constraint::Fill(1)])
        .margin(1)
        .areas(search_bar);

    Block::bordered()
        .border_type(BorderType::Rounded)
        .title(Line::from("Search Bar").fg(Color::LightRed))
        .fg(Color::White)
        .render(search_bar, frame.buffer_mut());

    Line::from(vec![
        Span::styled(
            ">",
            Style::default()
                .fg(Color::Green)
                .bold()
                .underlined()
                .slow_blink(),
        ),
        Span::raw(" "),
    ])
    .render(symbol, frame.buffer_mut());

    frame.render_widget(&app.text_areas.search_bar, text_area);

    match &app.notes {
        Some(notes) => {
            let list = List::new(
                notes
                    .iter()
                    .map(|x| ListItem::from(x.title.as_str()).fg(Color::Gray)),
            )
            .highlight_symbol(">")
            .highlight_style(Style::default().bg(Color::Yellow).fg(Color::White))
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title(
                        Line::from("Select a Note to Modify it")
                            .fg(Color::Green)
                            .bold(),
                    )
                    .fg(Color::White),
            );

            frame.render_stateful_widget(list, select_menu, &mut app.selected_item);
        }
        None => {
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title("No Available Notes")
                .fg(Color::Red)
                .render(select_menu, frame.buffer_mut());
        }
    }
}

fn render_modify(frame: &mut Frame, app: &mut AppState, body: Rect) {
    let [title_area, body_area] = Layout::vertical([Constraint::Length(3), Constraint::Fill(1)])
        .margin(1)
        .areas(body);

    if app.current_mode == ModifyNoteTitle {
        app.text_areas
            .note_title
            .set_cursor_style(Style::default().reversed().slow_blink());
        app.text_areas.note_body.set_cursor_style(Style::default());
    } else if app.current_mode == ModifyNoteBody {
        app.text_areas
            .note_body
            .set_cursor_style(Style::default().reversed().slow_blink());
        app.text_areas.note_title.set_cursor_style(Style::default());
    }

    frame.render_widget(&app.text_areas.note_title, title_area);
    frame.render_widget(&app.text_areas.note_body, body_area);
}

fn render_display(frame: &mut Frame, app: &mut AppState, body: Rect) {
    match &app.current_note {
        Some(note) => {
            let [left_side, right_side] =
                Layout::horizontal([Constraint::Percentage(70), Constraint::Percentage(30)])
                    .margin(1)
                    .areas(body);

            Paragraph::new(note.body.as_str())
                .fg(Color::White)
                .wrap(Wrap { trim: false })
                .scroll((app.display_offset, 0))
                .block(
                    Block::bordered()
                        .border_type(BorderType::Rounded)
                        .padding(Padding::uniform(1))
                        .fg(Color::Gray),
                )
                .render(left_side, frame.buffer_mut());

            let relevance_level = match &note.relevance {
                Normal => Line::from(RelevanceLevel::Normal.to_string().fg(Color::Gray)),
                Relevant => Line::from(RelevanceLevel::Relevant.to_string().fg(Color::Green)),
                Important => {
                    Line::from(RelevanceLevel::Important.to_string().fg(Color::LightYellow))
                }
                Crucial => Line::from(RelevanceLevel::Crucial.to_string().fg(Color::LightRed)),
            };

            Paragraph::new(vec![
                Line::from(note.title.as_str().fg(Color::LightYellow)),
                Line::default(),
                relevance_level.bold().underlined().italic(),
                Line::default(),
                Line::from(note.id.to_string().fg(Color::LightRed)),
                Line::default(),
                Line::from(note.timestamp.as_str().fg(Color::Gray)),
            ])
            .wrap(Wrap { trim: false })
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .fg(Color::Yellow),
            )
            .render(right_side, frame.buffer_mut());
        }
        None => {
            let [inner_body] = Layout::default()
                .constraints([Constraint::Fill(1)])
                .margin(1)
                .areas(body);

            Paragraph::new("Select any note to display it")
                .fg(Color::LightRed)
                .bold()
                .slow_blink()
                .wrap(Wrap { trim: false })
                .block(
                    Block::bordered()
                        .border_type(BorderType::Rounded)
                        .padding(Padding::uniform(3))
                        .fg(Color::Yellow),
                )
                .render(inner_body, frame.buffer_mut());
        }
    }
}
