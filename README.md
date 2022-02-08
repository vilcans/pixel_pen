# Pixel Pen

*Graphics editor for pictures compatible with Vic-20 hardware.*

This is still early in development.

You can load images in typical image file formats, which Pixel Pen will convert to Vic-20 format.
If you use the "Open" menu to open an image file, make sure it's not very high resolution. You can use the Import menu instead to load an image and scale it down at the same time.

It's also possible to load some of the files using the FLUFF64 file format (file extension: `flf`) used by [Turbo Rascal](https://lemonspawn.com/turbo-rascal-syntax-error-expected-but-begin/) (TRSE),
but that was only to get some images to test with at the start of the project. It's not a high priority to support all the formats of TRSE.

There is also Pixel Pen's own file format (file extension: `pixelpen`), the details of which are subject to change, but it's based on JSON. Pixel Pen can save and load files in this format.

Apart from that, you can paint single pixels, change a character cell's color, and switch a character cell to multicolor or high resolution.

## The Palette

The color swatches at the top of the window is the palette.
Left-click on one of then to select the primary color to use when painting.
Right-click on one of them to select the secondary color, which you use instead of the primary color when painting with the right mouse button.
Click one of the *Background*, *Border*, or *Aux* buttons to select which color to use for each one of those.
Background and Aux can be any of the Vic's 16 colors, while Border can only be one of 8.

## Tools

There are a few tools that you can select.

### Paint

With this tool, you draw pixels. See Paint Modes below for information about how the Paint tool affects the pixels.

### Grab

When you have selected the Grab tool, you can click on a cell to copy it into a brush.
You can also select several cells to create a brush from by holding the mouse button and dragging a rectangular selection.
The brush you grab this way can then be used with the Char Brush tool.
After you have grabbed a selection, Pixel Pen will switch to the Char Brush tool automatically.

### Char Brush

In the Char Brush mode, you can paint with a grabbed brush. Left click to draw with the brush on the image.

## Paint Modes

When using the Paint tool, it's possible to select which mode to draw in.
This affects how the pixels change when you paint on them.

### Pixel Paint

In this mode, select a color from the palette and left-click to paint with it.
Note that each cell (8 by 8 pixels) can contain only 2 colors in high-res mode,
and 4 colors in multicolor mode.

You can use the right mouse button to paint with the background color.

### Fill Cell

In Fill Cell mode, select a color from the palette and left-click to fill the whole character cell with that color.
Right-clicking fills the cell with the background color.
This is useful for quickly filling large areas with a color.

### Cell Color

In Cell Color mode, select a color from the palette and left-click to change the character color of a cell.

### Replace Color

In Replace Color mode, select a primary color by left-clicking on the palette. Select a secondary color by right-clicking on the palette. Left-click on the image to replace the secondary color with the primary color. You can also right-click to replace in the other direction, i.e. replacing the primary color with the secondary.

### Swap Colors

In Swap Colors mode, select one color by left-clicking on the palette and another one by right-clicking on the palette. Click on the image to replace one color with the other.

### Make High-res

The Make High-res tool changes a character cell to high-resolution mode.
In high-res mode, the cell can contain pixels of two colors: the background color and the character color.
There are 8 by 8 pixels in the cell in this mode.

### Make Multicolor

The Make Multicolor tool changes a character cell to multicolor mode.
In multicolor mode, the cell can contain pixels of four colors: the background color, the border color, the aux color, and the character color.
There are 4 by 8 pixels in the cell in this mode, and each pixel is twice as wide as in high-res mode.

## View Settings

### Grid

The Grid checkbox displays a grid so you can see the borders of each cell.

### Raw

The Raw checkbox changes the display mode to "raw". This mode is useful to "debug" the image, or understand how it's built. In raw mode, the selected colors for background, border, aux, and the cell's character color are not used. Instead they are displayed as:

  * Gray = background in hi-res cells
  * White = character color in hi-res cells
  * Black = background color in multicolor cells
  * Blue = border color in multicolor cells
  * Red = aux color in multicolor cells
  * White = character color in multicolor cells

## Input

| Input                               | Action
| ----------------------------------- | ---------------------------------
| Left mouse button                   | Paint
| Right mouse button                  | Paint with background color
| +                                   | Zoom in
| -                                   | Zoom out
| Z                                   | Undo
| Y                                   | Redo
| G                                   | Grid on/off
| W                                   | Raw display on/off
| Scroll wheel                        | Scroll
| Ctrl + scroll wheel                 | Zoom
| Hold middle mouse button            | Pan
| Hold shift + right mouse button     | Pan (for systems without a middle mouse button or where it scrolls instead)
On a Mac, substitute Ctrl for ⌘.

## Command-line interface

Oh, and there's a command-line interface! Run `pixel_pen --help` to get the possible commands:

    Pixel Pen 0.6.0
    Actual 8 bit graphics editor

    USAGE:
        pixel_pen [OPTIONS] [filename]

    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information

    OPTIONS:
            --import <import-file>    Open the given file in import mode
            --save <save-file>        Save the image to the given file and quit. File may be in pixelpen format or the image
                                      may be exported as a standard image file

    ARGS:
        <filename>    File to load

So it can be used to convert images from any supported format to any other format.
For example, to convert a `.pixelpen` file to `.png` format:

    pixel_pen file.pixelpen --save file.png

Or, to convert a `.png` file to `.pixelpen` format:

    pixel_pen file.png --save file.pixelpen

Note that this (as of version 0.6.0) does not resize the source image, so if it has high resolution, it won't be suitable for the target platform.
Use the Import menu item instead to scale the image down before exporting it.

# Changelog

See [Changelog](CHANGELOG.md).

# Development

See [Development](DEVELOPMENT.md)
