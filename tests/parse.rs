use mseedio::*;

mod payloads;

fn init_logger() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .is_test(true)
        .try_init();
}

#[test]
fn read_text_ms3() {
    init_logger();
    let bytes = include_bytes!("../tests/data/reference-text.mseed3");
    let mut ms3 = MS3Volume::from_bytes(bytes.to_vec())
        .map_err(|e| e)
        .unwrap();
    let rcd = ms3.next().unwrap();
    match rcd.data().unwrap() {
        DecodedData::Text(data) => {
            assert_eq!(data, "I've seen things you people wouldn't believe. Attack ships on fire off the shoulder of Orion. I watched C-beams glitter in the dark near the Tannhäuser Gate. All those moments will be lost in time, like tears...in...rain. Time to die.");
        }
        _ => {
            unreachable!("Unexpect data type");
        }
    }
}

#[test]
fn read_int16_ms3() {
    init_logger();
    let payload = payloads::INT16_PAYLOAD;
    let bytes = include_bytes!("../tests/data/reference-sinusoid-int16.mseed3");
    let mut ms3 = MS3Volume::from_bytes(bytes.to_vec())
        .map_err(|e| e)
        .unwrap();
    let rcd = ms3.next().unwrap();
    match rcd.data().unwrap() {
        DecodedData::I16(data) => {
            assert_eq!(data, payload);
        }
        _ => {
            unreachable!("Unexpect data type");
        }
    }
}

#[test]
fn read_int32_ms3() {
    init_logger();
    let payload = payloads::INT32_PAYLOAD;
    let bytes = include_bytes!("../tests/data/reference-sinusoid-int32.mseed3");
    let mut ms3 = MS3Volume::from_bytes(bytes.to_vec())
        .map_err(|e| e)
        .unwrap();
    let rcd = ms3.next().unwrap();
    match rcd.data().unwrap() {
        DecodedData::I32(data) => {
            assert_eq!(data, payload);
        }
        _ => {
            unreachable!("Unexpect data type");
        }
    }
}

#[test]
fn read_f32_ms3() {
    init_logger();
    let payload = payloads::F32_PAYLOAD;
    let bytes = include_bytes!("../tests/data/reference-sinusoid-float32.mseed3");
    let mut ms3 = MS3Volume::from_bytes(bytes.to_vec())
        .map_err(|e| e)
        .unwrap();
    let rcd = ms3.next().unwrap();
    match rcd.data().unwrap() {
        DecodedData::F32(data) => {
            assert_eq!(data, payload);
        }
        _ => {
            unreachable!("Unexpect data type");
        }
    }
}

#[test]
fn read_f64_ms3() {
    let payload = payloads::F64_PAYLOAD;
    let bytes = include_bytes!("../tests/data/reference-sinusoid-float64.mseed3");
    let mut ms3 = MS3Volume::from_bytes(bytes.to_vec())
        .map_err(|e| e)
        .unwrap();
    let rcd = ms3.next().unwrap();
    match rcd.data().unwrap() {
        DecodedData::F64(data) => {
            assert_eq!(data, payload);
        }
        _ => {
            unreachable!("Unexpect data type");
        }
    }
}

#[test]
fn read_steim1_ms3() {
    init_logger();
    let payload = payloads::STEIM1_PAYLOAD;
    let bytes = include_bytes!("../tests/data/reference-sinusoid-steim1.mseed3");
    let mut ms3 = MS3Volume::from_bytes(bytes.to_vec())
        .map_err(|e| e)
        .unwrap();
    let rcd = ms3.next().unwrap();
    match rcd.data().unwrap() {
        DecodedData::I32(data) => {
            assert_eq!(data, payload);
        }
        _ => {
            unreachable!("Unexpect data type");
        }
    }
}

#[test]
fn read_steim2_ms3() {
    init_logger();
    let payload = payloads::STEIM2_PAYLOAD;
    let bytes = include_bytes!("../tests/data/reference-sinusoid-steim2.mseed3");
    let mut ms3 = MS3Volume::from_bytes(bytes.to_vec())
        .map_err(|e| e)
        .unwrap();
    let rcd = ms3.next().unwrap();
    match rcd.data().unwrap() {
        DecodedData::I32(data) => {
            assert_eq!(data, payload);
        }
        _ => {
            unreachable!("Unexpect data type");
        }
    }
}
