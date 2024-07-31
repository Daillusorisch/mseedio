#![no_std]
#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("mseedio crate requires either std or alloc feature to be enabled");

pub use crate::data::DecodedData;
pub use crate::header::DataEncoding;
use anyhow::{anyhow, Ok};
pub use data::MS3Data;
pub use header::{FieldFlag, MS3Header, MS3Time, SampleRP};
pub use schema::FDSNSchema;
use utils::is_big_endian;

#[macro_use]
mod data;
mod header;
mod schema;
#[macro_use]
mod utils;

use crate::data::consts::*;
const CRC32C: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_ISCSI); // iSCSI CRC-32C

mod lib {
    #[cfg(feature = "alloc")]
    pub use alloc::{
        format,
        string::{String, ToString},
        vec::Vec,
    };
    #[cfg(feature = "alloc")]
    pub use core::{cell::Cell, default, fmt, mem};
    #[cfg(feature = "std")]
    pub use std::{
        cell::Cell,
        default, fmt, format, mem,
        path::PathBuf,
        string::{String, ToString},
        vec::Vec,
    };
}

#[cfg(feature = "std")]
use lib::PathBuf;
use lib::{format, Cell, String, ToString, Vec};

type MS3Result<T> = anyhow::Result<T>;

pub trait Steim1Decode {
    fn decode_steim1(&self, swap_flag: bool) -> anyhow::Result<Vec<i32>>;
}

pub trait Steim2Decode {
    fn decode_steim2(&self, swap_flag: bool) -> anyhow::Result<Vec<i32>>;
}

pub trait Steim3Decode {
    fn decode_steim3(&self, swap_flag: bool) -> anyhow::Result<Vec<i32>>;
}

pub trait Steim1Encode {
    fn encode_steim1(&self, diff_0: i32) -> anyhow::Result<Vec<u8>>;
}

pub trait Steim2Encode {
    fn encode_steim2(&self, diff_0: i32) -> anyhow::Result<Vec<u8>>;
}

pub trait Steim3Encode {
    fn encode_steim3(&self, diff_0: i32) -> anyhow::Result<Vec<u8>>;
}

pub struct MS3RecordBuilder {
    pub(crate) header: MS3Header,
    pub(crate) data: MS3Data,
}

impl MS3RecordBuilder {
    pub fn new() -> Self {
        Self {
            header: MS3Header::default(),
            data: MS3Data::default(),
        }
    }

    pub fn flag(mut self, flag: FieldFlag) -> Self {
        self.header.flag = flag;
        self
    }

    pub fn start_time<T: Into<MS3Time>>(mut self, start_time: T) -> Self {
        self.header.start_time = start_time.into();
        self
    }

    pub fn data_payload_encoding(mut self, encoding: DataEncoding) -> Self {
        self.header.data_payload_encoding = encoding;
        self
    }

    pub fn sample_rate<T: Into<SampleRP>>(mut self, sample_rate: T) -> Self {
        self.header.sample_rate = sample_rate.into();
        self
    }

    pub fn data_public_version(mut self, data_public_version: u8) -> Self {
        self.header.data_public_version = data_public_version;
        self
    }

    pub fn sid<T: ToString>(mut self, sid: T) -> MS3Result<Self> {
        let sid = sid.to_string();
        if sid.len() > u8::MAX.into() {
            return Err(anyhow!("SID too long"));
        }
        self.header.sid = sid;
        self.header.sid_len = self.header.sid.len() as u8;
        Ok(self)
    }

    pub fn data(mut self, data: DecodedData) -> Self {
        self.data = MS3Data {
            raw: Vec::new(),
            decoded: data.into(),
        };
        self
    }

    pub fn extra_header(mut self, extra_header: FDSNSchema) -> MS3Result<Self> {
        let extra_header_len = serde_json::to_vec(&extra_header)
            .map_err(|e| {
                anyhow!(
                    "Can't build MS3Record: Failed to serialize extra header: {}",
                    e
                )
            })?
            .len();
        if extra_header_len > u16::MAX.into() {
            return Err(anyhow!("Can't build MS3Record: Extra header too long"));
        }
        self.header.ex_hd_len = extra_header_len as u16;
        self.header.ex_hd = Some(extra_header);
        Ok(self)
    }

    /// build the record with diff0 = 0
    /// CRC will be calculated automatically
    pub fn build(self) -> MS3Result<MS3Record> {
        self.build_with_diff0(0)
    }

