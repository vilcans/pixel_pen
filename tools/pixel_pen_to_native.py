#!/usr/bin/env python3

"""For converting a Pixel Pen file to native format.

Writes a file with the video array (character numbers) followed by the character bitmaps.
"""

import sys
from argparse import ArgumentError
from array import array
import json
import codecs
from dataclasses import dataclass
from typing import List

@dataclass
class Colors:
    background: int
    border: int
    aux: int

@dataclass
class Image:
    colors: Colors
    width: int
    height: int
    char_height: int
    num_chars: int
    video: List[int]
    video_colors: List[int]
    bitmaps: List[int]


def parse_char(char_hex):
    return codecs.decode(char_hex, 'hex')


def convert(document, left, top, width, height) -> Image:
    """Convert the given document to native bytes.

    Returns the Image object.
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
    bitmaps = b''.join(
        parse_char(char)
        for char in characters
    )
    video_chars = image['video-chars']
    video = bytes(
        c & 0xff
        for r in range(top, top + height)
        for c in video_chars[r * columns + left:r * columns + left + width]
    )
    video_colors = bytes(
        c
        for r in range(top, top + height)
        for c in image['video-colors'][r * columns + left:r * columns + left + width]
    )

    assert len(video) == width * height
    return Image(
        colors=Colors(
            background=image['colors'][0],
            border=image['colors'][1],
            aux=image['colors'][2],
        ),
        width=width,
        height=height,
        num_chars=len(characters),
        char_height=8,
        video=video,
        video_colors=video_colors,
        bitmaps=bitmaps
    )


def get_order(width, height, *, column_major, reverse_columns, reverse_rows):
    h_range = list(range(width))
    if reverse_columns:
        h_range.reverse()

    v_range = list(range(height))
    if reverse_rows:
        v_range.reverse()

    if column_major:
        return [
            (col, row)
            for col in h_range
            for row in v_range
       ]
    else:
        return [
            (col, row)
            for row in v_range
            for col in h_range
       ]

def get_bitmaps(image, _char_numbers):
    return image.bitmaps

def get_bitmaps_linearly(image, char_numbers):
    bitmap = b''
    for c in char_numbers:
        offset = c * image.char_height
        bitmap += image.bitmaps[offset:offset + image.char_height]
    return bitmap

def get_bitmaps_by_pixel_rows(image, char_numbers):
    bitmap = b''
    for y in range(image.char_height * image.height):
        row = y // 8
        cy = y % 8
        for col in range(image.width):
            c = char_numbers[image.width * row + col]
            offset = c * image.char_height + cy
            bitmap += bytes([image.bitmaps[offset]])
    return bitmap

def sections(value):
    value = value.upper()
    for c in value:
        if not c in 'VCB':
            raise ArgumentError(f'Invalid section character: {c}')
    return value

bitmap_orders = {
    'characters': get_bitmaps,
    'linear': get_bitmaps_linearly,
    'pixel-rows': get_bitmaps_by_pixel_rows,
}

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
        '--column-major', action='store_true',
        help='Save cells in column major order'
        )
    parser.add_argument(
        '--reverse-columns', action='store_true',
        help='Store columns from right to left'
        )
    parser.add_argument(
        '--reverse-rows', action='store_true',
        help='Store rows from bottom to top'
        )
    parser.add_argument('--sections', type=sections, default='VCB',
        help='Which sections to include in the binary output and in which order. ' +
        'This is a string consisting of the letters V, C, and/or B. ' +
        'They correspond to: V = video array, C = color table, B = character bitmaps'
    )
    parser.add_argument(
        '--bitmap-order', default='characters',
        choices=bitmap_orders.keys(),
        help='Which order to write the bitmaps from the characters: '
        'characters = the character bitmaps in the order they appear in the character set (the order the hardware uses; the default), '
        'linear = the character bitmaps in the order they appear in the video array, one character cell at a time, '
        'pixel-rows = the character bitmaps in the order they appear in the video array, one pixel row at a time.'
    )
    parser.add_argument(
        '--invert', action='store_true',
        help='Invert the character bitmaps'
    )
    parser.add_argument('--meta', help='Write metadata to this file')
    parser.add_argument('--meta-prefix', default='', help='When writing metadata file, add this prefix to the beginning of each symbol')
    parser.add_argument(
        'input', type=argparse.FileType('rb'),
        help='Pixel Pen file to convert'
    )
    parser.add_argument(
        'output',
        help='Binary file to write.'
    )

    args = parser.parse_args()

    document = json.load(args.input)
    image = convert(document, args.left, args.top, args.width, args.height)
    if args.invert:
        image.bitmaps = bytes(x ^ 0xff for x in image.bitmaps)

    if 'V' in args.sections and image.num_chars > 256:
        print(f'Too many characters:', image.num_chars, file=sys.stderr)
        sys.exit(1)

    video_offset = None
    colors_offset = None
    bitmaps_offset = None

    cell_order = get_order(
        image.width, image.height,
        column_major=args.column_major,
        reverse_columns=args.reverse_columns,
        reverse_rows=args.reverse_rows
    )
    character_numbers = [
        image.video[col + row * image.width]
        for (col, row) in cell_order
    ]
    colors = [
        image.video_colors[col + row * image.width]
        for (col, row) in cell_order
    ]

    with open(args.output, 'wb') as out:
        for section in args.sections:
            if section == 'V':
                video_offset = out.tell()
                out.write(array('B', character_numbers))
            elif section == 'C':
                colors_offset = out.tell()
                out.write(array('B', colors))
            elif section == 'B':
                bitmaps_offset = out.tell()
                out.write(bitmap_orders[args.bitmap_order](image, character_numbers))
    if args.meta:
        lines = [
            '; Width in character cells',
            f'{args.meta_prefix}width = {image.width}',
            '; Height in character cells',
            f'{args.meta_prefix}height = {image.height}',
            '; If image was cropped during conversion, what column in the original image was the leftmost one',
            f'{args.meta_prefix}left = {args.left or 0}',
            '; If image was cropped during conversion, what row in the original image was the topmost one',
            f'{args.meta_prefix}top = {args.top or 0}',
            '; Total number of cells (width * height)',
            f'{args.meta_prefix}video_size = {image.width * image.height}',
            '; Background color',
            f'{args.meta_prefix}background = {image.colors.background}',
            '; Border color',
            f'{args.meta_prefix}border = {image.colors.border}',
            '; Aux color',
            f'{args.meta_prefix}aux = {image.colors.aux}',
        ]
        if args.bitmap_order == 'linear':
            lines += [
                '; Size of the character bitmaps (number of character cells * bytes per character)',
                f'{args.meta_prefix}bitmaps_size = {image.width * image.height * image.char_height}',
            ]
        else:
            lines += [
                '; Number of characters (bitmaps) used',
                f'{args.meta_prefix}num_chars = {image.num_chars}',
                '; Size of the character bitmaps (number of character * bytes per character)',
                f'{args.meta_prefix}bitmaps_size = {image.num_chars * image.char_height}',
            ]

        if video_offset is not None:
            lines += [
                '; Offset into file where video array starts',
                f'{args.meta_prefix}video_offset = {video_offset}',
            ]
        if colors_offset is not None:
            lines += [
                '; Offset into file where color array starts',
                f'{args.meta_prefix}colors_offset = {colors_offset}',
            ]
        if bitmaps_offset is not None:
            lines += [
                '; Offset into file where character bitmaps start',
                f'{args.meta_prefix}bitmaps_offset = {bitmaps_offset}',
            ]
        with open(args.meta, 'w') as out:
            for line in lines:
                out.write(line)
                out.write('\n')


if __name__ == '__main__':
    main()
