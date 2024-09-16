//! see <https://docs.fdsn.org/projects/miniseed3/en/latest/definition.html>

#![allow(deprecated)]

use crate::lib::{fmt, String, ToString, Vec};
use bitflags::bitflags;
use serde::{Deserialize, Serialize};

use crate::schema::FDSNSchema;

type MS3Result<T> = anyhow::Result<T>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
/// The header of a miniSEED 3.0 record.
/// A record is composed of a header followed by a data payload.
/// The byte order of binary fields in the header must be least significant byte first (little endian).
pub struct MS3Header {
    /// Format version. Set to 3 for this version. When a non-backwards compatible change
    /// is introduced the version will be incremented.
    pub(crate) format_version: u8,
    /// Bit field flags, see [`FieldFlag`]
    pub(crate) flag: FieldFlag,
    /// Record start time, time of the first data sample. A representation of UTC using individual fields for:
    /// - nanosecond
    /// - year
    /// - day-of-year
    /// - hour
    /// - minute
    /// - second
    ///
    /// A 60 second value is used to represent a time value during a positive leap second.
    /// If no time series data are included in this record, the time should be relevant for
    /// whatever headers or flags are included.
    pub(crate) start_time: MS3Time,
    ///  A code indicating the encoding format, see [`DataEncodeing`] for a list of valid codes.
    /// If no data payload is included set this value to [`DataEncodeing::Text`].
    pub(crate) data_payload_encoding: DataEncoding,
    /// Sample rate encoded in 64-bit IEEE-754 floating point format. When the value is positive
    /// it represents the rate in samples per second, when it is negative it represents the sample
    /// period in seconds. Creators should use the negative value sample period notation for rates
    /// less than 1 samples per second to retain resolution. Set to 0.0 if no time series data are
    /// included in the record.
    pub(crate) sample_rate: SampleRP,
    /// Total number of data samples in the data payload. Set to 0 if no samples (header-only records)
    /// or unknown number of samples (e.g. for opaque payload encoding).
    pub(crate) sample_count: u32,
    /// CRC-32C (Castagnoli) value of the complete record with the 4-byte CRC field set to zeros.
    /// The CRC-32C (Castagnoli) algorithm with polynomial 0x1EDC6F41 (reversed 0x82F63B78) to be
    /// used is defined in RFC 3309, which further includes references to the relevant background
    /// material.
    pub(crate) castagoli: u32,
    /// Values should only be considered relative to each other for data from the same data center.
    /// Semantics may vary between data centers but generally larger values denote later and more
    /// preferred data. Recommended values: 1 for raw data, 2+ for revisions produced later,
    /// incremented for each revision. A value of 0 indicates unknown version such as when
    /// data are converted to miniSEED from another format. Changes to this value for user-versioning
    /// are not recommended, instead an extra header should be used to allow for user-versioning of
    /// different derivatives of the data.
    pub(crate) data_public_version: u8,
    /// Length, in bytes, of source identifier
    pub(crate) sid_len: u8,
    /// Length, in bytes, of extra headers. If no extra headers, set this value to 0.
    pub(crate) ex_hd_len: u16,
    /// Length, in bytes, of data payload starting in field 15. If no data payload is present,
    /// set this value to 0. Note that no padding is permitted in the data record itself, although
    /// padding may exist within the payload depending on the type of encoding used.
    pub(crate) data_len: u32,
    /// A unique identifier of the source of the data contained in the record. Recommended to
    /// use URI-based identfiers. Commonly an
    /// [FDSN Source Identifier](https://docs.fdsn.org/projects/source-identifiers/).
    pub(crate) sid: String,
    /// See [`FDSNSchema`] and its inner
    pub(crate) ex_hd: Option<FDSNSchema>,
}

impl MS3Header {
    pub(crate) fn default() -> Self {
        Self {
            format_version: 3,
            flag: FieldFlag::CalibrationSignals,
            start_time: MS3Time::default(),
            data_payload_encoding: DataEncoding::Text,
            sample_rate: Default::default(),
            sample_count: Default::default(),
            castagoli: Default::default(),
            data_public_version: Default::default(),
            sid_len: 10,
            ex_hd_len: Default::default(),
            data_len: Default::default(),
            sid: "_mseedio_defaultsid_".to_string(),
            ex_hd: None,
        }
    }

