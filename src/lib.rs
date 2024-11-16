#![no_std]
#![deny(unsafe_code)]
#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("mseedio crate requires either std or alloc feature to be enabled");

pub use crate::data::DecodedData;
pub use crate::header::DataEncoding;

pub use data::MS3Data;
pub use header::{FieldFlag, MS3Header, MS3Time, SampleRP};
pub use schema::FDSNSchema;

#[macro_use]
mod data;
mod header;
mod schema;
#[macro_use]
mod utils;
mod record;

pub use record::*;
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

use crate::lib::Vec;

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
