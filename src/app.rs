use std::{path::Path, time::Instant};

use crate::{
    actions::{self, Action, Undoable},
    colors::TrueColor,
    coords::{PixelTransform, Point},
    document::Document,
    editing::Mode,
    error::{Error, Severity},
    import::{self, Import, ImportSettings},
    mutation_monitor::MutationMonitor,
    storage,
    system::{self, OpenFileOptions, SaveFileOptions, SystemFunctions},
    tool::Tool,
    ui::{self, UiState},
    vic::{Char, PaintColor, VicImage, ViewSettings},
};
use eframe::{
    egui::{
        self, paint::Mesh, Align2, Color32, CursorIcon, Label, Painter, PointerButton, Pos2, Rect,
        Response, Rgba, Shape, Stroke, TextStyle, TextureId, Vec2,
    },
    epi::{self, TextureAllocator},
};
use image::imageops::FilterType;
use imgref::ImgVec;
use undo::Record;

// Don't scale the texture more than this to avoid huge textures when zooming.
const MAX_SCALE: u32 = 8;

const POPUP_MESSAGE_TIME: f32 = 3.0;
const POPUP_HIGHLIGHT_TIME: f32 = 0.4;
const POPUP_FADE_OUT_TIME: f32 = 0.8;

const BORDER_CORNER_RADIUS: f32 = 15.0;
const BORDER_SIZE: Vec2 = Vec2::new(25.0, 20.0);

const GRID_COLOR: Color32 = Color32::GRAY;
const MAKE_HIRES_HIGHLIGHT: Stroke = Stroke {
    width: 2.0,
    color: Color32::from_rgb(200, 200, 200),
};
const MAKE_MULTICOLOR_HIGHLIGHT: Stroke = Stroke {
    width: 2.0,
    color: Color32::from_rgb(255, 255, 255),
};

const GRID_TOOLTIP: &str = "Show character cell grid";

const RAW_TOOLTIP: &str = "Show image with fixed colors:
• Gray = background color in hi-res cells
• Black = background color in multicolor cells
• White = character color
• Blue = border color in multicolor cells
• Red = aux color in multicolor cells";

struct Texture {
    pub id: TextureId,
    pub settings: ViewSettings,
    pub width: usize,
    pub height: usize,
}

#[derive(Default)]
struct UserActions {
    pub zoom_in: bool,
    pub zoom_out: bool,
    pub toggle_grid: bool,
    pub undo: bool,
    pub redo: bool,
}
impl UserActions {
    pub fn update_from_text(&mut self, t: &str) {
        match t {
            "+" => self.zoom_in = true,
            "-" => self.zoom_out = true,
            "g" => self.toggle_grid = true,
            "z" => self.undo = true,
            "y" => self.redo = true,
            _ => (),
        }
    }
}

pub struct Application {
    doc: Document,
    ui_state: UiState,
    image_texture: Option<Texture>,
    history: Record<actions::Undoable>,
    pub system: Box<dyn SystemFunctions>,
}

impl Default for Application {
    fn default() -> Self {
        let doc = Document::default();
        Self::with_doc(doc)
    }
}

