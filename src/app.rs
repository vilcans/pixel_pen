use crate::cell_image::CellImageSize;
use crate::egui_extensions::EnhancedResponse;
use crate::vic::Char;
use crate::{
    actions::{Action, UiAction},
    editor::Editor,
    mode::Mode,
    storage,
    system::{self, OpenFileOptions, SystemFunctions},
    tool::Tool,
    Document,
};
use eframe::{
    egui::{self, Color32, Label, Rgba, RichText, Sense, Shape, Stroke},
    epi,
};
use imgref::ImgVec;
use std::time::Instant;

const POPUP_MESSAGE_TIME: f32 = 3.0;
const POPUP_HIGHLIGHT_TIME: f32 = 0.4;
const POPUP_FADE_OUT_TIME: f32 = 0.8;

const TAB_SPACING: f32 = 5.0;
const TAB_STROKE: Stroke = Stroke {
    width: 0.1,
    color: Color32::LIGHT_GRAY,
};

/// All open editors, and the currently active one.
#[derive(Default)]
struct Editors {
    list: Vec<Editor>,
    active: usize,
}
#[allow(dead_code)] // not all methods are currently used
impl Editors {
    pub fn add(&mut self, ed: Editor) -> usize {
        let i = self.list.len();
        self.list.push(ed);
        i
    }
    pub fn len(&self) -> usize {
        self.list.len()
    }
    pub fn active_index(&self) -> usize {
        self.active
    }
    pub fn set_active_index(&mut self, index: usize) {
        assert!(index < self.list.len());
        self.active = index;
    }
    pub fn has_active(&self) -> bool {
        !self.list.is_empty()
    }
    pub fn active(&self) -> Option<&Editor> {
        self.list.get(self.active)
    }
    pub fn active_mut(&mut self) -> Option<&mut Editor> {
        self.list.get_mut(self.active)
    }
    pub fn get(&self, index: usize) -> Option<&Editor> {
        self.list.get(index)
    }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Editor> {
        self.list.get_mut(index)
    }
    pub fn iter(&self) -> impl Iterator<Item = &Editor> {
        self.list.iter()
    }
    fn remove(&mut self, index: usize) {
        self.list.remove(index);
        if self.active > index || self.active == index && self.active == self.list.len() {
            self.active = self.active.saturating_sub(1);
        }
    }

    fn find_by_filename(&self, filename: &std::path::PathBuf) -> Option<usize> {
        self.list
            .iter()
            .enumerate()
            .find(|(_, ed)| matches!(&ed.doc.filename, Some(f) if f == filename))
            .map(|(idx, _)| idx)
    }
}

/// State of the whole application.
pub struct Application {
    editors: Editors,
    pub system: Box<dyn SystemFunctions>,
    /// For giving each new document its own number
    next_document_index: u32,
    brush: ImgVec<Char>,
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
    fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
        let mut user_actions = Vec::new();

        for e in ctx.input().events.iter() {
            if !ctx.wants_keyboard_input() {
                if let egui::Event::Text(t) = e {
                    create_actions_from_keyboard(t, &mut user_actions);
                }
            }
        }

        if self.editors.has_active() {
            let actions = update_with_editor(
                ctx,
                frame,
                &mut self.editors,
                &self.brush,
                self.system.as_mut(),
                &mut user_actions,
            );
            // Apply the actions the editor did not handle
            for action in actions.into_iter() {
                self.apply_action(action);
            }
        }
    }
}

fn check_close(system: &mut dyn SystemFunctions, ed: &Editor) -> bool {
    if ed.history.is_saved() {
        true
    } else {
        system
            .request_confirmation(&format!(
                "This file is not saved:\n\n{}\n\nAre you sure you want to close it?",
                ed.doc.visible_name()
            ))
            .unwrap_or(false)
    }
}

