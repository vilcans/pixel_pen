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
    egui::{self, Label, Rgba, RichText},
    epi,
};
use std::time::Instant;

const POPUP_MESSAGE_TIME: f32 = 3.0;
const POPUP_HIGHLIGHT_TIME: f32 = 0.4;
const POPUP_FADE_OUT_TIME: f32 = 0.8;

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
}

/// State of the whole application.
pub struct Application {
    editors: Editors,
    pub system: Box<dyn SystemFunctions>,
    /// For giving each new document its own number
    next_document_index: u32,
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

fn check_quit(system: &mut dyn SystemFunctions, editors: &Editors) -> bool {
    let unsaved: Vec<String> = editors
        .iter()
        .enumerate()
        .filter(|(_, ed)| !ed.history.is_saved())
        .map(|(i, ed)| match &ed.doc.filename {
            Some(f) => f.display().to_string(),
            None => format!("Untitled {}", i),
        })
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
    system: &mut dyn SystemFunctions,
    user_actions: &mut Vec<Action>,
) -> Vec<Action> {
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        // Menu bar
        let doc_filename = editors.active_mut().unwrap().doc.filename.clone();
        egui::menu::bar(ui, |ui| {
            egui::menu::menu_button(ui, "File", |ui| {
                if system.has_open_file_dialog() && ui.button("Open...").clicked() {
                    match system
                        .open_file_dialog(OpenFileOptions::for_open(doc_filename.as_deref()))
                    {
                        Ok(Some(filename)) => {
                            match storage::load_any_file(std::path::Path::new(&filename)) {
                                Ok(doc) => {
                                    user_actions.push(Action::Ui(UiAction::NewDocument(doc)));
                                }
                                Err(e) => {
                                    system.show_error(&format!("Failed to load: {:?}", e));
                                }
                            }
                        }
                        Ok(None) => {}
                        Err(e) => {
                            system.show_error(&format!("Could not get file name: {:?}", e));
                        }
                    }
                }
                let ed = editors.active_mut().unwrap();
                ed.update_file_menu(ui, system);
                ui.separator();
                if ui.button("Quit").clicked() && check_quit(system, editors) {
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
            ui.horizontal_wrapped(|ui| {
                for (index, ed) in editors.iter().enumerate() {
                    if ui
                        .selectable_label(selected_index == index, ed.doc.short_name())
                        .clicked()
                    {
                        selected_index = index;
                    }
                }
            });
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
        ed.update_central_panel(ui, frame, ctx, &mut cursor_icon, user_actions);
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

impl Application {
    pub fn new() -> Self {
        let system = Box::new(system::DummySystemFunctions {});
        Self {
            editors: Default::default(),
            system,
            next_document_index: 1,
        }
    }

    pub fn add_editor(&mut self, doc: Document) -> usize {
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
                UiAction::NewDocument(mut doc) => {
                    self.next_document_index += 1;
                    doc.index_number = self.next_document_index;
                    self.add_editor(doc);
                }
                _action => {
                    eprintln!("Unhandled UiAction");
                }
            },
        }
    }
}
