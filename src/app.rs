use crate::{
    actions::{Action, UiAction, Undoable},
    cell_image::CellImageSize,
    document::Document,
    editor::Editor,
    error::Severity,
    mode::Mode,
    storage,
    system::{self, OpenFileOptions, SystemFunctions},
    tool::Tool,
    ui::{UiState, ViewSettings},
};
use eframe::{
    egui::{self, Label, Rgba},
    epi,
};
use std::{path::Path, time::Instant};
use undo::Record;

const POPUP_MESSAGE_TIME: f32 = 3.0;
const POPUP_HIGHLIGHT_TIME: f32 = 0.4;
const POPUP_FADE_OUT_TIME: f32 = 0.8;

/// State of the whole application.
pub struct Application {
    editor: Editor,
    pub system: Box<dyn SystemFunctions>,
}

impl Default for Application {
    fn default() -> Self {
        Application::new()
    }
}

impl epi::App for Application {
    fn name(&self) -> &str {
        "Pixel Pen"
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        let mut user_actions = Vec::new();
        let ed = &mut self.editor;
        let system = self.system.as_mut();

        let (_width, _height) = ed.doc.image.size_in_pixels();

        for e in ctx.input().events.iter() {
            if !ctx.wants_keyboard_input() {
                if let egui::Event::Text(t) = e {
                    create_actions_from_keyboard(t, &mut user_actions);
                }
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // Menu bar
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if system.has_open_file_dialog() && ui.button("Open...").clicked() {
                        match system
                            .open_file_dialog(OpenFileOptions::for_open(ed.doc.filename.as_deref()))
                        {
                            Ok(Some(filename)) => {
                                if ed.history.is_saved()
                                    || check_open(system, ed.doc.filename.as_deref())
                                {
                                    match storage::load_any_file(std::path::Path::new(&filename)) {
                                        Ok(doc) => {
                                            user_actions
                                                .push(Action::Ui(UiAction::NewDocument(doc)));
                                        }
                                        Err(e) => {
                                            system.show_error(&format!("Failed to load: {:?}", e));
                                        }
                                    }
                                }
                            }
                            Ok(None) => {}
                            Err(e) => {
                                system.show_error(&format!("Could not get file name: {:?}", e));
                            }
                        }
                    }
                    ed.update_file_menu(ui, system);
                    ui.separator();
                    if ui.button("Quit").clicked()
                        && (ed.history.is_saved() || check_quit(system, ed.doc.filename.as_deref()))
                    {
                        frame.quit();
                    }
                });
                egui::menu::menu(ui, "Edit", |ui| {
                    ed.update_edit_menu(ui, &mut user_actions);
                });
            });

            // Top toolbar
            ed.update_top_toolbar(ui, &mut user_actions);
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            if let Some((time, message)) = ed.ui_state.message.as_ref() {
                let age = Instant::now()
                    .saturating_duration_since(*time)
                    .as_secs_f32();
                let highlight =
                    1.0 - ((age - POPUP_HIGHLIGHT_TIME) / POPUP_FADE_OUT_TIME).clamp(0.0, 1.0);
                let bg_color = Rgba::RED * highlight;
                let text_color = (Rgba::WHITE * highlight)
                    + (Rgba::from(ctx.style().visuals.text_color()) * (1.0 - highlight));
                ui.add(
                    Label::new(message)
                        .text_color(text_color)
                        .background_color(bg_color),
                );
                if age >= POPUP_MESSAGE_TIME {
                    ed.ui_state.message = None;
                } else if highlight > 0.0 {
                    ctx.request_repaint(); // to animate color highlight
                }
            } else {
                ui.label(ed.ui_state.tool.instructions(&ed.ui_state.mode));
            }
        });

        // Left toolbar
        egui::SidePanel::left("toolbar").show(ctx, |ui| {
            ed.update_left_toolbar(ui, &mut user_actions);
        });

        let mut cursor_icon = None;

        // Main image.
        egui::CentralPanel::default().show(ctx, |ui| {
            ed.update_central_panel(ui, frame, ctx, &mut cursor_icon, &mut user_actions);
        });

        for action in user_actions {
            apply_action(&mut ed.doc, &mut ed.history, &mut ed.ui_state, action);
        }

        if let Some(icon) = cursor_icon {
            ctx.output().cursor_icon = icon;
        }
    }
}

fn check_quit(system: &mut dyn SystemFunctions, filename: Option<&Path>) -> bool {
    system
        .request_confirmation(&format!(
            "File is not saved: \"{}\". Are you sure you want to quit?",
            filename
                .map(|path| path.to_string_lossy().to_string())
                .unwrap_or_else(|| "Untitled".to_string())
        ))
        .unwrap_or(false)
}

