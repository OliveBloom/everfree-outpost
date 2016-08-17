//use std::prelude::v1::*;

use common::Gauge;

use day_night::DayNight;
use hotbar::Hotbar;
use inv_changes::InvChanges;

/// Miscellaneous client state
pub struct Misc {
    pub hotbar: Hotbar,
    pub day_night: DayNight,
    pub inv_changes: InvChanges,
    // TODO: move this to Pawn
    pub energy: Gauge,
    pub plane_is_dark: bool,
    pub show_cursor: bool,
}

impl Misc {
    pub fn new() -> Misc {
        Misc {
            hotbar: Hotbar::new(),
            day_night: DayNight::new(),
            inv_changes: InvChanges::new(),
            energy: Gauge::new(0, (0, 1), 0, 0, 1),
            plane_is_dark: false,
            show_cursor: false,
        }
    }

    pub fn reset(&mut self) {
        self.inv_changes.clear();
        self.energy = Gauge::new(0, (0, 1), 0, 0, 1);
    }
}
