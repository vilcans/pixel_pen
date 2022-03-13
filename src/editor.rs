use std::path::Path;

use eframe::{
    egui::{
        self, epaint::Mesh, Align, Align2, Color32, CursorIcon, Painter, PointerButton, Pos2, Rect,
        Response, Shape, Stroke, TextStyle, Ui, Vec2,
    },
    epi::TextureAllocator,
};
use imgref::ImgVec;
use undo::Record;

use crate::{
    actions::{self, Action, UiAction, Undoable},
    cell_image::CellImageSize,
    coords::{PixelPoint, PixelTransform},
    egui_extensions::EnhancedResponse,
    error::{Error, Severity},
    import::Import,
    mode::Mode,
    storage,
    system::{OpenFileOptions, SaveFileOptions, SystemFunctions},
    texture::{self, Texture},
    tool::{ImportTool, Tool},
    ui::{self, text, UiState, ViewSettings},
    vic::{Char, VicImage},
    Document,
};

const BORDER_CORNER_RADIUS: f32 = 15.0;
const BORDER_SIZE: Vec2 = Vec2::new(25.0, 20.0);

const GRID_COLOR: Color32 = Color32::GRAY;

/// An open document and its state.
pub struct Editor {
    pub doc: Document,
    pub ui_state: UiState,
    pub image_texture: Option<Texture>,
    pub history: Record<actions::Undoable>,
}

impl Editor {
    pub fn with_doc(doc: Document) -> Self {
        Self {
            doc,
            ui_state: Default::default(),
            image_texture: None,
            history: Default::default(),
        }
    }

    pub fn start_import_mode(&mut self, filename: &Path) -> Result<(), Error> {
        let mut i = Import::load(filename)?;
        i.settings.width = i
            .settings
            .width
            .min(self.doc.image.size_in_pixels().0 as u32);
        i.settings.height = i
            .settings
            .height
            .min(self.doc.image.size_in_pixels().1 as u32);
        self.ui_state.tool = Tool::Import(ImportTool::new(i));
        Ok(())
    }

    pub fn update_file_menu(&mut self, ui: &mut Ui, system: &mut dyn SystemFunctions) {
        if system.has_open_file_dialog() && ui.button("Import...").clicked_with_close(ui) {
            match system.open_file_dialog(OpenFileOptions::for_import(match &self.ui_state.tool {
                Tool::Import(tool) => tool.filename(),
                _ => None,
            })) {
                Ok(Some(filename)) => match self.start_import_mode(&filename) {
                    Ok(()) => {}
                    Err(e) => system.show_error(&format!(
                        "Could not import file {}: {:?}",
                        filename.display(),
                        e
                    )),
                },
                Ok(None) => {}
                Err(e) => system.show_error(&format!("Could not get file name: {:?}", e)),
            }
        }
        if system.has_save_file_dialog() {
            ui.separator();
            match self.doc.filename.clone() {
                Some(filename) => {
                    if ui
                        .button(format!(
                            "Save {}",
                            filename
                                .file_name()
                                .map(|s| s.to_string_lossy())
                                .unwrap_or_default()
                        ))
                        .clicked_with_close(ui)
                    {
                        save(&mut self.history, &mut self.doc, &filename, system);
                    }
                }
                None => {
                    if ui.button("Save").clicked_with_close(ui) {
                        save_as(&mut self.history, &mut self.doc, system);
                    }
                }
            }
            if ui.button("Save As...").clicked_with_close(ui) {
                save_as(&mut self.history, &mut self.doc, system);
            }
            if ui.button("Export...").clicked_with_close(ui) {
                export(&self.doc, system);
            }
        }
    }

    pub fn update_edit_menu(&mut self, ui: &mut Ui, user_actions: &mut Vec<Action>) {
        ui.set_enabled(self.history.can_undo());
        if ui.button("Undo").clicked_with_close(ui) {
            user_actions.push(Action::Ui(UiAction::Undo));
        }
        ui.set_enabled(self.history.can_redo());
        if ui.button("Redo").clicked_with_close(ui) {
            user_actions.push(Action::Ui(UiAction::Redo));
        }
    }

