use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(transparent)]
pub struct ByteCount(pub u64);

impl ByteCount {
    pub const fn bytes(b: f64) -> Self {
        Self(b.round() as u64)
    }
    pub const fn kibibytes(kib: f64) -> Self {
        Self::bytes(kib * 1024f64)
    }
    pub const fn mebibytes(mib: f64) -> Self {
        Self::bytes(mib * 1024f64 * 1024f64)
    }
    pub const fn gibibytes(gib: f64) -> Self {
        Self::bytes(gib * 1024f64 * 1024f64 * 1024f64)
    }
    pub const fn tebibytes(tib: f64) -> Self {
        Self::bytes(tib * 1024f64 * 1024f64 * 1024f64 * 1024f64)
    }
    pub const fn kilobytes(kb: f64) -> Self {
        Self::bytes(kb * 1_000f64)
    }
    pub const fn megabytes(mb: f64) -> Self {
        Self::bytes(mb * 1_000_000f64)
    }
    pub const fn gigabytes(gb: f64) -> Self {
        Self::bytes(gb * 1_000_000_000f64)
    }
    pub const fn terabytes(tb: f64) -> Self {
        Self::bytes(tb * 1_000_000_000_000f64)
    }
}

impl fmt::Display for ByteCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let b = self.0 as f64;
        let kib = b / 1024.0;
        let mib = kib / 1024.0;
        let gib = mib / 1024.0;
        let tib = gib / 1024.0;
        if tib > 1.0 {
            write!(f, "{tib:.2} TiB")?;
        } else if gib > 1.0 {
            write!(f, "{gib:.2} GiB")?;
        } else if mib > 1.0 {
            write!(f, "{mib:.2} MiB")?;
        } else if kib > 1.0 {
            write!(f, "{kib:.2} KiB")?;
        } else {
            write!(f, "{} B", self.0)?;
        }
        Ok(())
    }
}

impl From<u64> for ByteCount {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<ByteCount> for u64 {
    fn from(value: ByteCount) -> Self {
        value.0
    }
}