fn check_quit(system: &mut dyn SystemFunctions, editors: &Editors) -> bool {
    let unsaved: Vec<String> = editors
        .iter()
        .filter(|ed| !ed.history.is_saved())
        .map(|ed| ed.doc.visible_name())
        .collect();
    if unsaved.is_empty() {
        return true;
    }
    system
        .request_confirmation(&format!(
            "The following files are not saved:\n\n{}\n\nAre you sure you want to quit?",
            unsaved.join("\n")
        ))
        .unwrap_or(false)
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

/// UI for when there is an active editor.
fn update_with_editor(
    ctx: &egui::CtxRef,
    frame: &epi::Frame,
    editors: &mut Editors,
    brush: &ImgVec<Char>,
    system: &mut dyn SystemFunctions,
    user_actions: &mut Vec<Action>,
) -> Vec<Action> {
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        // Menu bar
        let doc_filename = editors.active_mut().unwrap().doc.filename.clone();
        egui::menu::bar(ui, |ui| {
            egui::menu::menu_button(ui, "File", |ui| {
                if ui.button("New").clicked_with_close(ui) {
                    let doc = Document::new();
                    user_actions.push(Action::Ui(UiAction::NewDocument(doc)));
                }
                if system.has_open_file_dialog() && ui.button("Open...").clicked_with_close(ui) {
                    match system
                        .open_file_dialog(OpenFileOptions::for_open(doc_filename.as_deref()))
                    {
                        Ok(Some(filename)) => {
                            open_file(filename, editors, system, user_actions);
                        }
                        Ok(None) => {}
                        Err(e) => {
                            system.show_error(&format!("Could not get file name: {:?}", e));
                        }
                    }
                }
                editors.active_mut().unwrap().update_file_menu(ui, system);
                ui.separator();
                ui.add_enabled_ui(editors.has_active() && editors.len() > 1, |ui| {
                    let ed = editors.active_mut().unwrap();
                    if ui.button("Close").clicked_with_close(ui) && check_close(system, ed) {
                        user_actions
                            .push(Action::Ui(UiAction::CloseEditor(editors.active_index())));
                    }
                });
                ui.separator();
                if ui.button("Quit").clicked_with_close(ui) && check_quit(system, editors) {
                    frame.quit();
                }
            });
            egui::menu::menu_button(ui, "Edit", |ui| {
                let ed = editors.active_mut().unwrap();
                ed.update_edit_menu(ui, user_actions);
            });
        });

        // Document selector
        {
            let mut selected_index = editors.active_index();
            let mut selected_rect = egui::Rect::NOTHING;
            egui::ScrollArea::horizontal().show(ui, |ui| {
                ui.horizontal(|ui| {
                    for (index, ed) in editors.iter().enumerate() {
                        let selected = selected_index == index;
                        let name = ed.doc.short_name();
                        let response = if selected {
                            ui.add_space(TAB_SPACING);
                            let response = ui.add(
                                Label::new(RichText::new(name).strong()).sense(Sense::click()),
                            );
                            selected_rect = response.rect;
                            response
                        } else {
                            ui.add_space(TAB_SPACING);
                            let response = ui
                                .add(Label::new(RichText::new(name).weak()).sense(Sense::click()));
                            let rect = response.rect;
                            ui.painter().add(Shape::line(
                                vec![
                                    rect.left_bottom() - egui::Vec2::new(TAB_SPACING, 0.0),
                                    rect.left_top(),
                                    rect.right_top(),
                                    rect.right_bottom() + egui::Vec2::new(TAB_SPACING, 0.0),
                                ],
                                TAB_STROKE,
                            ));
                            if response.clicked() {
                                selected_index = index;
                            }
                            response
                        };
                        if let Some(filename) = &ed.doc.filename {
                            response.on_hover_text(filename.to_string_lossy().to_string());
                        }
                    }
                });
            });
            ui.painter().add(Shape::line(
                vec![
                    egui::Pos2::new(0.0, selected_rect.max.y),
                    selected_rect.left_bottom() - egui::Vec2::new(TAB_SPACING, 0.0),
                    selected_rect.left_top(),
                    selected_rect.right_top(),
                    selected_rect.right_bottom() + egui::Vec2::new(TAB_SPACING, 0.0),
                    egui::Pos2::new(10000.0, selected_rect.max.y),
                ],
                TAB_STROKE,
            ));
            editors.set_active_index(selected_index);
        }

        // Top toolbar
        let ed = editors.active_mut().unwrap();
        ed.update_top_toolbar(ui, user_actions);
    });

    egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
        let ed = editors.active_mut().unwrap();
        if ed.ui_state.message.is_some() {
            let (time, message) = ed.ui_state.message.clone().unwrap();
            let age = Instant::now().saturating_duration_since(time).as_secs_f32();
            let highlight =
                1.0 - ((age - POPUP_HIGHLIGHT_TIME) / POPUP_FADE_OUT_TIME).clamp(0.0, 1.0);
            let bg_color = Rgba::RED * highlight;
            let text_color = (Rgba::WHITE * highlight)
                + (Rgba::from(ctx.style().visuals.text_color()) * (1.0 - highlight));
            ui.add(Label::new(
                RichText::new(message)
                    .color(text_color)
                    .background_color(bg_color),
            ));
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
        let ed = editors.active_mut().unwrap();
        ed.update_left_toolbar(ui, user_actions);
    });

    let mut cursor_icon = None;

    // Main image.
    egui::CentralPanel::default().show(ctx, |ui| {
        let ed = editors.active_mut().unwrap();
        ed.update_central_panel(ui, frame, ctx, &mut cursor_icon, brush, user_actions);
    });

    let ed = editors.active_mut().unwrap();
    let mut unhandled_actions = Vec::new();
    for action in user_actions.drain(..) {
        if let Some(action) = ed.apply_action(action) {
            unhandled_actions.push(action);
        }
    }

    if let Some(icon) = cursor_icon {
        ctx.output().cursor_icon = icon;
    }

    unhandled_actions
}