    pub fn bytes(&self) -> MS3Result<Vec<u8>> {
        let mut bytes = Vec::new();
        bytes.push(self.format_version);
        bytes.push(self.flag.bits());
        bytes.extend_from_slice(&self.start_time.nanosecond.to_le_bytes());
        bytes.extend_from_slice(&self.start_time.year.to_le_bytes());
        bytes.extend_from_slice(&self.start_time.day_of_year.to_le_bytes());
        bytes.push(self.start_time.hour);
        bytes.push(self.start_time.minute);
        bytes.push(self.start_time.second);
        bytes.push(self.data_payload_encoding.bits());
        bytes.extend_from_slice(&self.sample_rate.raw.to_le_bytes());
        bytes.extend_from_slice(&self.sample_count.to_le_bytes());
        bytes.extend_from_slice(&self.castagoli.to_le_bytes());
        bytes.push(self.data_public_version);
        bytes.push(self.sid_len);
        bytes.extend_from_slice(&self.ex_hd_len.to_le_bytes());
        bytes.extend_from_slice(&self.data_len.to_le_bytes());
        bytes.extend_from_slice(self.sid.as_bytes());
        if let Some(ex_hd) = &self.ex_hd {
            bytes.extend_from_slice(&serde_json::to_vec(ex_hd).map_err(|e| anyhow::anyhow!(e))?);
        }
        Ok(bytes)
    }
}

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct MS3Time {
    pub(crate) nanosecond: u32,
    pub(crate) year: u16,
    pub(crate) day_of_year: u16,
    pub(crate) hour: u8,
    pub(crate) minute: u8,
    pub(crate) second: u8,
}

const MONTH_DAYS: [u16; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
const MONTH_DAYS_LEAP: [u16; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
macro_rules! is_leap_year {
    ($year:expr) => {
        ($year % 4 == 0 && $year % 100 != 0) || $year % 400 == 0
    };
    () => {};
}
impl fmt::Display for MS3Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut day_of_year = self.day_of_year;
        let mut month = 0;
        // leap year
        if is_leap_year!(self.year) {
            for i in 0..12 {
                if day_of_year <= MONTH_DAYS_LEAP[i] {
                    month = i + 1;
                    break;
                }
                day_of_year -= MONTH_DAYS_LEAP[i];
            }
        } else {
            for i in 0..12 {
                if day_of_year <= MONTH_DAYS[i] {
                    month = i + 1;
                    break;
                }
                day_of_year -= MONTH_DAYS[i];
            }
        }
        write!(
            f,
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:09}Z",
            self.year, month, day_of_year, self.hour, self.minute, self.second, self.nanosecond
        )
    }
}

#[cfg(feature = "chrono")]
use chrono::{Datelike, Timelike};
#[cfg(feature = "chrono")]
impl<Tz: chrono::TimeZone> Into<MS3Time> for chrono::DateTime<Tz> {
    fn into(self) -> MS3Time {
        let utct = self.to_utc();
        MS3Time {
            nanosecond: utct.timestamp_subsec_nanos(),
            year: utct.year() as u16,
            day_of_year: utct.ordinal() as u16,
            hour: utct.hour() as u8,
            minute: utct.minute() as u8,
            second: utct.second() as u8,
        }
    }
}

impl MS3Time {
    pub fn from_parts(
        y: usize,
        m: usize,
        d: usize,
        h: usize,
        min: usize,
        s: usize,
        ns: usize,
    ) -> Self {
        let doy: u16;
        if is_leap_year!(y) {
            doy = MONTH_DAYS_LEAP[..m - 1].iter().sum::<u16>() + d as u16;
        } else {
            doy = MONTH_DAYS[..m - 1].iter().sum::<u16>() + d as u16;
        }
        Self {
            nanosecond: ns as u32,
            year: y as u16,
            day_of_year: doy,
            hour: h as u8,
            minute: min as u8,
            second: s as u8,
        }
    }
}

bitflags! {
    /// UINT8
    /// - 0: Calibration signals present. [same as SEED 2.4 FSDH, field 12, bit 0]
    /// - 1: Time tag is questionable. [same as SEED 2.4 FSDH, field 14, bit 7]
    /// - 2: Clock locked. [same as SEED 2.4 FSDH, field 13, bit 5]
    /// - others: Reserved
    #[derive(Deserialize, Serialize, Debug)]
    pub struct FieldFlag: u8 {
        const CalibrationSignals = 0b001;
        const QuestionableTimeTag = 0b010;
        const ClockLocked = 0b0100;
        const Reserved = !0;
    }
}

bitflags! {
    /// Data payload encodings in the format are identified by a code (number).
    #[derive(Deserialize, Serialize, PartialEq, Eq, Debug, Clone)]
    pub struct DataEncoding: u8 {
        const Text = 0;
        const I16 = 1;
        const I32 = 3;
        const F32 = 4;
        const F64 = 5;
        const Steim1 = 10;
        const Steim2 = 11;
        /// Steim-3 integer compression, big endian (not in common use in archives)
        const Steim3 = 19;
        #[deprecated]
        const I24 = 2;
        #[deprecated]
        const GMI24 = 12;
        #[deprecated]
        const GM16G3 = 13;
        #[deprecated]
        const GM16G4 = 14;
        #[deprecated]
        const USNN = 15;
        #[deprecated]
        const CDSN16 = 16;
        #[deprecated]
        const Grae16 = 17;
        #[deprecated]
        const IPGS16 = 18;
        #[deprecated]
        const SRO = 30;
        #[deprecated]
        const HGLP = 31;
        #[deprecated]
        const DWWSSN = 32;
        #[deprecated]
        const RSTN16 = 33;
        const Reserved = !0;
    }
}

