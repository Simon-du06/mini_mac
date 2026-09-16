use anyhow::{anyhow, Result};
use std::time::{SystemTime, UNIX_EPOCH};
use embedded_graphics::{
    mono_font::MonoTextStyle,
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Polyline, PrimitiveStyle},
    text::Text,
};

use super::display::{Display, CENTER_MIDDLE_TEXT_STYLE};

pub fn draw_glucose(
    display: &mut Display,
    history: &[GlucoseDatas],
    style: MonoTextStyle<BinaryColor>
) -> Result<()> {
    display
        .clear(BinaryColor::Off)
        .map_err(|err| anyhow!("Failed to clear display: {err:?}"))?;

    let now = SystemTime::now();
    let duration = now.duration_since(UNIX_EPOCH)?;
    let now_ms = duration.as_millis();

    let current_sugar = history.first();
    let message = match current_sugar {
        None => "NO DATA".to_string(),
        Some(sugar) if sugar.age_minutes(now_ms) >= 10 => {
            format!("{} OLD", sugar.sgv)
        }
        Some(sugar) => format!("{} mg/dL", sugar.sgv),
    };

    Text::with_text_style(
        &message,
        Point::new(64, 12),
        style, CENTER_MIDDLE_TEXT_STYLE,
    )
    .draw(display)
    .map_err(|err| anyhow!("Failed to draw glucose: {err:?}"))?;

    let glucose_values: Vec<f32> = history
        .iter()
        .rev()
        .map(|x| x.sgv as f32)
        .collect();

    if glucose_values.len() > 3 {
        let min = glucose_values.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = glucose_values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let range = (max - min).max(1.0);

        const GRAPH_LEFT: i32 = 4;
        const GRAPH_RIGHT: i32 = 124;
        const GRAPH_TOP: i32 = 26;
        const GRAPH_HEIGHT: i32 = 32;

        let span = (GRAPH_RIGHT - GRAPH_LEFT) as f32 / (history.len() - 1) as f32;
        let points: Vec<Point> = glucose_values
            .iter()
            .enumerate()
            .map(|(i, &glucose)| {
                let x = GRAPH_LEFT + (i as f32 * span) as i32;
                let y = GRAPH_TOP + GRAPH_HEIGHT - ((glucose - min) / range * GRAPH_HEIGHT as f32) as i32;
                Point::new(x, y)
            })
            .collect();

        Polyline::new(&points)
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(display)
            .map_err(|err| anyhow!("Failed to draw glucose trend: {err:?}"))?;
    }

    display
        .flush()
        .map_err(|err| anyhow!("Failed to flush display buffer: {err:?}"))?;
    Ok(())
}