    pub fn update_top_toolbar(&mut self, ui: &mut Ui, user_actions: &mut Vec<Action>) {
        ui.vertical(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("Zoom:");
                if ui.button("-").on_hover_text("Zoom out").clicked() {
                    user_actions.push(Action::Ui(UiAction::ZoomOut));
                }
                if ui
                    .button(format!("{:0.0}x", self.ui_state.zoom))
                    .on_hover_text("Set to 2x")
                    .clicked()
                {
                    user_actions.push(Action::Ui(UiAction::SetZoom(2.0)));
                }
                if ui.button("+").on_hover_text("Zoom in").clicked() {
                    user_actions.push(Action::Ui(UiAction::ZoomIn));
                }
                ui.separator();
                ui.checkbox(&mut self.ui_state.grid, "Grid")
                    .on_hover_text(text::GRID_TOOLTIP);
                let mut raw_mode = self.ui_state.image_view_settings == ViewSettings::Raw;
                if ui
                    .checkbox(&mut raw_mode, "Raw")
                    .on_hover_text(text::RAW_TOOLTIP)
                    .changed()
                {
                    user_actions.push(Action::Ui(UiAction::ViewSettings(if raw_mode {
                        ViewSettings::Raw
                    } else {
                        ViewSettings::Normal
                    })))
                }
            });
            ui.separator();
            if let Some(action) = ui::palette::render_palette(
                ui,
                &mut self.ui_state.primary_color,
                &mut self.ui_state.secondary_color,
                &mut self.doc.image,
            ) {
                user_actions.push(action);
            }
        });
    }

    pub fn update_left_toolbar(&self, ui: &mut Ui, user_actions: &mut Vec<Action>) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            select_tool_ui(ui, &self.ui_state.tool, user_actions);
            if let Tool::Paint(_) = self.ui_state.tool {
                ui.separator();
                select_mode_ui(ui, &self.ui_state.mode, user_actions);
            }
        });
    }

    pub fn update_central_panel(
        &mut self,
        ui: &mut Ui,
        frame: &eframe::epi::Frame,
        ctx: &egui::CtxRef,
        cursor_icon: &mut Option<CursorIcon>,
        brush: &ImgVec<Char>,
        user_actions: &mut Vec<Action>,
    ) {
        let (width, height) = self.doc.image.size_in_pixels();
        let par = self.doc.image.pixel_aspect_ratio();
        let (response, painter) = image_painter(ui);
        let pixel_transform = PixelTransform {
            screen_rect: Rect::from_center_size(
                response.rect.center() + self.ui_state.pan,
                Vec2::new(
                    width as f32 * par * self.ui_state.zoom,
                    height as f32 * self.ui_state.zoom,
                ),
            ),
            pixel_width: width as i32,
            pixel_height: height as i32,
        };

        let hover_pos_screen = ui.input().pointer.hover_pos();
        let hover_pos = hover_pos_screen.map(|p| pixel_transform.pixel_pos(p));

        let input = ui.input();
        if input.modifiers.command {
            if input.scroll_delta.y < 0.0 {
                user_actions.push(Action::Ui(UiAction::ZoomOut));
            } else if input.scroll_delta.y > 0.0 {
                user_actions.push(Action::Ui(UiAction::ZoomIn));
            }
        } else {
            self.ui_state.pan += input.scroll_delta;
        }

        if response.drag_started() && input.pointer.button_down(PointerButton::Middle)
            || (input.pointer.button_down(PointerButton::Secondary) && input.modifiers.shift)
        {
            self.ui_state.panning = true;
        }
        if self.ui_state.panning {
            self.ui_state.pan += input.pointer.delta();
            *cursor_icon = Some(CursorIcon::Grabbing);
        }
        if response.drag_released() {
            self.ui_state.panning = false;
        }

        // Draw border
        painter.rect_filled(
            pixel_transform
                .screen_rect
                .expand2(BORDER_SIZE * self.ui_state.zoom),
            BORDER_CORNER_RADIUS * self.ui_state.zoom,
            self.doc.image.border(),
        );

        // Draw the main image
        let texture = texture::update_texture(
            &mut self.doc.image,
            &mut self.image_texture,
            frame as &dyn TextureAllocator,
            par,
            self.ui_state.zoom,
            &self.ui_state.image_view_settings,
        );
        let mut mesh = Mesh::with_texture(texture);
        mesh.add_rect_with_uv(
            pixel_transform.screen_rect,
            Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
        painter.add(Shape::Mesh(mesh));

        // Grid lines
        if self.ui_state.grid {
            draw_grid(&self.doc.image, &painter, &pixel_transform);
        }

        // Tool UI
        if !self.ui_state.panning {
            match &mut self.ui_state.tool {
                Tool::Import(tool) => {
                    tool.update_ui(ctx, ui, &painter, &self.doc, &pixel_transform, user_actions)
                }
                Tool::Paint(tool) => tool.update_ui(
                    hover_pos,
                    ui,
                    &response,
                    &painter,
                    &pixel_transform,
                    cursor_icon,
                    &self.ui_state.mode,
                    (self.ui_state.primary_color, self.ui_state.secondary_color),
                    &self.doc,
                    user_actions,
                ),
                Tool::Grab(tool) => tool.update_ui(
                    &painter,
                    &pixel_transform,
                    cursor_icon,
                    &self.doc,
                    hover_pos,
                    &response,
                    user_actions,
                ),
                Tool::CharBrush(tool) => tool.update_ui(
                    &response,
                    &painter,
                    &pixel_transform,
                    cursor_icon,
                    brush,
                    hover_pos,
                    &self.doc,
                    user_actions,
                ),
            };
        }

        let info_text = {
            let t = self.doc.image.image_info();
            if let Some(p) = hover_pos {
                format!("{}\n{}", self.doc.image.pixel_info(p), t)
            } else {
                t
            }
        };
        painter.text(
            response.rect.left_bottom(),
            Align2::LEFT_BOTTOM,
            &info_text,
            TextStyle::Monospace,
            Color32::from_rgb(0x88, 0x88, 0x88),
        );

        if let Some(icon) = cursor_icon {
            ctx.output().cursor_icon = *icon;
        }
    }

    /// Apply an action and record it in the history. Show any error to the user.
    /// If the action was not handled, returns the action to the caller.
    pub fn apply_action(&mut self, action: Action) -> Option<Action> {
        let Editor {
            doc,
            history,
            ui_state,
            ..
        } = self;

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
            Action::Ui(ref ui_action) => match ui_action {
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
                UiAction::SelectTool(tool) => ui_state.tool = tool.clone(),
                UiAction::SelectMode(mode) => ui_state.mode = mode.clone(),
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
                    ui_state.zoom = *amount;
                }
                UiAction::ToggleGrid => ui_state.grid = !ui_state.grid,
                UiAction::ToggleRaw => {
                    ui_state.image_view_settings = match ui_state.image_view_settings {
                        ViewSettings::Normal => ViewSettings::Raw,
                        ViewSettings::Raw => ViewSettings::Normal,
                    }
                }
                UiAction::ViewSettings(settings) => {
                    ui_state.image_view_settings = settings.clone();
                }
                // Not handled by Editor
                UiAction::NewDocument(_)
                | UiAction::CloseEditor(_)
                | UiAction::CreateCharBrush { .. }
                | UiAction::MirrorBrushX
                | UiAction::MirrorBrushY => {
                    return Some(action);
                }
            },
        }
        None
    }
}