impl epi::App for Application {
    fn name(&self) -> &str {
        "Pixel Pen"
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        let Application {
            doc,
            ui_state,
            image_texture,
            system,
            history,
            ..
        } = self;
        let undo_available = history.can_undo();
        let redo_available = history.can_redo();

        let (width, height) = doc.image.pixel_size();
        let mut new_doc = None;
        let mut cursor_icon = None;

        let mut user_actions = UserActions::default();
        for e in ctx.input().events.iter() {
            if !ctx.wants_keyboard_input() {
                if let egui::Event::Text(t) = e {
                    user_actions.update_from_text(t)
                }
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // Menu bar
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if system.has_open_file_dialog() && ui.button("Open...").clicked() {
                        match system
                            .open_file_dialog(OpenFileOptions::for_open(doc.filename.as_deref()))
                        {
                            Ok(Some(filename)) => {
                                match storage::load_any_file(std::path::Path::new(&filename)) {
                                    Ok(doc) => {
                                        new_doc = Some(doc);
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
                    if system.has_open_file_dialog() && ui.button("Import...").clicked() {
                        match system.open_file_dialog(OpenFileOptions::for_import(match &ui_state
                            .tool
                        {
                            Tool::Import(Import {
                                settings: ImportSettings { filename, .. },
                                ..
                            }) => filename.as_deref(),
                            _ => None,
                        })) {
                            Ok(Some(filename)) => {
                                match start_import_mode(&filename, doc, ui_state) {
                                    Ok(()) => {}
                                    Err(e) => system.show_error(&format!(
                                        "Could not import file {}: {:?}",
                                        filename.display(),
                                        e
                                    )),
                                }
                            }
                            Ok(None) => {}
                            Err(e) => {
                                system.show_error(&format!("Could not get file name: {:?}", e))
                            }
                        }
                    }
                    if system.has_save_file_dialog() {
                        ui.separator();
                        match doc.filename.clone() {
                            Some(filename) => {
                                if ui
                                    .button(format!(
                                        "Save {}",
                                        filename
                                            .file_name()
                                            .map(|s| s.to_string_lossy())
                                            .unwrap_or_default()
                                    ))
                                    .clicked()
                                {
                                    save(history, doc, &filename, system);
                                }
                            }
                            None => {
                                if ui.button("Save").clicked() {
                                    save_as(history, doc, system);
                                }
                            }
                        }
                        if ui.button("Save As...").clicked() {
                            save_as(history, doc, system);
                        }
                        if ui.button("Export...").clicked() {
                            export(doc, system);
                        }
                    }
                    ui.separator();
                    if ui.button("Quit").clicked()
                        && (history.is_saved()
                            || check_quit(system.as_mut(), doc.filename.as_deref()))
                    {
                        frame.quit();
                    }
                });
                egui::menu::menu(ui, "Edit", |ui| {
                    ui.set_enabled(undo_available);
                    if ui.button("Undo").clicked() {
                        user_actions.undo = true;
                    }
                    ui.set_enabled(redo_available);
                    if ui.button("Redo").clicked() {
                        user_actions.redo = true;
                    }
                });
            });

            // Top toolbar
            ui.vertical(|ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label("Zoom:");
                    user_actions.zoom_out |= ui.button("-").on_hover_text("Zoom out").clicked();
                    if ui
                        .button(format!("{:0.0}x", ui_state.zoom))
                        .on_hover_text("Set to 2x")
                        .clicked()
                    {
                        ui_state.zoom = 2.0;
                    }
                    user_actions.zoom_in |= ui.button("+").on_hover_text("Zoom in").clicked();
                    ui.separator();
                    ui.checkbox(&mut ui_state.grid, "Grid")
                        .on_hover_text(GRID_TOOLTIP);
                    let mut raw_mode = ui_state.image_view_settings == ViewSettings::Raw;
                    ui.checkbox(&mut raw_mode, "Raw").on_hover_text(RAW_TOOLTIP);
                    ui_state.image_view_settings = if raw_mode {
                        ViewSettings::Raw
                    } else {
                        ViewSettings::Normal
                    };
                });
                ui.separator();
                ui::palette::render_palette(
                    ui,
                    &mut ui_state.primary_color,
                    &mut ui_state.secondary_color,
                    &mut doc.image,
                );
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            if let Some((time, message)) = ui_state.message.as_ref() {
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
                    ui_state.message = None;
                } else if highlight > 0.0 {
                    ctx.request_repaint(); // to animate color highlight
                }
            } else {
                ui.label(ui_state.tool.instructions(&ui_state.mode));
            }
        });

