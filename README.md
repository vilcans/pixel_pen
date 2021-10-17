# Pixel Pen

*Graphics editor for pictures compatible with Vic-20 hardware.*

This is still early in development.

You can load images in typical image file formats, which Pixel Pen will convert to Vic-20 format.
If you use the "Open" menu to open an image file, make sure it's not very high resolution. You can use the Import menu instead to load an image and scale it down at the same time.

It's also possible to load some of the files using the FLUFF64 file format (file extension: `flf`) used by [Turbo Rascal](https://lemonspawn.com/turbo-rascal-syntax-error-expected-but-begin/) (TRSE),
but that was only to get some images to test with at the start of the project. It's not a high priority to support all the formats of TRSE.

There is also Pixel Pen's own file format (file extension: `pixelpen`), the details of which are subject to change, but it's based on JSON. Pixel Pen can save and load files in this format.

Apart from that, you can paint single pixels, and that's about it right now.

## Input

| Input                               | Action
| ----------------------------------- | ---------------------------------
| Left mouse button                   | Paint
| Right mouse button                  | Paint with background color
| +                                   | Zoom in
| -                                   | Zoom out
| G                                   | Grid on/off
| Scroll wheel                        | Scroll
| Ctrl + scroll wheel                 | Zoom
| Hold middle mouse button            | Pan
| Hold shift + right mouse button     | Pan (for systems without a middle mouse button or where it scrolls instead)

On a Mac, substitute Ctrl for ⌘.

## Command-line interface

Oh, and there's a command-line interface! Run `pixel_pen --help` to get the possible commands:

    Pixel Pen 0.4.0
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

Note that this (as of version 0.4.0) does not resize the source image, so if it has high resolution, it won't be suitable for the target platform.
Use the Import menu item instead to scale the image down before exporting it.

# Changelog

See [Changelog](CHANGELOG.md).

# Development

See [Development](DEVELOPMENT.md)