/// Ask for filename and save the document. Show any error message to the user.
/// Returns false if the file was not saved, either because user cancelled or there was an error.
fn save_as(
    history: &mut Record<actions::Undoable>,
    doc: &mut Document,
    system: &mut dyn SystemFunctions,
) -> bool {
    match system.save_file_dialog(SaveFileOptions::for_save(doc.filename.as_deref())) {
        Ok(Some(filename)) => save(history, doc, &filename, system),
        Ok(None) => false,
        Err(e) => {
            system.show_error(&format!("Could not get file name: {:?}", e));
            false
        }
    }
}

/// Ask for filename and export the document.
fn export(doc: &Document, system: &mut dyn SystemFunctions) {
    match system.save_file_dialog(SaveFileOptions::for_export(doc.filename.as_deref())) {
        Ok(Some(filename)) => {
            if let Err(e) = storage::save_any_file(doc, &filename) {
                system.show_error(&format!("Failed to save image: {}", e));
            }
        }
        Ok(None) => {}
        Err(e) => {
            system.show_error(&format!("Could not get file name: {:?}", e));
        }
    }
}

/// Save the document as a given filename.
/// Ask for filename and save the document. Show any error message to the user.
/// Returns false if the file was not saved, either because user cancelled or there was an error.
fn save(
    history: &mut Record<actions::Undoable>,
    doc: &mut Document,
    filename: &Path,
    system: &mut dyn SystemFunctions,
) -> bool {
    println!("Saving as {}", filename.display());
    match storage::save(doc, filename) {
        Ok(()) => {
            doc.filename = Some(filename.to_owned());
            history.set_saved(true);
            true
        }
        Err(e) => {
            system.show_error(&format!("Failed to save: {:?}", e));
            false
        }
    }
}

