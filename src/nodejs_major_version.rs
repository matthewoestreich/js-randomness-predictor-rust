use clap::ValueEnum;
use std::fmt::{Display, Formatter, Result};

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum NodeJsMajorVersion {
  V0 = 0,
  V4 = 4,
  V5 = 5,
  V6 = 6,
  V7 = 7,
  V8 = 8,
  V9 = 9,
  V10 = 10,
  V11 = 11,
  V12 = 12,
  V13 = 13,
  V14 = 14,
  V15 = 15,
  V16 = 16,
  V17 = 17,
  V18 = 18,
  V19 = 19,
  V20 = 20,
  V21 = 21,
  V22 = 22,
  V23 = 23,
  V24 = 24,
}

impl NodeJsMajorVersion {
  #[allow(dead_code)]
  pub fn from_u8(value: u8) -> Option<Self> {
    match value {
      0 => Some(Self::V0),
      4 => Some(Self::V4),
      5 => Some(Self::V5),
      6 => Some(Self::V6),
      7 => Some(Self::V7),
      8 => Some(Self::V8),
      9 => Some(Self::V9),
      10 => Some(Self::V10),
      11 => Some(Self::V11),
      12 => Some(Self::V12),
      13 => Some(Self::V13),
      14 => Some(Self::V14),
      15 => Some(Self::V15),
      16 => Some(Self::V16),
      17 => Some(Self::V17),
      18 => Some(Self::V18),
      19 => Some(Self::V19),
      20 => Some(Self::V20),
      21 => Some(Self::V21),
      22 => Some(Self::V22),
      23 => Some(Self::V23),
      24 => Some(Self::V24),
      _ => None,
    }
  }
}

impl Display for NodeJsMajorVersion {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    write!(f, "v{}", *self as u8)
  }
}
