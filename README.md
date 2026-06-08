# mini_mac

MiniMac is a Wi-Fi connected embedded Rust project running on an ESP32-C6 Super Mini.
It recreates a minimalist Macintosh-inspired desk companion on an SSD1306 OLED display: a self-updating clock and live weather station, with more screens (touch navigation, market quotes) on the way.

## Features
- Boot screen with custom bitmap logo
- Wi-Fi connectivity with IP-based geolocation (city, timezone offset)
- NTP time sync and local clock display, adjusted to the detected timezone
- Live weather (temperature + condition) fetched from Open-Meteo, with custom monochrome weather icons (sun, clouds, rain, snow, fog, storm, sleet...)
- Modular embedded Rust architecture (network, time, weather)

## Hardware
- ESP32-C6 Super Mini
- SSD1306 OLED (I2C)
- TTP223 Capacitive Touch Sensor (planned: screen navigation)

## Technologies
- Rust + esp-idf-svc
- embedded-graphics / embedded-hal
- HTTP client over Wi-Fi, JSON parsing with serde
- I2C communication

## Roadmap
- Touch-based navigation between screens (clock / weather / market)
- Custom Macintosh-style "Chicago" font rendering
- Crypto market screen (e.g. Bitcoin price with trend graph)
- Periodic background weather refresh without blocking the UI

## Status
🚧 Work in progress
