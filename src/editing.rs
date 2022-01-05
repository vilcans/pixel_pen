/// In what way an edit operation changes the pixels or character.
#[derive(Debug)]
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
}