/// Open file or show error to user.
/// Switches to an existing editor if the document is already open.
fn open_file(
    filename: std::path::PathBuf,
    editors: &mut Editors,
    system: &mut dyn SystemFunctions,
    user_actions: &mut Vec<Action>,
) {
    if let Some(i) = editors.find_by_filename(&filename) {
        editors.set_active_index(i);
        return;
    }
    match storage::load_any_file(std::path::Path::new(&filename)) {
        Ok(doc) => {
            user_actions.push(Action::Ui(UiAction::NewDocument(doc)));
        }
        Err(e) => {
            system.show_error(&format!("Failed to load: {:?}", e));
        }
    }
}

impl Application {
    pub fn new() -> Self {
        let system = Box::new(system::DummySystemFunctions {});
        Self {
            editors: Default::default(),
            system,
            next_document_index: 1,
            brush: ImgVec::new(vec![Char::DEFAULT_BRUSH], 1, 1),
        }
    }

    pub fn add_editor(&mut self, mut doc: Document) -> usize {
        doc.index_number = self.next_document_index;
        self.next_document_index += 1;
        let editor = Editor::with_doc(doc);
        let i = self.editors.add(editor);
        self.editors.set_active_index(i);
        i
    }

    pub fn editor_mut(&mut self, index: usize) -> Option<&mut Editor> {
        self.editors.get_mut(index)
    }

    fn apply_action(&mut self, action: Action) {
        match action {
            Action::Document(_) => eprintln!("Unhandled Document action"),
            Action::Ui(ui_action) => match ui_action {
                UiAction::NewDocument(doc) => {
                    self.add_editor(doc);
                }
                UiAction::CloseEditor(index) => {
                    self.editors.remove(index);
                }
                UiAction::CreateCharBrush { rect } => {
                    if let Some(ed) = self.editors.active_mut() {
                        if let Some(rect) = rect.within_size(ed.doc.image.size_in_cells()) {
                            self.brush = ed.doc.image.grab_cells(&rect);
                            ed.ui_state.tool = Tool::CharBrush(Default::default());
                        } else {
                            println!("Rect {:?} did not fit inside image", rect);
                        }
                    }
                }
                _action => {
                    eprintln!("Unhandled UiAction");
                }
            },
        }
    }
}