    /// build the record with provided diff0
    /// CRC will be calculated automatically
    pub fn build_with_diff0(mut self, diff_0: i32) -> MS3Result<MS3Record> {
        let data_encoding = self.header.data_payload_encoding.clone();

        if data_encoding == DataEncoding::Reserved {
            return Err(anyhow!("Please set data encoding"));
        }
        let unencoded = self.data.decoded.take();
        self.header.sample_count = match &unencoded {
            DecodedData::I16(data) => data.len() as u32,
            DecodedData::I32(data) => data.len() as u32,
            DecodedData::F32(data) => data.len() as u32,
            DecodedData::F64(data) => data.len() as u32,
            DecodedData::Text(data) => data.len() as u32,
            DecodedData::Opaque(data) => data.len() as u32,
            DecodedData::None => 0,
        };
        match data_encoding {
            DataEncoding::I16 => match unencoded {
                DecodedData::I16(data) => {
                    self.data.raw = data.iter().flat_map(|d| d.to_le_bytes()).collect();
                }
                _ => {
                    return Err(anyhow!("Invalid data type: {}", data_encoding));
                }
            },
            DataEncoding::I32 => match unencoded {
                DecodedData::I32(data) => {
                    self.data.raw = data.iter().flat_map(|d| d.to_le_bytes()).collect();
                }
                _ => {
                    return Err(anyhow!("Invalid data type: {}", data_encoding));
                }
            },
            DataEncoding::F32 => match unencoded {
                DecodedData::F32(data) => {
                    self.data.raw = data.iter().flat_map(|d| d.to_le_bytes()).collect();
                }
                _ => {
                    return Err(anyhow!("Invalid data type: {}", data_encoding));
                }
            },
            DataEncoding::F64 => match unencoded {
                DecodedData::F64(data) => {
                    self.data.raw = data.iter().flat_map(|d| d.to_le_bytes()).collect();
                }
                _ => {
                    return Err(anyhow!("Invalid data type: {}", data_encoding));
                }
            },
            DataEncoding::Text => match unencoded {
                DecodedData::Text(data) => {
                    self.data.raw = data.as_bytes().to_vec();
                }
                _ => {
                    return Err(anyhow!("Invalid data type: {}", data_encoding));
                }
            },
            DataEncoding::Steim1 => {
                // encode steim will do the type check
                self.data.decoded.set(unencoded);
                let encoded = self.encode_steim1(diff_0)?;
                self.data.raw = encoded;
            }
            DataEncoding::Steim2 => {
                // encode steim will do the type check
                self.data.decoded.set(unencoded);
                let encoded = self.encode_steim2(diff_0)?;
                self.data.raw = encoded;
            }
            DataEncoding::Steim3 => {
                // encode steim will do the type check
                self.data.decoded.set(unencoded);
                let encoded = self.encode_steim3(diff_0)?;
                self.data.raw = encoded;
            }

            #[allow(deprecated)]
            DataEncoding::CDSN16
            | DataEncoding::DWWSSN
            | DataEncoding::GM16G3
            | DataEncoding::GM16G4
            | DataEncoding::GMI24
            | DataEncoding::Grae16
            | DataEncoding::HGLP
            | DataEncoding::I24
            | DataEncoding::SRO
            | DataEncoding::USNN
            | DataEncoding::RSTN16
            | DataEncoding::IPGS16 => {
                return Err(anyhow!("Deprecated data encoding"));
            }
            DataEncoding::Reserved | _ => {
                return Err(anyhow!("Reserved data encoding"));
            }
        }
        self.header.data_len = self.data.raw.len() as u32;
        let mut cs_bytes = Vec::new();
        cs_bytes.extend_from_slice(b"MS");
        cs_bytes.extend_from_slice(&self.header.bytes()?);
        cs_bytes.extend_from_slice(&self.data.raw);
        self.header.castagoli = CRC32C.checksum(&cs_bytes);
        Ok(MS3Record {
            header: self.header,
            data: self.data,
        })
    }
}

/// The fundamental unit of miniseed.
///
/// A time series is commonly stored and exchanged as a sequence of these records.
/// There is no interdependence of records, each is independent. There are data
/// encodings for integers, floats, text or compressed data samples. To limit
/// problems with timing system drift and resolution in addition to practical
/// issues of subsetting and resource limitation for readers of the data,
/// typical record lengths for raw data generation and archiving are recommended
/// to be in the range of 256 and 4096 bytes.
#[derive(Debug)]
pub struct MS3Record {
    header: MS3Header,
    data: MS3Data,
}

