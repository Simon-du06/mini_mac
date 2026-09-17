use anyhow::{Result, anyhow};
use embedded_graphics::{
    image::Image,
    mono_font::MonoTextStyle,
    pixelcolor::{BinaryColor, Rgb565},
    prelude::*,
    text::{Alignment, Baseline, Text, TextStyle, TextStyleBuilder},
};
use esp_idf_svc::hal::{
    gpio::{Gpio6, Gpio7},
    i2c::{I2C0, I2cDriver, config::Config as I2cConfig},
    prelude::Hertz,
};
use ssd1306::{
    I2CDisplayInterface, Ssd1306,
    mode::{BufferedGraphicsMode, DisplayConfig},
    prelude::{DisplayRotation, DisplaySize128x64, I2CInterface},
};
use tinybmp::Bmp;

pub type Display = Ssd1306<
    I2CInterface<I2cDriver<'static>>,
    DisplaySize128x64,
    BufferedGraphicsMode<DisplaySize128x64>,
>;

pub const CENTER_MIDDLE_TEXT_STYLE: TextStyle = TextStyleBuilder::new()
    .alignment(Alignment::Center)
    .baseline(Baseline::Middle)
    .build();

pub const CENTER_RIGHT_TEXT_STYLE: TextStyle = TextStyleBuilder::new()
    .alignment(Alignment::Right)
    .baseline(Baseline::Middle)
    .build();

pub fn init_display(i2c0: I2C0, sda: Gpio6, scl: Gpio7) -> Result<Display> {
    let i2c = I2cDriver::new(i2c0, sda, scl, &I2cConfig::new().baudrate(Hertz(400_000)))?;

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display
        .init()
        .map_err(|err| anyhow!("Failed to initialize display: {err:?}"))?;

    Ok(display)
}

pub fn show_boot_image(display: &mut Display) -> Result<()> {
    let bmp = Bmp::from_slice(include_bytes!("../asset/images/hello2.bmp"))
        .map_err(|err| anyhow!("Failed to parse boot bitmap: {err:?}"))?;
    let image: Image<Bmp<Rgb565>> = Image::new(&bmp, Point::new(0, 0));

    image
        .draw(&mut display.color_converted())
        .map_err(|err| anyhow!("Failed to draw boot bitmap: {err:?}"))?;
    display
        .flush()
        .map_err(|err| anyhow!("Failed to flush display buffer: {err:?}"))?;

    Ok(())
}

pub fn draw_no_data(
    display: &mut Display,
    label: &str,
    style: MonoTextStyle<BinaryColor>,
) -> Result<()> {
    display
        .clear(BinaryColor::Off)
        .map_err(|err| anyhow!("Failed to clear display: {err:?}"))?;
    Text::with_text_style(label, Point::new(64, 20), style, CENTER_MIDDLE_TEXT_STYLE)
        .draw(display)
        .map_err(|err| anyhow!("Failed to draw no data message: {err:?}"))?;
    Text::with_text_style(
        "NO DATA",
        Point::new(64, 44),
        style,
        CENTER_MIDDLE_TEXT_STYLE,
    )
    .draw(display)
    .map_err(|err| anyhow!("Failed to draw no data message: {err:?}"))?;
    display
        .flush()
        .map_err(|err| anyhow!("Failed to flush display buffer: {err:?}"))?;

    Ok(())
}
