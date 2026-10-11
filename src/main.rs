use fontdue::{Font, FontSettings};
use kj_assoc::App;
use rakugaki::{RenderMode, render_char_with_mode};

fn main() -> anyhow::Result<()> {
    env_logger::init();

    ratatui::run(|terminal| App::default().run(terminal))

    // let rendered_kanji_lines = render_char_with_mode(
    //     "res/NotoSansJP-VariableFont_wght.ttf",
    //     '蘇',
    //     40,
    //     20,
    //     RenderMode::ANSI_TRUECOLOR,
    // )
    // .unwrap()
    // .join("\n");
    // print!("{}", rendered_kanji_lines);
    //

    // let mut rows = vec![String::new(); H / 4];
    // for ch in "地下鉄道".chars() {
    //     for (row, line) in rows.iter_mut().zip(to_braille(&glyph_dots(&font, ch))) {
    //         row.push_str(&line);
    //         row.push_str("  "); // gap between glyphs
    //     }
    // }
    // println!("{}", rows.join("\n"));
    // Ok(())
}