impl From<u8> for DataEncoding {
    fn from(value: u8) -> Self {
        match value {
            0 => DataEncoding::Text,
            1 => DataEncoding::I16,
            3 => DataEncoding::I32,
            4 => DataEncoding::F32,
            5 => DataEncoding::F64,
            10 => DataEncoding::Steim1,
            11 => DataEncoding::Steim2,
            19 => DataEncoding::Steim3,
            2 => DataEncoding::I24,
            12 => DataEncoding::GMI24,
            13 => DataEncoding::GM16G3,
            14 => DataEncoding::GM16G4,
            15 => DataEncoding::USNN,
            16 => DataEncoding::CDSN16,
            17 => DataEncoding::Grae16,
            18 => DataEncoding::IPGS16,
            30 => DataEncoding::SRO,
            31 => DataEncoding::HGLP,
            32 => DataEncoding::DWWSSN,
            33 => DataEncoding::RSTN16,
            _ => DataEncoding::Reserved,
        }
    }
}

impl fmt::Display for DataEncoding {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            &DataEncoding::Text => write!(f, "Text"),
            &DataEncoding::I16 => write!(f, "I16"),
            &DataEncoding::I32 => write!(f, "I32"),
            &DataEncoding::F32 => write!(f, "F32"),
            &DataEncoding::F64 => write!(f, "F64"),
            &DataEncoding::Steim1 => write!(f, "Steim1"),
            &DataEncoding::Steim2 => write!(f, "Steim2"),
            &DataEncoding::Steim3 => write!(f, "Steim3"),
            &DataEncoding::I24 => write!(f, "I24"),
            &DataEncoding::GMI24 => write!(f, "GMI24"),
            &DataEncoding::GM16G3 => write!(f, "GM16G3"),
            &DataEncoding::GM16G4 => write!(f, "GM16G4"),
            &DataEncoding::USNN => write!(f, "USNN"),
            &DataEncoding::CDSN16 => write!(f, "CDSN16"),
            &DataEncoding::Grae16 => write!(f, "Grae16"),
            &DataEncoding::IPGS16 => write!(f, "IPGS16"),
            &DataEncoding::SRO => write!(f, "SRO"),
            &DataEncoding::HGLP => write!(f, "HGLP"),
            &DataEncoding::DWWSSN => write!(f, "DWWSSN"),
            &DataEncoding::RSTN16 => write!(f, "RSTN16"),
            &DataEncoding::Reserved | _ => write!(f, "Reserved"),
        }
    }
}

/// Sample Rate and Period
#[derive(Deserialize, Serialize, Debug)]
pub struct SampleRP {
    pub(crate) raw: f64,
}

impl Into<SampleRP> for f32 {
    fn into(self) -> SampleRP {
        SampleRP { raw: self as f64 }
    }
}

impl Into<SampleRP> for f64 {
    fn into(self) -> SampleRP {
        SampleRP { raw: self }
    }
}

impl SampleRP {
    pub fn get_raw(&self) -> f64 {
        self.raw
    }

    pub fn get_sample_rate(&self) -> f64 {
        if self.raw >= 0.0 {
            self.raw
        } else {
            -1. / self.raw
        }
    }
}

impl Default for SampleRP {
    fn default() -> Self {
        Self { raw: 0.0 }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn dataencodeing() {
        let f64 = DataEncoding::F64;
        assert_eq!(f64.bits(), 5u8);
        let steim2 = DataEncoding::Steim2;
        assert_eq!(steim2.bits(), 11u8);
        let reserved = DataEncoding::Reserved;
        assert!(reserved.is_all());
        let a = DataEncoding::from_bits_retain(102u8);
        match a {
            DataEncoding::Text => unreachable!(),
            DataEncoding::F32 => unreachable!(),
            DataEncoding::F64 => unreachable!(),
            DataEncoding::I16 => unreachable!(),
            DataEncoding::I32 => unreachable!(),
            DataEncoding::Steim1 => unreachable!(),
            DataEncoding::Steim2 => unreachable!(),
            DataEncoding::Steim3 => unreachable!(),
            DataEncoding::I24 => unreachable!(),
            DataEncoding::GMI24 => unreachable!(),
            DataEncoding::GM16G3 => unreachable!(),
            DataEncoding::GM16G4 => unreachable!(),
            DataEncoding::USNN => unreachable!(),
            DataEncoding::CDSN16 => unreachable!(),
            DataEncoding::Grae16 => unreachable!(),
            DataEncoding::IPGS16 => unreachable!(),
            DataEncoding::SRO => unreachable!(),
            DataEncoding::HGLP => unreachable!(),
            DataEncoding::DWWSSN => unreachable!(),
            DataEncoding::RSTN16 => unreachable!(),
            _ => assert!(true),
        }
    }

    #[test]
    fn zero_len_ex_hd() {
        let bytes = serde_json::to_vec(&FDSNSchema::new()).unwrap();
        assert_ne!(bytes.len(), 0); // Caution! this is not zero
    }

    #[test]
    fn sid_len() {
        let sid = "FDSN:XX_TEST__L_H_Z";
        assert_eq!(sid.bytes().len(), 19)
    }
}
