use crate::{actions::Action, update_area::UpdateArea, vic::PaintColor};

/// In what way an edit operation changes the pixels or character.
#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    PixelPaint,
    FillCell,
    CellColor,
    MakeHiRes,
    MakeMulticolor,
    ReplaceColor,
    SwapColors,
}

impl Mode {
    pub fn title(&self) -> &str {
        match self {
            Mode::PixelPaint => "Pixel Paint",
            Mode::FillCell => "Fill Cell",
            Mode::CellColor => "Cell Color",
            Mode::MakeHiRes => "Make Hi-Res",
            Mode::MakeMulticolor => "Make Multicolor",
            Mode::ReplaceColor => "Replace Color",
            Mode::SwapColors => "Swap Colors",
        }
    }

    pub fn tip(&self) -> &str {
        match self {
            Mode::PixelPaint => "Paint pixels",
            Mode::FillCell => "Fill the whole character cell with a color",
            Mode::CellColor => "Change the color of character cells",
            Mode::MakeHiRes => "Set character cells to high-resolution mode",
            Mode::MakeMulticolor => "Set character cells to multicolor mode",
            Mode::ReplaceColor => "Replace one color with another",
            Mode::SwapColors => "Swap two colors",
        }
    }

    pub fn instructions(&self) -> &'static str {
        match self {
            Mode::PixelPaint => "Click to paint. Right-click to paint with background color.",
            Mode::FillCell => {
                "Click to fill the character cell with a color. Right-click to fill with background color."
            }
            Mode::CellColor => {
                "Click to change the color of the character cell. Right-click for background color."
            }
            Mode::MakeHiRes => "Click to make the character cell high-resolution.",
            Mode::MakeMulticolor => "Click to make the character cell multicolor.",
            Mode::ReplaceColor => "Click to replace secondary color with primary color. Right-click for the inverse.",
            Mode::SwapColors => "Click to replace primary color with secondary color and vice versa.",
        }
    }

    /// Create an Action from a paint Mode.
    pub fn paint_action(
        &self,
        area: UpdateArea,
        color: PaintColor,
        other_color: PaintColor,
    ) -> Action {
        match self {
            Mode::PixelPaint => Action::Plot { area, color },
            Mode::FillCell => Action::Fill { area, color },
            Mode::CellColor => Action::CellColor { area, color },
            Mode::MakeHiRes => Action::MakeHighRes { area },
            Mode::MakeMulticolor => Action::MakeMulticolor { area },
            Mode::ReplaceColor => Action::ReplaceColor {
                area,
                to_replace: other_color,
                replacement: color,
            },
            Mode::SwapColors => Action::SwapColors {
                area,
                color_1: color,
                color_2: other_color,
            },
        }
    }
}
