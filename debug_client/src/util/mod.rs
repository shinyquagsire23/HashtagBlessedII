#[cfg(feature = "termion")]
pub mod event;
use crate::{get_sparkline, get_sparkline_idx};

#[derive(Clone)]
pub struct StaticSignal {
    last_idx: u64
}

impl StaticSignal {
    pub fn new() -> StaticSignal {
        StaticSignal {
            last_idx: 0
        }
    }
}

impl Iterator for StaticSignal {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        if get_sparkline_idx() != self.last_idx {
            self.last_idx = get_sparkline_idx();
            Some(get_sparkline())
        }
        else
        {
            None
        }
    }
}
