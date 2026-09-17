use anyhow::{Result, anyhow};
use embedded_graphics::{
    image::{Image, ImageRaw},
    mono_font::MonoTextStyle,
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};

use crate::{
    network::geo::GeoInfo,
    weather::{
        icons,
        sync_weather::{CurrentWeather, get_weather_icon},
    },
};

use super::display::{CENTER_RIGHT_TEXT_STYLE, Display};

pub fn draw_weather(
    display: &mut Display,
    weather: &CurrentWeather,
    geo: &GeoInfo,
    style: MonoTextStyle<BinaryColor>,
) -> Result<()> {
    let raw: ImageRaw<BinaryColor> = ImageRaw::new(
        get_weather_icon(weather.weathercode),
        icons::ICON_LARGE_WIDTH,
    );
    let image = Image::new(&raw, Point::new(5, 5));

    display
        .clear(BinaryColor::Off)
        .map_err(|err| anyhow!("Failed to clear display: {err:?}"))?;
    image
        .draw(&mut display.color_converted())
        .map_err(|err| anyhow!("Failed to draw icon bitmap: {err:?}"))?;
    Text::with_text_style(
        geo.city.as_str(),
        Point::new(128, 54),
        style,
        CENTER_RIGHT_TEXT_STYLE,
    )
    .draw(display)
    .map_err(|err| anyhow!("Failed to draw city: {err:?}"))?;
    Text::with_text_style(
        &format!("{:.1}°C", weather.temperature_2m),
        Point::new(128, 32),
        style,
        CENTER_RIGHT_TEXT_STYLE,
    )
    .draw(display)
    .map_err(|err| anyhow!("Failed to draw temperature: {err:?}"))?;
    display
        .flush()
        .map_err(|err| anyhow!("Failed to flush display buffer: {err:?}"))?;
    Ok(())
}