impl MS3Record {
    pub fn summary(&self) -> String {
        format!(
            "MS3Record: SID: {}, Time: {}, Sample Rate: {} Hz, Sample Count: {}, Data Len: {}",
            self.sid(),
            self.time(),
            self.sample_rate(),
            self.sample_count(),
            self.data_len()
        )
    }

    pub fn data_raw(&self) -> &[u8] {
        self.data.raw()
    }

    pub fn time(&self) -> &MS3Time {
        &self.header.start_time
    }

    pub fn extra_header(&self) -> Option<&FDSNSchema> {
        self.header.ex_hd.as_ref()
    }

    pub fn encode_type(&self) -> DataEncoding {
        self.header.data_payload_encoding.clone()
    }

    pub fn sid(&self) -> &str {
        &self.header.sid
    }

    /// decoded simple rate
    pub fn sample_rate(&self) -> f64 {
        self.header.sample_rate.get_sample_rate()
    }

    pub fn sample_count(&self) -> u32 {
        self.header.sample_count
    }

    pub fn castagoli(&self) -> u32 {
        self.header.castagoli
    }

    pub fn data_public_version(&self) -> u8 {
        self.header.data_public_version
    }

    pub fn data_len(&self) -> u32 {
        self.header.data_len
    }

    pub fn ex_hd_len(&self) -> u16 {
        self.header.ex_hd_len
    }

