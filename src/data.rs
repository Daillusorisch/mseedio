use crate::lib::{default, fmt, Cell, String, Vec};

pub mod consts {
    pub const FIXED_HEADER_LEN: usize = 40;
    // pub const FORMAT_VERSION_OFFSET: usize = 2;
    // pub const FLAGS_OFFSET: usize = 3;
    // pub const NANOSECOND_OFFSET: usize = 4;
    // pub const YEAR_OFFSET: usize = 8;
    // pub const DAY_OF_YEAR_OFFSET: usize = 10;
    // pub const HOUR_OFFSET: usize = 12;
    // pub const MINUTE_OFFSET: usize = 13;
    // pub const SECOND_OFFSET: usize = 14;
    // pub const DATA_PAYLOAD_ENCODING_OFFSET: usize = 15;
    // pub const SAMPLE_RATE_OFFSET: usize = 16;
    // pub const SAMPLE_COUNT_OFFSET: usize = 24;
    pub const CASTAGOLI_OFFSET: usize = 28;
    // pub const DATA_PUBLIC_VERSION_OFFSET: usize = 32;
    // pub const SID_LENGTH_OFFSET: usize = 33;
    // pub const EXTRA_HEADERS_LENGTH_OFFSET: usize = 34;
    // pub const DATA_PAYLOAD_LENGTH_OFFSET: usize = 36;
}

/// format_version:
///
/// |  type  | length | offset |
/// |--------|--------|--------|
/// | UINT8  |   1    |    2   |
///
macro_rules! fversion {
    ($fixhd:expr) => {
        $fixhd[2]
    };
}

/// flags:
///
/// |  type  | length | offset |
/// |--------|--------|--------|
/// | UINT8  |   1    |    3   |
///
macro_rules! flags {
    ($fixhd:expr) => {
        $fixhd[3]
    };
}

/// nanosecond:
///
/// |  type  | length | offset |
/// |--------|--------|--------|
/// | UINT32 |   4    |    4   |
///
macro_rules! nanosecond {
    ($fixhd:expr) => {
        u32::from_le_bytes([$fixhd[4], $fixhd[5], $fixhd[6], $fixhd[7]])
    };
}

/// year:
///
/// |  type  | length | offset |
/// |--------|--------|--------|
/// | UINT16 |   2    |    8   |
///
macro_rules! year {
    ($fixhd:expr) => {
        u16::from_le_bytes([$fixhd[8], $fixhd[9]])
    };
}

/// day_of_year:
///
/// |  type  | length | offset |
/// |--------|--------|--------|
/// | UINT16 |   2    |   10   |
///
macro_rules! day_of_year {
    ($fixhd:expr) => {
        u16::from_le_bytes([$fixhd[10], $fixhd[11]])
    };
}

/// hour:
///
/// |  type  | length | offset |
/// |--------|--------|--------|
/// | UINT8  |   1    |   12   |
///  
macro_rules! hour {
    ($fixhd:expr) => {
        $fixhd[12]
    };
}

/// minute:
///
/// |  type  | length | offset |
/// |--------|--------|--------|
/// | UINT8  |   1    |   13   |
///
macro_rules! minute {
    ($fixhd:expr) => {
        $fixhd[13]
    };
}

/// second:
///
/// |  type  | length | offset |
/// |--------|--------|--------|
/// | UINT8  |   1    |   14   |
///
macro_rules! second {
    ($fixhd:expr) => {
        $fixhd[14]
    };
}

/// data payload encoding:
///
/// |  type  | length | offset |
/// |--------|--------|--------|
/// | UINT8  |   1    |   15   |
///
macro_rules! data_payload_encoding {
    ($fixhd:expr) => {
        $fixhd[15]
    };
}

