#!/usr/bin/env python3

"""For converting a Pixel Pen file to native format.

Writes a file with the video array (character numbers) followed by the character bitmaps.
"""

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


def main():
    import argparse

    parser = argparse.ArgumentParser(
        description='Convert Pixel Pen file to Vic-20 format.'
    )
    parser.add_argument('--left', type=int, help='Include source image starting at this character column')
    parser.add_argument('--top', type=int, help='Include source image starting at this character row')
    parser.add_argument('--width', type=int, help='Number of character columns to include')
    parser.add_argument('--height', type=int, help='Number of character rows to include')
    parser.add_argument('--meta', help='Write metadata to this file')
    parser.add_argument('--meta-prefix', default='', help='When writing metadata file, add this prefix to the beginning of each symbol')
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
    image = convert(document, args.left, args.top, args.width, args.height)
    with open(args.output, 'wb') as out:
        out.write(image.video)
        out.write(image.video_colors)
        out.write(image.bitmaps)
    if args.meta:
        video_size = image.width * image.height
        with open(args.meta, 'w') as out:
            for line in [
                f'{args.meta_prefix}width = {image.width}',
                f'{args.meta_prefix}height = {image.height}',
                f'{args.meta_prefix}background = {image.colors.background}',
                f'{args.meta_prefix}border = {image.colors.border}',
                f'{args.meta_prefix}aux = {image.colors.aux}',
                f'{args.meta_prefix}num_chars = {image.num_chars}',
                f'{args.meta_prefix}video_offset = 0',
                f'{args.meta_prefix}colors_offset = {video_size}',
                f'{args.meta_prefix}video_size = {video_size}',
                f'{args.meta_prefix}bitmaps_offset = {video_size * 2}',
                f'{args.meta_prefix}bitmaps_size = {image.num_chars * image.char_height}',
            ]:
                out.write(line)
                out.write('\n')


if __name__ == '__main__':
    main()