    /// get decoded data
    /// return Err if data is empty
    pub fn data(&self) -> MS3Result<DecodedData> {
        let raw = self.data.raw();
        if raw.is_empty() {
            return Err(anyhow!("Data is empty"));
        }
        let data = self.data.decoded.take();

        match data {
            DecodedData::None => match self.header.data_payload_encoding {
                DataEncoding::Text => {
                    log::debug!("decoding text data...");
                    let res = DecodedData::Text(
                        String::from_utf8(raw.to_vec())
                            .map_err(|e| anyhow!("Invalid UTF-8: {}", e))?,
                    );
                    Ok(res)
                }
                DataEncoding::I16 => {
                    log::debug!("decoding i16 data...");
                    let res = DecodedData::I16(decode_data!(raw, i16, 2));
                    Ok(res)
                }
                DataEncoding::I32 => {
                    log::debug!("decoding i32 data...");
                    let res = DecodedData::I32(decode_data!(raw, i32, 4));
                    Ok(res)
                }
                DataEncoding::F32 => {
                    log::debug!("decoding f32 data...");
                    let res = DecodedData::F32(decode_data!(raw, f32, 4));
                    Ok(res)
                }
                DataEncoding::F64 => {
                    log::debug!("decoding f64 data...");
                    let res = DecodedData::F64(decode_data!(raw, f64, 8));
                    Ok(res)
                }
                DataEncoding::Steim1 => {
                    log::debug!("decoding steim1 data...");
                    let res = DecodedData::I32(self.decode_steim1(is_big_endian())?);
                    Ok(res)
                }
                DataEncoding::Steim2 => {
                    log::debug!("decoding steim2 data...");
                    let res = DecodedData::I32(self.decode_steim2(is_big_endian())?);
                    Ok(res)
                }
                DataEncoding::Steim3 => {
                    log::debug!("decoding steim3 data...");
                    let res = DecodedData::I32(self.decode_steim3(is_big_endian())?);
                    Ok(res)
                }
                // deprecated data encoding will not be supported
                #[allow(deprecated)]
                DataEncoding::CDSN16
                | DataEncoding::DWWSSN
                | DataEncoding::GM16G3
                | DataEncoding::GM16G4
                | DataEncoding::GMI24
                | DataEncoding::Grae16
                | DataEncoding::HGLP
                | DataEncoding::I24
                | DataEncoding::SRO
                | DataEncoding::USNN
                | DataEncoding::RSTN16
                | DataEncoding::IPGS16 => {
                    log::warn!(
                        "Deprecated data encoding: {:?}",
                        self.header.data_payload_encoding
                    );
                    Err(anyhow!(
                        "Deprecated data encoding: {:?}",
                        self.header.data_payload_encoding
                    ))
                }
                // all other bits are reserved
                DataEncoding::Reserved | _ => {
                    log::warn!("Reserved data encoding");
                    Err(anyhow!("Reserved data encoding"))
                }
            },
            _ => {
                log::debug!("data already decoded...using cache");
                self.data.decoded.set(data.clone());
                Ok(data)
            }
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> MS3Result<Self> {
        is_vaild_ms3(bytes)?;
        let record = MS3Record::from_bytes_uncheck(bytes)?;
        let mut ob = Vec::with_capacity(bytes.len());
        ob.extend_from_slice(&bytes[..CASTAGOLI_OFFSET]);
        ob.extend_from_slice(&[0u8; 4]);
        ob.extend_from_slice(&bytes[(CASTAGOLI_OFFSET + 4)..]);
        let c = record.castagoli();
        log::trace!("check crc32... {:?}", c);
        if c != CRC32C.checksum(&ob) {
            log::warn!("Invalid MS3: Broken record: Castagoli mismatch");
            return Err(anyhow!("Invalid MS3: Broken record: Castagoli mismatch"));
        }
        Ok(record)
    }

    pub fn from_bytes_uncheck(bytes: &[u8]) -> MS3Result<Self> {
        log::debug!("parse bytes chunk...");
        let fixed_header = bytes
            .get(0..FIXED_HEADER_LEN)
            .ok_or("Invalid MS3: Broken record header")
            .map_err(|e| anyhow!(e))?;
        let sid_len = sid_length!(fixed_header);
        let extra_header_len = extra_headers_length!(fixed_header);
        let data_len = data_payload_length!(fixed_header);
        let sid = String::from_utf8(
            bytes
                .get(FIXED_HEADER_LEN..(FIXED_HEADER_LEN + sid_len as usize))
                .ok_or("Invalid MS3: Broken record SID")
                .map_err(|e| anyhow!(e))?
                .to_vec(),
        )
        .map_err(|e| anyhow!("Invalid MS3: Can not decode SID: {}", e))?;

        let extra_header = if extra_header_len == 0 {
            FDSNSchema::new()
        } else {
            serde_json::from_slice::<FDSNSchema>(
                bytes
                    .get(
                        (FIXED_HEADER_LEN + sid_len as usize)
                            ..(FIXED_HEADER_LEN + sid_len as usize + extra_header_len as usize),
                    )
                    .ok_or("Invalid MS3: Broken record extra header")
                    .map_err(|e| anyhow!(e))?,
            )
            .map_err(|e| anyhow!("Invalid MS3: Can not decode extra header: {}", e))?
        };
        let flag = FieldFlag::from_bits(flags!(fixed_header))
            .ok_or("Invalid MS3: Broken record flag")
            .map_err(|e| anyhow!(e))?;
        let data_payload_encoding = DataEncoding::from(data_payload_encoding!(fixed_header));
        let data = MS3Data {
            raw: bytes
                .get(
                    (FIXED_HEADER_LEN + sid_len as usize + extra_header_len as usize)
                        ..(FIXED_HEADER_LEN
                            + sid_len as usize
                            + extra_header_len as usize
                            + data_len as usize),
                )
                .ok_or("Invalid MS3: Broken record data")
                .map_err(|e| anyhow!(e))?
                .to_vec(),
            // Some types of data encoding has fixed size
            decoded: Cell::new(DecodedData::None),
        };
        Ok(Self {
            header: MS3Header {
                format_version: fversion!(fixed_header),
                flag,
                start_time: MS3Time {
                    nanosecond: nanosecond!(fixed_header),
                    year: year!(fixed_header),
                    day_of_year: day_of_year!(fixed_header),
                    hour: hour!(fixed_header),
                    minute: minute!(fixed_header),
                    second: second!(fixed_header),
                },
                data_payload_encoding,
                sample_rate: SampleRP {
                    raw: sample_rate!(fixed_header),
                },
                sample_count: sample_count!(fixed_header),
                castagoli: castagoli!(fixed_header),
                data_public_version: data_public_version!(fixed_header),
                sid_len,
                ex_hd_len: extra_header_len,
                data_len,
                sid,
                ex_hd: Some(extra_header),
            },
            data,
        })
    }

    pub fn to_bytes(&self) -> MS3Result<Vec<u8>> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"MS");
        bytes.extend_from_slice(&self.header.bytes()?);
        bytes.extend_from_slice(&self.data.raw);
        Ok(bytes)
    }
}

