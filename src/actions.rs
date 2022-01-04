use std::fmt;

use crate::{
    error::{DisallowedAction, Severity},
    update_area::UpdateArea,
    vic::PaintColor,
    Document,
};

pub struct Action {
    pub action: ActionType,
    previous: Option<Document>,
}

impl Action {
    pub fn new(action: ActionType) -> Self {
        Self {
            action,
            previous: None,
        }
    }
}

pub enum ActionType {
    /// Change the color of single pixels
    Plot { area: UpdateArea, color: PaintColor },
    /// Fill the whole character cell with a color
    Fill { area: UpdateArea, color: PaintColor },
    /// Change the color of the cell
    CellColor { area: UpdateArea, color: PaintColor },
    /// Make the cell high-res
    MakeHighRes { area: UpdateArea },
    /// Make the cell multicolor
    MakeMulticolor { area: UpdateArea },
    /// Replace one color with another.
    ReplaceColor {
        area: UpdateArea,
        to_replace: PaintColor,
        replacement: PaintColor,
    },
    /// Swap two colors
    SwapColors {
        area: UpdateArea,
        color_1: PaintColor,
        color_2: PaintColor,
    },
}

impl undo::Action for Action {
    type Target = Document;
    type Output = bool;
    type Error = Box<dyn DisallowedAction>;

    fn apply(&mut self, target: &mut Self::Target) -> undo::Result<Self> {
        let previous = target.clone();
        let image = &mut target.image;
        let result = match &self.action {
            ActionType::Plot { area, color } => image.plot(area, *color),
            ActionType::Fill { area, color } => image.fill_cells(area, *color),
            ActionType::CellColor { area, color } => {
                let c = image.color_index_from_paint_color(color);
                image.set_color(area, c)
            }
            ActionType::MakeHighRes { area } => image.make_high_res(area),
            ActionType::MakeMulticolor { area } => image.make_multicolor(area),
            ActionType::ReplaceColor {
                area,
                to_replace,
                replacement,
            } => image.replace_color(area, *to_replace, *replacement),
            ActionType::SwapColors {
                area,
                color_1,
                color_2,
            } => image.swap_colors(area, *color_1, *color_2),
        };
        match result {
            Ok(true) => {
                self.previous = Some(previous);
                Ok(true)
            }
            Ok(false) => Err(Box::new(NoChange)),
            other => other,
        }
    }

    fn undo(&mut self, target: &mut Self::Target) -> undo::Result<Self> {
        match self.previous.take() {
            Some(previous) => {
                *target = previous;
                Ok(true)
            }
            None => Ok(false),
        }
    }
}

#[derive(Debug)]
struct NoChange;

impl fmt::Display for NoChange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "No change")
    }
}

impl DisallowedAction for NoChange {
    fn severity(&self) -> crate::error::Severity {
        Severity::Silent
    }
}
