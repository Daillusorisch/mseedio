use mseedio::*;

mod payloads;

fn init_logger() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .is_test(true)
        .try_init();
}

/// "SID": "FDSN:XX_TEST__L_H_Z",
/// "RecordLength": 1595,
/// "FormatVersion": 3,
/// "Flags": {
///     "RawUInt8": 4,
///     "ClockLocked": true
/// },
/// "StartTime": "2022-06-05T20:32:38.123456789Z",
// "EncodingFormat": 10,
/// "SampleRate": 1.0,
/// "SampleCount": 500,
/// "CRC": "0xEFB85A60",
/// "PublicationVersion": 1,
/// "ExtraLength": 0,
/// "DataLength": 1536,
///
/// should produce exactly the same bytes as reference-sinusoid-steim1.mseed3
#[test]
fn encode_steim1() {
    init_logger();
    let payload = payloads::STEIM1_PAYLOAD;
    let bytes = include_bytes!("../tests/data/reference-sinusoid-steim1.mseed3");
    let rcd = MS3RecordBuilder::new()
        .data_payload_encoding(DataEncoding::Steim1)
        .data(DecodedData::I32(payload.to_vec()))
        .sample_rate(1.0)
        .start_time(MS3Time::from_parts(2022, 6, 5, 20, 32, 38, 123456789)) // "2022-06-05T20:32:38.123456789Z"
        .flag(FieldFlag::ClockLocked)
        .data_public_version(1)
        .sid("FDSN:XX_TEST__L_H_Z")
        .unwrap()
        .build()
        .unwrap();

    assert_eq!(rcd.to_bytes().unwrap(), bytes);
}

#[test]
fn encode_steim2() {
    init_logger();
    let payload = payloads::STEIM2_PAYLOAD;
    let rcd = MS3RecordBuilder::new()
        .data_payload_encoding(DataEncoding::Steim2)
        .data(DecodedData::I32(payload.to_vec()))
        .sample_rate(5.0)
        .start_time(MS3Time::from_parts(2022, 6, 5, 20, 32, 38, 123456789)) // "2022-06-05T20:32:38.123456789Z"
        .flag(FieldFlag::ClockLocked)
        .data_public_version(1)
        .sid("FDSN:XX_TEST__M_H_Z")
        .unwrap()
        .build()
        .unwrap();
    let bytes = include_bytes!("../tests/data/reference-sinusoid-steim2.mseed3");
    assert_eq!(rcd.to_bytes().unwrap(), bytes);
}

#[cfg(feature = "chrono")]
#[test]
fn chrono() {
    init_logger();
    let payload = payloads::STEIM2_PAYLOAD;
    let rcd = MS3RecordBuilder::new()
        .data_payload_encoding(DataEncoding::Steim2)
        .data(DecodedData::I32(payload.to_vec()))
        .sample_rate(5.0)
        .start_time(chrono::DateTime::parse_from_rfc3339("2022-06-05T20:32:38.123456789Z").unwrap())
        .flag(FieldFlag::ClockLocked)
        .data_public_version(1)
        .sid("FDSN:XX_TEST__M_H_Z")
        .unwrap()
        .build()
        .unwrap();
    let bytes = include_bytes!("../tests/data/reference-sinusoid-steim2.mseed3");
    assert_eq!(rcd.to_bytes().unwrap(), bytes);
}
