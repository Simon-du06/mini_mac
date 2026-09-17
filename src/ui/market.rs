use anyhow::{Result, anyhow};
use embedded_graphics::{
    mono_font::MonoTextStyle,
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Polyline, PrimitiveStyle},
    text::Text,
};

use super::display::{CENTER_MIDDLE_TEXT_STYLE, Display, draw_no_data};

pub fn draw_market(
    display: &mut Display,
    symbol: &str,
    history: &[f32],
    style: MonoTextStyle<BinaryColor>,
) -> Result<()> {
    if history.is_empty() {
        return draw_no_data(display, symbol, style);
    }

    display
        .clear(BinaryColor::Off)
        .map_err(|err| anyhow!("Failed to clear display: {err:?}"))?;

    let current_price = history.last().copied().unwrap_or_default();
    Text::with_text_style(
        &format!("{symbol} ${current_price:.2}"),
        Point::new(64, 12),
        style,
        CENTER_MIDDLE_TEXT_STYLE,
    )
    .draw(display)
    .map_err(|err| anyhow!("Failed to draw price: {err:?}"))?;

    if history.len() >= 2 {
        let min = history.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = history.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let range = (max - min).max(1.0);

        const GRAPH_LEFT: i32 = 4;
        const GRAPH_RIGHT: i32 = 124;
        const GRAPH_TOP: i32 = 26;
        const GRAPH_HEIGHT: i32 = 32;

        let span = (GRAPH_RIGHT - GRAPH_LEFT) as f32 / (history.len() - 1) as f32;
        let points: Vec<Point> = history
            .iter()
            .enumerate()
            .map(|(i, &price)| {
                let x = GRAPH_LEFT + (i as f32 * span) as i32;
                let y =
                    GRAPH_TOP + GRAPH_HEIGHT - ((price - min) / range * GRAPH_HEIGHT as f32) as i32;
                Point::new(x, y)
            })
            .collect();

        Polyline::new(&points)
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(display)
            .map_err(|err| anyhow!("Failed to draw price trend: {err:?}"))?;
    }

    display
        .flush()
        .map_err(|err| anyhow!("Failed to flush display buffer: {err:?}"))?;
    Ok(())
}
