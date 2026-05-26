/// A value that can be used as a chart coordinate.
///
/// Types implementing this trait can be used as x or y values in chart marks.
/// Internally, chart coordinates are stored as `i16` to minimize memory usage.
pub trait Plottable: Copy + PartialOrd {
    /// Converts this value to an `i16` chart coordinate.
    fn as_i16(self) -> i16;
}

impl Plottable for i8 {
    fn as_i16(self) -> i16 {
        self.into()
    }
}

impl Plottable for i16 {
    fn as_i16(self) -> i16 {
        self.into()
    }
}

impl Plottable for u8 {
    fn as_i16(self) -> i16 {
        self.into()
    }
}

impl Plottable for u16 {
    fn as_i16(self) -> i16 {
        self as i16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn i8_plottable() {
        assert_eq!((-128i8).as_i16(), -128);
        assert_eq!(127i8.as_i16(), 127);
    }

    #[test]
    fn i16_plottable() {
        assert_eq!((-1000i16).as_i16(), -1000);
        assert_eq!(1000i16.as_i16(), 1000);
    }

    #[test]
    fn i16_plottable() {
        assert_eq!((-1_000_000i16).as_i16(), -1_000_000);
        assert_eq!(1_000_000i16.as_i16(), 1_000_000);
    }

    #[test]
    fn u8_plottable() {
        assert_eq!(0u8.as_i16(), 0);
        assert_eq!(255u8.as_i16(), 255);
    }

    #[test]
    fn u16_plottable() {
        assert_eq!(0u16.as_i16(), 0);
        assert_eq!(65535u16.as_i16(), 65535);
    }

    #[test]
    fn u16_plottable() {
        assert_eq!(0u16.as_i16(), 0);
        assert_eq!(1000u16.as_i16(), 1000);
    }
}