/// Create a Response and Painter for the main image area.
fn image_painter(ui: &mut egui::Ui) -> (Response, Painter) {
    let size = ui.available_size();
    let response = ui.allocate_response(size, egui::Sense::click_and_drag());
    let clip_rect = ui.clip_rect().intersect(response.rect);
    let painter = Painter::new(ui.ctx().clone(), ui.layer_id(), clip_rect);
    (response, painter)
}

fn draw_grid(image: &VicImage, painter: &Painter, pixel_transform: &PixelTransform) {
    let (width, height) = image.size_in_pixels();
    let stroke = Stroke {
        width: 1.0,
        color: GRID_COLOR,
    };
    for x in image.vertical_grid_lines() {
        painter.line_segment(
            [
                pixel_transform.screen_pos(PixelPoint::new(x, 0)),
                pixel_transform.screen_pos(PixelPoint::new(x, height as i32)),
            ],
            stroke,
        )
    }
    for y in image.horizontal_grid_lines() {
        painter.line_segment(
            [
                pixel_transform.screen_pos(PixelPoint::new(0, y)),
                pixel_transform.screen_pos(PixelPoint::new(width as i32, y)),
            ],
            stroke,
        )
    }
}

/// Renders the UI for tool selection.
/// Returns which tool to switch to, or None if the user did not change tool.
fn select_tool_ui(ui: &mut egui::Ui, current_tool: &Tool, user_actions: &mut Vec<Action>) {
    let mut new_tool = None;
    ui.with_layout(egui::Layout::top_down_justified(Align::LEFT), |ui| {
        ui.style_mut().body_text_style = egui::TextStyle::Heading;
        ui.label("Tool");
        if ui
            .selectable_label(matches!(current_tool, Tool::Paint(_)), "Paint")
            .on_hover_text("Paint pixels")
            .clicked()
        {
            new_tool = Some(Tool::Paint(Default::default()));
        }
        if ui
            .selectable_label(matches!(current_tool, Tool::Grab { .. }), "Grab")
            .on_hover_text("Create a brush from a part of the picture")
            .clicked()
        {
            new_tool = Some(Tool::Grab(Default::default()));
        }
        if ui
            .selectable_label(matches!(current_tool, Tool::CharBrush { .. }), "Char Brush")
            .on_hover_text("Draw with a character brush")
            .clicked()
        {
            new_tool = Some(Tool::CharBrush(Default::default()));
        }
    });
    if let Some(t) = new_tool {
        user_actions.push(Action::Ui(UiAction::SelectTool(t)));
    }
}

/// Renders the UI for mode selection.
fn select_mode_ui(ui: &mut egui::Ui, current_mode: &Mode, user_actions: &mut Vec<Action>) {
    ui.with_layout(egui::Layout::top_down_justified(Align::LEFT), |ui| {
        ui.style_mut().body_text_style = egui::TextStyle::Heading;
        ui.label("Mode");
        for mode in [
            Mode::PixelPaint,
            Mode::FillCell,
            Mode::CellColor,
            Mode::ReplaceColor,
            Mode::SwapColors,
            Mode::MakeHiRes,
            Mode::MakeMulticolor,
        ] {
            if ui
                .selectable_label(*current_mode == mode, mode.title())
                .on_hover_text(mode.tip())
                .clicked()
            {
                user_actions.push(Action::Ui(UiAction::SelectMode(mode)));
            }
        }
    });
}
