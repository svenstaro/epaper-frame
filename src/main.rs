use std::io::Cursor;
use std::time::Duration;

use anyhow::Result;
use esp_idf_hal::delay::Ets;
use esp_idf_hal::gpio::{self, PinDriver};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;
use esp_idf_hal::spi::{SpiConfig, SpiDeviceDriver, SpiDriverConfig, SpiError};
use esp_idf_sys::{self as _, EspError};
use image::imageops::{dither, ColorMap};
use image::Rgb;
// If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use log::*;
use thiserror::Error;

#[derive(Debug, Error)]
enum Error {
    #[error("SPI")]
    Spi(#[from] SpiError),
    #[error("ESP")]
    Esp(#[from] EspError),
}

struct InkyColors;
impl ColorMap for InkyColors {
    type Color = Rgb<u8>;

    fn index_of(&self, color: &Self::Color) -> usize {
        let palette = uc8159::Palette::new(1.0);
        let color_ = palette.closest_color(color[0], color[1], color[2]);
        uc8159::Color::all_significant()
            .iter()
            .position(|&c| c == color_)
            .unwrap()
    }

    fn map_color(&self, color: &mut Self::Color) {
        let palette = uc8159::Palette::new(1.0);
        let color_ = palette.closest_color(color[0], color[1], color[2]);
        *color = match color_ {
            uc8159::Color::Black => [57, 48, 57].into(),
            uc8159::Color::White => [255, 255, 255].into(),
            uc8159::Color::Green => [58, 91, 70].into(),
            uc8159::Color::Blue => [61, 59, 94].into(),
            uc8159::Color::Red => [156, 72, 75].into(),
            uc8159::Color::Yellow => [208, 190, 71].into(),
            uc8159::Color::Orange => [77, 106, 73].into(),
            uc8159::Color::Clean => [255, 255, 255].into(),
        }
    }
}

fn main() -> Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Hello");
    let peripherals = Peripherals::take().unwrap();
    let spi = peripherals.spi2;
    let delay = Ets;

    let reset = PinDriver::output(peripherals.pins.gpio4)?;
    let dc = PinDriver::output(peripherals.pins.gpio5)?;
    let busy = PinDriver::input(peripherals.pins.gpio8)?;
    let sclk = peripherals.pins.gpio6;
    let sdo = peripherals.pins.gpio7;
    let cs = peripherals.pins.gpio10;

    info!("Before init SPI");
    let device = SpiDeviceDriver::new_single(
        spi,
        sclk,
        sdo,
        Option::<gpio::AnyIOPin>::None,
        Some(cs),
        &SpiDriverConfig::new(),
        &SpiConfig::new().baudrate(3.MHz().into()),
    )?;
    info!("After init SPI");

    let builder = std::thread::Builder::new().stack_size(170_000);
    builder
        .spawn(move || {
            info!("Before init display");
            let mut display = uc8159::Display::<_, _, _, _, _, Error>::new(
                device,
                delay,
                reset,
                busy,
                dc,
                uc8159::Config {
                    border_color: uc8159::Color::White,
                },
            );
            info!("After init display");

            display.fill(uc8159::Color::White);

            info!("Loading image");
            let image_data = include_bytes!("../test.png");
            let mut image = image::io::Reader::new(Cursor::new(image_data))
                .with_guessed_format()
                .unwrap()
                .decode()
                .unwrap()
                .resize(
                    display.width() as u32,
                    display.height() as u32,
                    image::imageops::FilterType::Nearest,
                )
                .to_rgb8();
            let color_map = InkyColors {};
            dither(&mut image, &color_map);
            info!("Loaded image successfully");

            let palette = uc8159::Palette::new(1.0);

            info!("Displaying image");

            let padding_x = 0.max((display.width() - image.width() as usize) / 2);
            let padding_y = 0.max((display.height() - image.height() as usize) / 2);
            for (x, y, &image::Rgb([r, g, b])) in image.enumerate_pixels() {
                display.set_pixel(
                    padding_x + x as usize,
                    padding_y + y as usize,
                    palette.closest_color(r, g, b),
                );
            }
            display.show().unwrap();
        })
        .unwrap();
    Ok(())
}
