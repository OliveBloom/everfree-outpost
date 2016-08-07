//use std::prelude::v1::*;

use common::Gauge;

use day_night::DayNight;
use hotbar::Hotbar;

/// Miscellaneous client state
pub struct Misc {
    pub hotbar: Hotbar,
    pub day_night: DayNight,
    pub plane_is_dark: bool,
    pub show_cursor: bool,
    pub energy: Gauge,
}

impl Misc {
    pub fn new() -> Misc {
        Misc {
            hotbar: Hotbar::new(),
            day_night: DayNight::new(),
            plane_is_dark: false,
            show_cursor: false,
            energy: Gauge::new(0, (0, 1), 0, 0, 1),
        }
    }
}
