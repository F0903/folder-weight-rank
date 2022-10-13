use std::fmt::Display;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SizeUnit {
    Byte,
    KiloByte,
    MegaByte,
    GigaByte,
}

impl Display for SizeUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            SizeUnit::Byte => "B",
            SizeUnit::KiloByte => "KB",
            SizeUnit::MegaByte => "MB",
            SizeUnit::GigaByte => "GB",
        })
    }
}

pub struct FileSize {
    size: f64,
    unit: SizeUnit,
}

impl FileSize {
    pub fn new(size: f64, unit: SizeUnit) -> Self {
        Self { size, unit }
    }

    pub fn bytes(size: usize) -> Self {
        Self {
            size: size as f64,
            unit: SizeUnit::Byte,
        }
    }

    pub fn get_as(self, unit: SizeUnit) -> Self {
        if self.unit == unit {
            return self;
        }
        let relative_unit_num = (self.unit as i32) - (unit as i32);
        let is_negative = relative_unit_num < 0;
        let exp = relative_unit_num.unsigned_abs();
        let unit_pow = (1000 as i32).pow(exp);
        let to_mult = if is_negative {
            1.0 / unit_pow as f64
        } else {
            unit_pow as f64
        };
        let size = self.size as f64 * to_mult;
        Self::new(size, unit)
    }

    pub fn get(&self) -> f64 {
        self.size
    }
}

impl Display for FileSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:.3}{}", self.size, self.unit))
    }
}