/// A collection of [`MS3Record`], general miniseed file form
#[derive(Debug)]
pub struct MS3Volume {
    records: Vec<MS3Record>,
}

impl MS3Volume {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: MS3Record) -> &mut Self {
        self.records.push(record);
        self
    }

    pub fn add_records(&mut self, records: Vec<MS3Record>) -> &mut Self {
        self.records.extend(records);
        self
    }

    #[cfg(feature = "std")]
    /// read MS3Volume from file, only available with `std` feature
    pub fn from_file<T: Into<PathBuf>>(path: T) -> MS3Result<Self> {
        let file = std::fs::read(path.into())?;
        MS3Volume::from_bytes(file)
    }

    pub fn from_bytes(bytes: Vec<u8>) -> MS3Result<Self> {
        log::debug!("try parse MS3Volume from bytes...");
        let records_chunks = devide_into_record_chunk(&bytes)?;
        let mut records = Vec::new();
        for r in records_chunks.iter() {
            r.to_vec();
            records.push(MS3Record::from_bytes(r)?);
        }
        Ok(Self { records })
    }

    #[cfg(feature = "std")]
    pub fn to_file<T: Into<PathBuf>>(&self, path: T) -> MS3Result<()> {
        let bytes = self.to_bytes()?;
        std::fs::write(path.into(), bytes)?;
        Ok(())
    }

    pub fn to_bytes(&self) -> MS3Result<Vec<u8>> {
        let bytes = self
            .records
            .iter()
            .flat_map(|r| r.to_bytes())
            .collect::<Vec<Vec<u8>>>()
            .concat();
        Ok(bytes)
    }
}

impl Iterator for MS3Volume {
    type Item = MS3Record;

    fn next(&mut self) -> Option<Self::Item> {
        self.records.pop()
    }
}

pub fn devide_into_record_chunk(bytes: &[u8]) -> MS3Result<Vec<&[u8]>> {
    // check is it valid ms3 bytes
    is_vaild_ms3(bytes)?;
    let mut readed_len: usize = 0;
    let mut records = Vec::new();
    let bytes_len = bytes.len();
    while bytes_len > readed_len {
        is_vaild_ms3(&bytes[readed_len..])?;
        let fixed_header = bytes
            .get(readed_len..(readed_len + FIXED_HEADER_LEN))
            .ok_or("Invalid MS3: Broken record header")
            .map_err(|e| anyhow!(e))?;
        let sid_len = sid_length!(fixed_header);
        let extra_header_len = extra_headers_length!(fixed_header);
        let data_len = data_payload_length!(fixed_header);
        let record_len =
            FIXED_HEADER_LEN + sid_len as usize + extra_header_len as usize + data_len as usize;
        records.push(
            bytes
                .get(readed_len..(readed_len + record_len))
                .ok_or("Invalid MS3: Broken record data")
                .map_err(|e| anyhow!(e))?,
        );
        readed_len += record_len;
    }
    Ok(records)
}

/// check the each `MS3` indicator and byte length
/// only check the first record
pub fn is_vaild_ms3(bytes: &[u8]) -> MS3Result<()> {
    match bytes
        .get(0..3)
        .ok_or("Invalid MS3: To short")
        .map_err(|e| anyhow!(e))?
        .eq(&[0x4d, 0x53, 0x3]) // b"MS",3
    {
        true => Ok(()),
        false => Err(anyhow!("Invalid MS3: Invalid MS3 indicator")),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn init_logger() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Trace)
            .is_test(true)
            .try_init();
    }

    #[test]
    fn test_is_vaild_ms3() {
        init_logger();
        let mut ms3 = b"MS".to_vec();
        ms3.extend([3u8]);
        assert!(is_vaild_ms3(&ms3).is_ok());
        let msthree = b"MS3";
        assert!(is_vaild_ms3(msthree).is_err());
    }

    #[test]
    fn read_ms3() {
        init_logger();
        let bytes = include_bytes!("../tests/data/testdata-3channel-signal.mseed3");
        let ms3 = MS3Volume::from_bytes(bytes.to_vec())
            .map_err(|e| e)
            .unwrap();
        for rcd in ms3 {
            assert_eq!(rcd.data_raw().len(), rcd.data_len() as usize);
            log::info!("record: {:?}", rcd.summary());
            log::info!("record: {:?}", rcd.data());
        }
    }
}
