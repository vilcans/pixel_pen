//! Support for fluff64 file format.
//! Reverse engineered from Turbo Rascal's example files and source code.

use bincode::Options;
use imgref::ImgVec;
use serde::{de::DeserializeOwned, Deserialize};
use std::{
    fs::File,
    io::{self, BufReader, Read},
    path::Path,
};

use crate::{
    error::Error,
    image_io,
    vic::{self, GlobalColors, VicImage},
};

#[derive(Deserialize, Copy, Clone, Debug)]
#[repr(packed(1))]
struct FluffHeader {
    /// Magic string "FLUFF64"
    pub identifier: [u8; 7],

    /// Version number. 2 on the files I have tested.
    pub version: u32,

    /// Image type.
    ///
    /// 0: QImageBitmap
    /// 1: MultiColorBitmap
    /// 2: HiresBitmap
    /// 3: LevelEditor
    /// 4: CharMapMulticolor
    /// 5: Sprites
    /// 6: CharmapRegular
    /// 7: FullScreenChar
    /// 8: CharMapMultiColorFixed
    /// 9: VIC20_MultiColorbitmap
    /// 10: Sprites2
    /// 11: CGA
    /// 12: AMIGA320x200
    /// 13: AMIGA320x256
    /// 14: OK64_256x256
    /// 15: X16_640x480
    /// 16: NES
    /// 17: LMetaChunk
    /// 18: LevelEditorNES
    /// 19: SpritesNES
    /// 20: GAMEBOY
    /// 21: LevelEditorGameboy
    /// 22: ATARI320x200
    /// 23: HybridCharset
    /// 24: AmstradCPC
    /// 25: AmstradCPCGeneric
    /// 26: BBC
    pub image_type: u8,

    /// Palette.
    ///
    /// 0: C64
    /// 1: C64_ORG
    /// 2: CGA1_LOW
    /// 3: CGA1_HIGH
    /// 4: CGA2_LOW
    /// 5: CGA2_HIGH
    /// 6: VIC20
    /// 7: PICO8
    /// 8: OK64
    /// 9: X16
    /// 10: NES
    /// 11: AMSTRADCPC
    /// 12: BBC
    pub palette_type: u8,

    /// Background color
    pub background: u8,
    /// A copy of background.
    /// (From `charsetimage.cpp: CharsetImage::SavePensBin`)
    pub _background2: u8,
    /// Border color
    pub border: u8,
    /// Auxiliary color
    pub aux: u8,
    /// Unknown. Is set to 5 on the images I've checked.
    pub _pen3: u8,
    /// Picture width in characters.
    pub width_chars: u8,
    /// Picture height in characters.
    pub height_chars: u8,
}

#[derive(Deserialize, Debug)]
#[repr(packed(1))]
struct FluffChar {
    bits: [u8; 8],
    _background: u8,
    _border: u8,
    _aux: u8,
    color: u8,
}

pub fn load_fluff64(reader: &mut impl Read) -> Result<VicImage, Error> {
    let header: FluffHeader = io::read_struct(reader)?;
    println!("Fluff header: {:?}", header);
    let width = header.width_chars as usize;
    let height = header.height_chars as usize;
    let video_buffer = (0..width * height)
        .map(|index| -> Result<vic::Char, Error> {
            let flf_char: FluffChar = read_struct(reader)?;
            //println!("Fluff char: {:?}", flf_char);
            print!("{:2x}", flf_char.color);
            if index % width == width - 1 {
                println!();
            }

            let mut bits = [0; 8];
            for i in 0..vic::Char::HEIGHT {
                // Fluff stores multicolor pixels in reverse order
                let c = flf_char.bits[i];
                bits[i] = (((c >> 6) & 3) << 0)
                    | (((c >> 4) & 3) << 2)
                    | (((c >> 2) & 3) << 4)
                    | (((c >> 0) & 3) << 6);
            }
            Ok(vic::Char {
                bits,
                color: if (0..7).contains(&flf_char.color) {
                    flf_char.color
                } else {
                    // Color may be 255 for characters with no color.
                    1
                },
            })
        })
        .collect::<Result<Vec<vic::Char>, Error>>()?;
    let mut image = VicImage::with_content(ImgVec::new(video_buffer, width, height));
    image.colors = GlobalColors([header.background, header.border, header.aux]);
    println!("Colors: {:?}", image.colors);
    Ok(image)
}
