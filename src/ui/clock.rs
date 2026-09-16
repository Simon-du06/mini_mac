use anyhow::{anyhow, Result};
use embedded_graphics::{
    mono_font::MonoTextStyle,
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};

use super::display::{Display, CENTER_MIDDLE_TEXT_STYLE};

pub fn draw_clock(
    display: &mut Display,
    h: u8,
    m: u8,
    s: u8,
    style: MonoTextStyle<BinaryColor>,
) -> Result<()> {
    display
        .clear(BinaryColor::Off)
        .map_err(|err| anyhow!("Failed to clear display: {err:?}"))?;
    Text::with_text_style(
        &format!("{h:02}:{m:02}:{s:02}"),
        Point::new(64, 32),
        style,
        CENTER_MIDDLE_TEXT_STYLE,
    )
    .draw(display)
    .map_err(|err| anyhow!("Failed to draw clock: {err:?}"))?;
    display
        .flush()
        .map_err(|err| anyhow!("Failed to flush display buffer: {err:?}"))?;

    Ok(())
}