        // Left toolbar
        egui::SidePanel::left("toolbar").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                if let Some(new_tool) = select_tool_ui(ui, &ui_state.tool) {
                    ui_state.tool = new_tool;
                }
                ui_state.mode = select_mode_ui(ui, &ui_state.mode);
            });
        });

        // Main image.
        egui::CentralPanel::default().show(ctx, |ui| {
            let par = doc.image.pixel_aspect_ratio();
            let (response, painter) = image_painter(ui);
            let pixel_transform = PixelTransform {
                screen_rect: Rect::from_center_size(
                    response.rect.center() + ui_state.pan,
                    Vec2::new(
                        width as f32 * par * ui_state.zoom,
                        height as f32 * ui_state.zoom,
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
                    user_actions.zoom_out = true;
                } else if input.scroll_delta.y > 0.0 {
                    user_actions.zoom_in = true;
                }
            } else {
                ui_state.pan += input.scroll_delta;
            }

            if response.drag_started() && input.pointer.button_down(PointerButton::Middle)
                || (input.pointer.button_down(PointerButton::Secondary) && input.modifiers.shift)
            {
                ui_state.panning = true;
            }
            if ui_state.panning {
                ui_state.pan += input.pointer.delta();
                cursor_icon = Some(CursorIcon::Grabbing);
            } else {
                let action = match &mut ui_state.tool {
                    Tool::Import(_) => None,
                    Tool::Paint(tool) => tool.update_ui(
                        hover_pos,
                        ui,
                        &response,
                        &mut cursor_icon,
                        &ui_state.mode,
                        (ui_state.primary_color, ui_state.secondary_color),
                    ),
                    Tool::Grab(tool) => tool.update_ui(doc, hover_pos, &response),
                    Tool::CharBrush(tool) => {
                        tool.update_ui(&response, &ui_state.char_brush, hover_pos, doc)
                    }
                };
                if let Some(action) = action {
                    apply_action(doc, history, ui_state, action);
                }
            }
            if response.drag_released() {
                ui_state.panning = false;
            }

            painter.rect_filled(
                pixel_transform
                    .screen_rect
                    .expand2(BORDER_SIZE * ui_state.zoom),
                BORDER_CORNER_RADIUS * ui_state.zoom,
                doc.image.border(),
            );

            // Draw the main image
            let texture = update_texture(
                &mut doc.image,
                image_texture,
                frame.tex_allocator(),
                par,
                ui_state.zoom,
                &ui_state.image_view_settings,
            );
            let mut mesh = Mesh::with_texture(texture);
            mesh.add_rect_with_uv(
                pixel_transform.screen_rect,
                Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
            painter.add(Shape::Mesh(mesh));

            // Grid lines
            if ui_state.grid {
                draw_grid(&doc.image, &painter, &pixel_transform);
            }

            // Import preview
            if let Tool::Import(import) = &mut ui_state.tool {
                import::image_ui(ui, &painter, import, &pixel_transform);
            }

            // Highlight character
            if let Some(pos) = hover_pos {
                if let Some((top_left, w, h)) = doc.image.character_box(pos) {
                    if let Some(stroke) = match ui_state.mode {
                        Mode::FillCell | Mode::CellColor => Some(Stroke {
                            width: 1.0,
                            color: doc
                                .image
                                .true_color_from_paint_color(&ui_state.primary_color)
                                .into(),
                        }),
                        Mode::MakeHiRes => Some(MAKE_HIRES_HIGHLIGHT),
                        Mode::MakeMulticolor => Some(MAKE_MULTICOLOR_HIGHLIGHT),
                        _ => None,
                    } {
                        painter.rect_stroke(
                            Rect::from_min_max(
                                pixel_transform.screen_pos(top_left),
                                pixel_transform
                                    .screen_pos(Point::new(top_left.x + w, top_left.y + h)),
                            ),
                            0.0,
                            stroke,
                        );
                    }
                }
            }

            let info_text = {
                let t = doc.image.image_info();
                if let Some(p) = hover_pos {
                    format!("{}\n{}", doc.image.pixel_info(p), t)
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
        });

        if let Tool::Import(import) = &mut ui_state.tool {
            let mut action = None;
            egui::Window::new("Import").show(ctx, |ui| action = import::tool_ui(ui, doc, import));
            if let Some(action) = action {
                apply_action(doc, history, ui_state, action);
            }
        }

        if user_actions.zoom_in && ui_state.zoom < 16.0 {
            ui_state.zoom *= 2.0
        }
        if user_actions.zoom_out && ui_state.zoom > 1.0 {
            ui_state.zoom /= 2.0
        }
        if user_actions.toggle_grid {
            ui_state.grid = !ui_state.grid;
        }
        if user_actions.undo && undo_available {
            history.undo(doc);
            doc.image.dirty = true;
        }
        if user_actions.redo && redo_available {
            history.redo(doc);
            doc.image.dirty = true;
        }

        if let Some(doc) = new_doc {
            self.doc = doc;
        }

        if let Some(icon) = cursor_icon {
            ctx.output().cursor_icon = icon;
        }
    }
}

/// Renders the UI for tool selection.
/// Returns which tool to switch to, or None if the user did not change tool.
fn select_tool_ui(ui: &mut egui::Ui, current_tool: &Tool) -> Option<Tool> {
    let mut new_tool = None;
    ui.add(Label::new("Tool").strong());
    ui.vertical_centered_justified(|ui| {
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
    new_tool
}

/// Renders the UI for mode selection.
/// Returns which mode to use, which is the same as the current one passed in unless changed by the user.
fn select_mode_ui(ui: &mut egui::Ui, current_mode: &Mode) -> Mode {
    let mut new_mode = current_mode.clone();
    ui.add(Label::new("Mode").strong());
    ui.vertical_centered_justified(|ui| {
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
                new_mode = mode;
            }
        }
    });
    new_mode
}

/// Ask for filename and save the document. Show any error message to the user.
/// Returns false if the file was not saved, either because user cancelled or there was an error.
fn save_as(
    history: &mut Record<actions::Undoable>,
    doc: &mut Document,
    system: &mut Box<dyn SystemFunctions>,
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
fn export(doc: &Document, system: &mut Box<dyn SystemFunctions>) {
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
    system: &mut Box<dyn SystemFunctions>,
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

fn start_import_mode(
    filename: &Path,
    doc: &mut Document,
    ui_state: &mut UiState,
) -> Result<(), Error> {
    let mut i = Import::load(filename)?;
    i.settings.width = i.settings.width.min(doc.image.pixel_size().0 as u32);
    i.settings.height = i.settings.height.min(doc.image.pixel_size().1 as u32);
    ui_state.tool = Tool::Import(i);
    Ok(())
}

fn draw_grid(image: &VicImage, painter: &Painter, pixel_transform: &PixelTransform) {
    let (width, height) = image.pixel_size();
    let stroke = Stroke {
        width: 1.0,
        color: GRID_COLOR,
    };
    for x in image.vertical_grid_lines() {
        painter.line_segment(
            [
                pixel_transform.screen_pos(Point { x, y: 0 }),
                pixel_transform.screen_pos(Point {
                    x,
                    y: height as i32,
                }),
            ],
            stroke,
        )
    }
    for y in image.horizontal_grid_lines() {
        painter.line_segment(
            [
                pixel_transform.screen_pos(Point { x: 0, y }),
                pixel_transform.screen_pos(Point { x: width as i32, y }),
            ],
            stroke,
        )
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
            actions::UiAction::SelectTool(tool) => ui_state.tool = tool,
            actions::UiAction::CreateCharBrush {
                column,
                row,
                width,
                height,
            } => {
                ui_state.char_brush = doc.image.grab_cells(column, row, width, height);
                ui_state.tool = Tool::CharBrush(Default::default());
            }
        },
    }
}

/// Updates the texture with the current image content, if needed.
/// Returns the texture id.
fn update_texture(
    image: &mut MutationMonitor<VicImage>,
    image_texture: &mut Option<Texture>,
    tex_allocator: &mut dyn TextureAllocator,
    par: f32,
    zoom: f32,
    settings: &ViewSettings,
) -> TextureId {
    let scale_x = ((par * zoom).ceil() as u32).max(1).min(MAX_SCALE);
    let scale_y = (zoom.ceil() as u32).max(1).min(MAX_SCALE);
    let (source_width, source_height) = image.pixel_size();
    let texture_width = source_width * scale_x as usize;
    let texture_height = source_height * scale_y as usize;

    // Recreate the texture if the size has changed or the image has been updated
    if let Some(t) = image_texture {
        if t.settings != *settings
            || t.width != texture_width
            || t.height != texture_height
            || image.dirty
        {
            tex_allocator.free(t.id);
            *image_texture = None;
        }
    }
    if image.dirty {
        image.update();
    }
    let texture = if let Some(texture) = image_texture {
        texture.id
    } else {
        let unscaled_image = image.render_with_settings(settings);
        let scaled_image = image::imageops::resize(
            &unscaled_image,
            unscaled_image.width() * scale_x,
            unscaled_image.height() * scale_y,
            FilterType::Nearest,
        );
        let pixels: Vec<Color32> = scaled_image
            .pixels()
            .map(|p| (<image::Rgba<u8> as Into<TrueColor>>::into(*p)).into())
            .collect();
        let texture_id =
            tex_allocator.alloc_srgba_premultiplied((texture_width, texture_height), &pixels);
        *image_texture = Some(Texture {
            id: texture_id,
            settings: settings.clone(),
            width: texture_width,
            height: texture_height,
        });
        texture_id
    };
    image.dirty = false;
    texture
}

impl Application {
    pub fn with_doc(doc: Document) -> Self {
        let system = Box::new(system::DummySystemFunctions {});
        Application {
            history: Record::new(),
            ui_state: UiState {
                tool: Tool::Paint(Default::default()),
                mode: Mode::PixelPaint,
                zoom: 2.0,
                image_view_settings: ViewSettings::Normal,
                primary_color: PaintColor::CharColor(7),
                secondary_color: PaintColor::Background,
                char_brush: ImgVec::new(vec![Char::DEFAULT_BRUSH], 1, 1),
                grid: false,
                panning: false,
                pan: Vec2::ZERO,
                message: None,
            },
            doc,
            image_texture: None,
            system,
        }
    }

    pub fn start_import_mode(&mut self, filename: &Path) -> Result<(), Error> {
        start_import_mode(filename, &mut self.doc, &mut self.ui_state)
    }
}