fn check_open(system: &mut dyn SystemFunctions, filename: Option<&Path>) -> bool {
    system
        .request_confirmation(&format!(
            "File is not saved: \"{}\". Are you sure you want to open a new file?",
            filename
                .map(|path| path.to_string_lossy().to_string())
                .unwrap_or_else(|| "Untitled".to_string())
        ))
        .unwrap_or(false)
}

/// Apply an action and record it in the history. Show any error to the user.
fn apply_action(
    doc: &mut Document,
    history: &mut Record<Undoable>,
    ui_state: &mut UiState,
    action: Action,
) {
    match action {
        Action::Document(action) => {
            let was_dirty = doc.image.dirty;
            match history.apply(doc, Undoable::new(action)) {
                Ok(true) => (),
                Ok(false) => doc.image.dirty = was_dirty,
                Err(e) => match e.severity() {
                    Severity::Silent => {}
                    Severity::Notification => ui_state.show_warning(e.to_string()),
                },
            }
        }
        Action::Ui(action) => match action {
            UiAction::Undo => {
                if history.can_undo() {
                    history.undo(doc);
                    doc.image.dirty = true;
                }
            }
            UiAction::Redo => {
                if history.can_redo() {
                    history.redo(doc);
                    doc.image.dirty = true;
                }
            }
            UiAction::NewDocument(new_doc) => {
                *doc = new_doc;
            }
            UiAction::SelectTool(tool) => ui_state.tool = tool,
            UiAction::SelectMode(mode) => ui_state.mode = mode,
            UiAction::CreateCharBrush { rect } => {
                if let Some(rect) = rect.within_size(doc.image.size_in_cells()) {
                    ui_state.char_brush = doc.image.grab_cells(&rect);
                    ui_state.tool = Tool::CharBrush(Default::default());
                } else {
                    println!("Rect {:?} did not fit inside image", rect);
                }
            }
            UiAction::ZoomIn => {
                if ui_state.zoom < 16.0 {
                    ui_state.zoom *= 2.0;
                }
            }
            UiAction::ZoomOut => {
                if ui_state.zoom > 1.0 {
                    ui_state.zoom /= 2.0;
                }
            }
            UiAction::SetZoom(amount) => {
                ui_state.zoom = amount;
            }
            UiAction::ToggleGrid => ui_state.grid = !ui_state.grid,
            UiAction::ToggleRaw => {
                ui_state.image_view_settings = match ui_state.image_view_settings {
                    ViewSettings::Normal => ViewSettings::Raw,
                    ViewSettings::Raw => ViewSettings::Normal,
                }
            }
            UiAction::ViewSettings(settings) => {
                ui_state.image_view_settings = settings;
            }
        },
    }
}

fn create_actions_from_keyboard(keypress: &str, actions: &mut Vec<Action>) {
    let action = match keypress {
        "+" => Action::Ui(UiAction::ZoomIn),
        "-" => Action::Ui(UiAction::ZoomOut),
        "b" => Action::Ui(UiAction::SelectTool(Tool::CharBrush(Default::default()))),
        "c" => Action::Ui(UiAction::SelectMode(Mode::CellColor)),
        "d" => Action::Ui(UiAction::SelectTool(Tool::Paint(Default::default()))),
        "f" => Action::Ui(UiAction::SelectMode(Mode::FillCell)),
        "g" => Action::Ui(UiAction::ToggleGrid),
        "h" => Action::Ui(UiAction::SelectMode(Mode::MakeHiRes)),
        "H" => Action::Ui(UiAction::SelectMode(Mode::MakeMulticolor)),
        "r" => Action::Ui(UiAction::SelectMode(Mode::ReplaceColor)),
        "R" => Action::Ui(UiAction::SelectMode(Mode::SwapColors)),
        "w" => Action::Ui(UiAction::ToggleRaw),
        "u" => Action::Ui(UiAction::Undo),
        "U" => Action::Ui(UiAction::Redo),
        "v" => Action::Ui(UiAction::SelectTool(Tool::Grab(Default::default()))),
        _ => return,
    };
    actions.push(action);
}

impl Application {
    pub fn new() -> Self {
        let system = Box::new(system::DummySystemFunctions {});
        Self {
            editor: Editor::default(),
            system,
        }
    }

    pub fn add_editor(&mut self, doc: Document) {
        self.editor = Editor::with_doc(doc);
    }

    pub fn editor_mut(&mut self) -> &mut Editor {
        &mut self.editor
    }
}