/// sample_rate:
///
/// |   type   | length | offset |
/// |----------|--------|--------|
/// | FLOAT64  |   8    |   16   |
///
macro_rules! sample_rate {
    ($fixhd:expr) => {
        f64::from_le_bytes([
            $fixhd[16], $fixhd[17], $fixhd[18], $fixhd[19], $fixhd[20], $fixhd[21], $fixhd[22],
            $fixhd[23],
        ])
    };
}

/// sample count:
///
/// |  type  | length | offset |
/// |--------|--------|--------|
/// | UINT32 |   4    |   24   |
///
macro_rules! sample_count {
    ($fixhd:expr) => {
        u32::from_le_bytes([$fixhd[24], $fixhd[25], $fixhd[26], $fixhd[27]])
    };
}

/// castagoli:
///
/// |  type  | length | offset |
/// |--------|--------|--------|
/// | UINT32 |   4    |   28   |
///
macro_rules! castagoli {
    ($fixhd:expr) => {
        u32::from_le_bytes([$fixhd[28], $fixhd[29], $fixhd[30], $fixhd[31]])
    };
}

/// data public version:
///
/// |  type  | length | offset |
/// |--------|--------|--------|
/// | UINT8  |   1    |   32   |
///
macro_rules! data_public_version {
    ($fixhd:expr) => {
        $fixhd[32]
    };
}

/// length of sid:
///
/// |  type  | length | offset |
/// |--------|--------|--------|
/// | UINT8  |   1    |   33   |
///
macro_rules! sid_length {
    ($fixhd:expr) => {
        $fixhd[33]
    };
}

/// length of extra headers:
///
/// |  type  | length | offset |
/// |--------|--------|--------|
/// | UINT16 |   2    |   34   |
///
macro_rules! extra_headers_length {
    ($fixhd:expr) => {
        u16::from_le_bytes([$fixhd[34], $fixhd[35]])
    };
}

/// length of data payload:
///
/// |  type  | length | offset |
/// |--------|--------|--------|
/// | UINT32 |   4    |   36   |
///
macro_rules! data_payload_length {
    ($fixhd:expr) => {
        u32::from_le_bytes([$fixhd[36], $fixhd[37], $fixhd[38], $fixhd[39]])
    };
}

/// The body of a miniseed record.
/// encodeing format and length should be determined by [`crate::MS3Header`]
pub struct MS3Data {
    pub(crate) raw: Vec<u8>,
    pub(crate) decoded: Cell<DecodedData>,
}

impl fmt::Debug for MS3Data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let temp = self.decoded.take();
        let _ = match temp {
            DecodedData::None => write!(f, "DecodedData::None"),
            DecodedData::Text(ref s) => write!(f, "DecodedData::Text({})", s),
            DecodedData::F32(ref v) => write!(f, "DecodedData::F32({:?})", v),
            DecodedData::F64(ref v) => write!(f, "DecodedData::F64({:?})", v),
            DecodedData::I16(ref v) => write!(f, "DecodedData::I16({:?})", v),
            DecodedData::I32(ref v) => write!(f, "DecodedData::I32({:?})", v),
            DecodedData::Opaque(ref v) => write!(f, "DecodedData::Opaque({:?})", v),
        };
        self.decoded.set(temp);
        Ok(())
    }
}

impl Default for MS3Data {
    fn default() -> Self {
        Self {
            raw: Vec::new(),
            decoded: Cell::new(DecodedData::None),
        }
    }
}

impl MS3Data {
    pub fn raw(&self) -> &[u8] {
        &self.raw
    }

    pub fn raw_mut(&mut self) -> &mut Vec<u8> {
        &mut self.raw
    }
}

#[derive(Debug, Clone)]
pub enum DecodedData {
    None,
    Text(String),
    F32(Vec<f32>),
    F64(Vec<f64>),
    I16(Vec<i16>),
    I32(Vec<i32>),
    Opaque(Vec<u8>),
}

impl default::Default for DecodedData {
    fn default() -> Self {
        Self::None
    }
}
