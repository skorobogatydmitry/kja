//! drawing kanjis on TTY

use std::sync::LazyLock;

use fontdue::{Font, FontSettings};

pub static WIDTH_PX: usize = WIDTH_SYMBOLS * SYMBOL_WIDTH_DOTS;
pub static HEIGHT_PX: usize = HEIGHT_SYMBOLS * SYMBOL_HEIGHT_DOTS;

pub static WIDTH_SYMBOLS: usize = 14;
pub static HEIGHT_SYMBOLS: usize = 5;

pub static SYMBOL_WIDTH_DOTS: usize = 2;
pub static SYMBOL_HEIGHT_DOTS: usize = 4;

pub static BRAILLE_START_OFFSET: u32 = 0x2800;

// less readable in 25x25, 40x40 is min to make out compex kanjis
pub static FONT_BYTES: &[u8] = include_bytes!("../res/KleeOne-Regular.ttf");
/// this font is simple and could be made out from 25x25
// static FONT_BYTES: &[u8] = include_bytes!("../res/NotoSansJP-VariableFont_wght.ttf");
static JP_FONT: LazyLock<Font> = LazyLock::new(|| {
    Font::from_bytes(FONT_BYTES, FontSettings::default()).expect("cannot load font")
});

pub(crate) fn render(kanji: char) -> Vec<String> {
    let dots = glyph_dots(kanji);
    to_braille(dots)
}

fn glyph_dots(ch: char) -> [[bool; WIDTH_PX]; HEIGHT_PX] {
    let (m, bmp) = JP_FONT.rasterize(ch, std::cmp::min(WIDTH_PX, HEIGHT_PX) as f32);
    let mut dots = [[false; WIDTH_PX]; HEIGHT_PX];
    let ox = WIDTH_PX.saturating_sub(m.width) / 2;
    let oy = HEIGHT_PX.saturating_sub(m.height) / 2;
    for y in 0..m.height.min(HEIGHT_PX) {
        for x in 0..m.width.min(WIDTH_PX) {
            if bmp[y * m.width + x] > 70 {
                dots[oy + y][ox + x] = true;
            }
        }
    }
    dots
}

/// convert dots matrix to list of braille symbol rows
fn to_braille(dots: [[bool; WIDTH_PX]; HEIGHT_PX]) -> Vec<String> {
    const BITS: [[u32; SYMBOL_WIDTH_DOTS]; SYMBOL_HEIGHT_DOTS] =
        [[0x01, 0x08], [0x02, 0x10], [0x04, 0x20], [0x40, 0x80]];
    (0..HEIGHT_SYMBOLS)
        .map(|cy| {
            (0..WIDTH_SYMBOLS)
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
