# Changelog

<!-- next-header -->

## [Unreleased] - ReleaseDate
### Added
- Create image from brush.
- If you select a primary color that is the same as the secondary color, swap them instead of selecting the same color for both.

### Changed
- Render the secondary color marker as an outline, so it's easier to distinguish from the primary color.

### Changed
- Improve clarity in file format:
  Global colors are stored as a JSON dictionary with the keys "background", "border", and "aux" instead of a list. Loading old format is supported.

## [0.14.0] - 2022-03-14
### Added
- Rectangle (filled) tool.

## [0.13.0] - 2022-02-26
### Added
- Can open multiple documents from the command-line.
- Can mirror the brush horizontally (x) or vertically (y).

### Changed
- How the hair-cross and grabbing rectangle reacts to the mouse movement changed in an attempt to make if feel better.

### Fixed
- Opening a file that is already open switches to that tab instead of opening a duplicate.

## [0.12.0] - 2022-02-21
### Added
- Keyboard shortcuts for many functions. See [README](README.md).
- Can have multiple documents open simultaneously.
- File->New menu item creates a new document.

### Fixed
- File and confirmation dialogs no longer crash on Mac.

## [0.11.0] - 2022-01-09
### Added
- Possible to undo importing an image.
- Grab tool for creating a brush from a part of an image, and Char Brush tool for drawing with it.
- Selecting one of the global colors is now undoable.
- Importing an image is now undoable.

## [0.10.0] - 2022-01-04
### Added
- "Replace Color" tool to replace one color with another.
- "Swap Colors" tool to replace one color with another, and vice versa.

### Changed
- Renamed "Color Paint" tool to "Cell Color" to better explain what it does.
- When dragging the mouse while painting, fill in all intermediate pixels between mouse position,
so painting becomes a continuous line.

## [0.9.0] - 2021-12-01
### Changed
- UI change: Show background, border, and aux as separate color patches.
This makes it impossible to try to draw with colors 8-15 unless in a multicolor cell, and then only using aux or bg.

### Added
- Possible to select a secondary color, used to draw with the right mouse button, instead of always using background.
- In Fill Cell, Color Paint, Make High-Res, and Make Multicolor modes, highlight the cell that will be affected. (Was only in Color Paint mode.)

## [0.8.0] - 2021-11-14
### Added
- "Fill Cell" tool for quickly filling a character cell with a color.

## [0.7.0] - 2021-11-13
### Added
- "Raw" display mode for inspecting the image without colors

## [0.6.0] - 2021-10-19
### Added
- Undo and redo

## [0.5.0] - 2021-10-17
### Added
- "Make High-Res" and "Make Multicolor" paint modes
- Python script for converting Pixel Pen format to binary Vic-20 format.

## [0.4.0] - 2021-10-14
### Added
- Possible to import with twice the pixel aspect ratio, facilitating loading multicolor images stored as one low resolution pixel per pixel.
- Possible to export to image file from command-line interface.

### Fixed
- Palette colors that look more like the actual Vic-20 colors.

## [0.3.0] - 2021-10-07
### Added
- Export to image file
- Possible to import with match pixel aspect ratio, facilitating round-trip to other image editor.

## [0.2.2] - 2021-10-05
### Fixed
- Char color changed on set pixel with bg color

## [0.2.1] - 2021-10-05

## [0.2.0] - 2021-10-05
### Added
- Setting pixels and colors.
- Save, load, and import


<!-- next-url -->
[Unreleased]: https://github.com/vilcans/pixel_pen/compare/v0.14.0...HEAD
[0.14.0]: https://github.com/vilcans/pixel_pen/compare/v0.13.0...v0.14.0
[0.13.0]: https://github.com/vilcans/pixel_pen/compare/v0.12.0...v0.13.0
[0.12.0]: https://github.com/vilcans/pixel_pen/compare/v0.11.0...v0.12.0
[0.11.0]: https://github.com/vilcans/pixel_pen/compare/v0.10.0...v0.11.0
[0.10.0]: https://github.com/vilcans/pixel_pen/compare/v0.9.0...v0.10.0
[0.9.0]: https://github.com/vilcans/pixel_pen/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/vilcans/pixel_pen/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/vilcans/pixel_pen/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/vilcans/pixel_pen/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/vilcans/pixel_pen/compare/pixel_pen-v0.4.0...v0.5.0
[0.4.0]: https://github.com/vilcans/pixel_pen/compare/v0.3.0...pixel_pen-v0.4.0
[0.3.0]: https://github.com/vilcans/pixel_pen/compare/pixel_pen-v0.2.2...v0.3.0
[0.2.2]: https://github.com/vilcans/pixel_pen/compare/pixel_pen-v0.2.1...pixel_pen-v0.2.2
[0.2.1]: https://github.com/vilcans/pixel_pen/compare/pixel_pen-v0.2.0...pixel_pen-v0.2.1
[0.2.0]: https://github.com/vilcans/pixel_pen/releases/tag/pixel_pen-v0.2.0
