#!/usr/bin/env python3

"""For converting a Pixel Pen file to native format.

Writes a file with the video array (character numbers) followed by the character bitmaps.
"""

import json
import codecs


def parse_char(char_hex):
    return codecs.decode(char_hex, 'hex')


def convert(document, left, top, width, height):
    """Convert the given document to native bytes.

    Returns the video array, color array, and the bitmaps.
    """

    image = document['image']
    columns = image['columns']
    rows = image['rows']

    if top is None:
        top = 0
    if left is None:
        left = 0
    if width is None:
        width = columns - left
    if height is None:
        height = rows - top

    if top not in range(0, rows + 1):
        raise Exception("'top' out of range")
    if (top + height) not in range(0, rows + 1):
        raise Exception("'height' out of range: {}".format(height))
    if left not in range(0, columns + 1):
        raise Exception("'columns' out of range")
    if (left + width) not in range(0, columns + 1):
        raise Exception("'width' out of range")

    characters = image['characters']
    if len(characters) > 256:
        raise Exception('More than 256 characters used')
    bitmaps = b''.join(
        parse_char(char)
        for char in characters
    )
    video_chars = image['video-chars']
    video = bytes(
        c
        for r in range(top, top + height)
        for c in video_chars[r * columns + left:r * columns + left + width]
    )
    video_colors = image['video-colors']
    colors = bytes(
        c
        for r in range(top, top + height)
        for c in video_colors[r * columns + left:r * columns + left + width]
    )

    assert len(video) == width * height
    return (video, colors, bitmaps)


def main():
    import argparse

    parser = argparse.ArgumentParser(
        description='Convert Pixel Pen file to Vic-20 format.'
    )
    parser.add_argument('--left', type=int, help='Include source image starting at this character column')
    parser.add_argument('--top', type=int, help='Include source image starting at this character row')
    parser.add_argument('--width', type=int, help='Number of character columns to include')
    parser.add_argument('--height', type=int, help='Number of character rows to include')
    parser.add_argument(
        'input', type=argparse.FileType('rb'),
        help='Pixel Pen file to convert'
    )
    parser.add_argument(
        'output',
        help='Binary file to write'
    )

    args = parser.parse_args()

    document = json.load(args.input)
    (video, colors, bitmaps) = convert(document, args.left, args.top, args.width, args.height)
    with open(args.output, 'wb') as out:
        out.write(video)
        out.write(colors)
        out.write(bitmaps)


if __name__ == '__main__':
    main()
