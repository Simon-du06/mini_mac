mod clock;
pub mod display;
mod glucose;
mod market;
mod weather;

pub use clock::draw_clock;
pub use glucose::draw_glucose;
pub use market::draw_market;
pub use weather::draw_weather;

pub enum Screen {
    Clock,
    Weather,
    Crypto,
    Market,
    Glucose,
}

impl Screen {
    pub fn next(self) -> Self {
        match self {
            Screen::Clock => Screen::Weather,
            Screen::Weather => Screen::Crypto,
            Screen::Crypto => Screen::Market,
            Screen::Market => Screen::Glucose,
            Screen::Glucose => Screen::Clock,
        }
    }
}
