//! Support for fluff64 file format.
//! Reverse engineered from Turbo Rascal's example files and source code.

use imgref::ImgVec;
use serde::Deserialize;
use std::io::Read;

use crate::{
    error::Error,
    image_io,
    vic::{self, GlobalColors, VicImage},
};

#[derive(Deserialize, Copy, Clone, Debug)]
#[repr(packed(1))]
struct FluffHeader {
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
    /// Bitmap bits. In reverse order compared to memory layout, and with aux=0b10 and color=0b11,
    // compared to hardware's aux=0b11 and color=0b10.
    bits: [u8; 8],
    _background: u8,
    _border: u8,
    _aux: u8,
    color: u8,
}

pub fn load_fluff64(reader: &mut impl Read) -> Result<VicImage, Error> {
    let mut identifier = [0u8; 7];
    reader
        .read_exact(&mut identifier)
        .map_err(|err| match err.kind() {
            std::io::ErrorKind::UnexpectedEof => Error::TruncatedData,
            _ => Error::ReadFailure(err),
        })?;
    if &identifier != b"FLUFF64" {
        return Err(Error::WrongMagic);
    }

    let header: FluffHeader = image_io::read_struct(reader)?;

    let width = header.width_chars as usize;
    let height = header.height_chars as usize;
    if width == 0 || height == 0 {
        return Err(Error::InvalidSize(width, height));
    }
    let video_buffer = (0..width * height)
        .map(|_| -> Result<vic::Char, Error> {
            let flf_char: FluffChar = image_io::read_struct(reader)?;
            let mut bits = [0; 8];
            for (flf_bits, result_bits) in flf_char.bits.iter().zip(bits.iter_mut()) {
                // Fluff stores multicolor pixels in reverse order.
                // Swap aux and color and reverse the pixels.
                let fixed = (0..8)
                    .step_by(2)
                    .map(|bit|
                        match (flf_bits >> (6 - bit)) & 0b11 {
                                0b10 => 0b11,
                                0b11 => 0b10,
                                a => a,
                            } << bit
                    )
                    .sum();
                *result_bits = fixed;
            }
            Ok(vic::Char::new(
                bits,
                if vic::ALLOWED_CHAR_COLORS.contains(&flf_char.color) {
                    flf_char.color
                } else {
                    // Color may be 255 for characters with no color.
                    1
                },
            ))
        })
        .collect::<Result<Vec<vic::Char>, Error>>()?;
    let mut image = VicImage::with_content(ImgVec::new(video_buffer, width, height));
    image.colors = GlobalColors([header.background, header.border, header.aux]);
    Ok(image)
}
