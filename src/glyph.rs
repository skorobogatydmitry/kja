//! drawing kanjis on TTY

use std::sync::LazyLock;

use fontdue::{Font, FontSettings};

pub const MIN_WIDTH_SYMBOLS: usize = 14;
pub const MIN_HEIGHT_SYMBOLS: usize = 5;

// width / height
const ASPECT_RATIO: f32 = 3.0;

const SYMBOL_WIDTH_DOTS: usize = 2;
const SYMBOL_HEIGHT_DOTS: usize = 4;

const BRAILLE_START_OFFSET: u32 = 0x2800;

// less readable in 25x25, 40x40 is min to make out compex kanjis
pub static FONT_BYTES: &[u8] = include_bytes!("../res/KleeOne-Regular.ttf");
/// this font is simple and could be made out from 25x25
// static FONT_BYTES: &[u8] = include_bytes!("../res/NotoSansJP-VariableFont_wght.ttf");
static JP_FONT: LazyLock<Font> = LazyLock::new(|| {
    Font::from_bytes(FONT_BYTES, FontSettings::default()).expect("cannot load font")
});

/// size of the block which the glyph must fit
#[derive(Debug)]
pub struct RenderSizeSymbols {
    height: u16,
    width: u16,
}

impl RenderSizeSymbols {
    pub fn new(w: u16, h: u16) -> Self {
        Self {
            height: h,
            width: w,
        }
    }

    fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    fn width_px(&self) -> usize {
        self.width as usize * SYMBOL_WIDTH_DOTS as usize
    }

    fn height_px(&self) -> usize {
        self.height as usize * SYMBOL_HEIGHT_DOTS as usize
    }
}

pub(crate) fn render(kanji: char, box_size: RenderSizeSymbols) -> Vec<String> {
    let box_ratio = box_size.aspect_ratio();
    let size = if box_ratio > ASPECT_RATIO {
        RenderSizeSymbols::new(
            (box_size.height as f32 * ASPECT_RATIO).ceil() as u16,
            box_size.height,
        )
    } else {
        RenderSizeSymbols::new(
            box_size.width,
            (box_size.width as f32 / ASPECT_RATIO).ceil() as u16,
        )
    };
    let dots = glyph_dots(kanji, &size);
    to_braille(dots, size)
}

fn glyph_dots(ch: char, size: &RenderSizeSymbols) -> Vec<Vec<bool>> {
    let (width_px, height_px) = (size.width_px(), size.height_px());
    let (m, bmp) = JP_FONT.rasterize(ch, width_px.min(height_px) as f32);
    let mut dots = vec![vec![false; width_px]; height_px];
    let ox = width_px.saturating_sub(m.width) / 2;
    let oy = height_px.saturating_sub(m.height) / 2;
    for y in 0..m.height.min(height_px) {
        for x in 0..m.width.min(width_px) {
            if bmp[y * m.width + x] > 70 {
                dots[oy + y][ox + x] = true;
            }
        }
    }
    dots
}

/// convert dots matrix to list of braille symbol rows
fn to_braille(dots: Vec<Vec<bool>>, size: RenderSizeSymbols) -> Vec<String> {
    const BITS: [[u32; SYMBOL_WIDTH_DOTS]; SYMBOL_HEIGHT_DOTS] =
        [[0x01, 0x08], [0x02, 0x10], [0x04, 0x20], [0x40, 0x80]];
    (0..size.height as usize)
        .map(|cy| {
            (0..size.width as usize)
                .map(|cx| {
                    let mut b = 0;
                    for (dy, row) in BITS.iter().enumerate() {
                        for (dx, bit) in row.iter().enumerate() {
                            let y_px = cy * SYMBOL_HEIGHT_DOTS + dy;
                            let x_px = cx * SYMBOL_WIDTH_DOTS + dx;
                            if dots[y_px][x_px] {
                                b |= *bit;
                            }
                        }
                    }
                    char::from_u32(BRAILLE_START_OFFSET + b).unwrap()
                })
                .collect()
        })
        .collect()
}
