//! FDSN Schema for miniSEED 3 data format
//! ref [FDSN Extra Header schema v1.0.](https://raw.githubusercontent.com/FDSN/miniSEED3/main/extra-headers/ExtraHeaders-FDSN-v1.0.schema-2023-07.json)
//! generated using [quicktype](https://app.quicktype.io/)

use crate::lib::{String, Vec};
use serde::{Deserialize, Serialize};

/// Extra headers in miniSEED 3 data format
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct FDSNSchema {
    /// Reserved extra headers defined by the FDSN
    fdsn: Option<Fdsn>,
}

impl FDSNSchema {
    pub fn new() -> Self {
        Self { fdsn: None }
    }
}

/// Reserved extra headers defined by the FDSN
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Fdsn {
    /// Headers related to calibrations
    calibration: Option<Calibration>,

    /// Description of clock system
    clock: Option<Clock>,

    /// Data quality indicator, use D, R, Q or M. [same as SEED 2.4 FSDH, field 2]
    data_quality: Option<String>,

    /// Headers related to event detection and progression
    event: Option<Event>,

    flags: Option<Flags>,

    /// Description of data logger
    logger: Option<Clock>,

    /// An identifier for a provenance description
    #[serde(rename = "ProvenanceURI")]
    provenance_uri: Option<String>,

    /// Headers related to sensor recentering (mass, gimble, etc.)
    recenter: Option<Recenter>,

    /// Description of sensor
    sensor: Option<Clock>,

    /// Data record sequence number. [same as SEED 2.4 FSDH, field 1]
    sequence: Option<i64>,

    /// Headers related to data timing and recording system clock
    time: Option<Time>,
}

/// Headers related to calibrations
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Calibration {
    /// List of calibration sequences
    sequence: Option<Vec<CalibrationSequence>>,
}

/// List of calibrations
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct CalibrationSequence {
    /// Amplitude of calibration signal.  For pseudo-random calibrations, this should be the
    /// peak-to-peak amplitude of the steps. [same as SEED 2.4 Blockettes 300,310, field 8 and
    /// Blockettes 320,390, field 7]
    amplitude: Option<f64>,

    /// Amplitude range of calibration, use values of PEAKTOPEAK, ZEROTOPEAK, RMS OR RANDOM.
    /// [same as SEED 2.4 Blockette 310, field 5, bits 4,5,6 and Blockette 320, field 5, bit 4]
    amplitude_range: Option<String>,

    /// Calibration begin time. [similar to SEED 2.4 Blockettes 300,310,320,390, field 3]
    begin_time: Option<String>,

    /// Calibration is continued from previous records. [same as SEED 2.4 Blockettes
    /// 300,310,320,390, field 5, bit 3]
    continued: Option<bool>,

    /// Coupling of the calibration signal, such as RESISTIVE or CAPACITIVE. [same as SEED 2.4
    /// Blockettes 300,310, field 12 and Blockette 320, field 11]
    coupling: Option<String>,

    /// Duration of calibration in seconds. For step calibrations this is the duration of the
    /// step.  [same as SEED 2.4 Blockettes 300,310,320,390, field 6]
    duration: Option<f64>,

    /// Calibration end time. [same as SEED 2.4 Blockette 395, field 3]
    end_time: Option<String>,

    /// Channel containing the calibration input. [same as SEED 2.4 Blockettes 300,310, field 9
    /// and Blockettes 320,390, field 8]
    input_channel: Option<String>,

    /// Units of calibration input, usually volts or amps. [same as SEED 2.4 Blockette 52, field
    /// 9]
    input_units: Option<String>,

    /// Noise characteristics for pseudo-random calibrations, such as WHITE or RED. [same as SEED
    /// 2.4 Blockette 320, field 13]
    noise: Option<String>,

    /// Reference amplitude.  This is a user-defined value that indicates either the voltage or
    /// amperage of the calibration signal when the calibrator is set to 0dB. [same as SEED 2.4
    /// Blockettes 300,310, field 11 and Blockette 320, field 10]
    reference_amplitude: Option<f64>,

    /// Rolloff characteristics for any filters used on the calibrator, such as 3dB@10Hz.  [same
    /// as SEED 2.4 Blockettes 300,310, field 13 and Blockette 320, field 12]
    rolloff: Option<String>,

    /// Period of sine calibrations in seconds. [same as SEED 2.4 Blockette 310, field 7]
    sine_period: Option<f64>,

    /// Step calibration alternate sign.  [same as SEED 2.4 Blockettes 300, field 5, bit 1]
    step_alternate_sign: Option<bool>,

    /// Interval between times a step calibration is on in seconds. [same as SEED 2.4 Blockette
    /// 300, field 7]
    step_between: Option<f64>,

    /// Step calibration, first pulse is positive. [same as SEED 2.4 Blockette 300, field 5, bit
    /// 0]
    step_first_pulse_positive: Option<bool>,

    /// Number of step calibrations in sequence. [same as SEED 2.4 Blockette 300, field 4]
    steps: Option<f64>,

    /// Calibration trigger, use AUTOMATIC or MANUAL. [same as SEED 2.4 Blockettes
    /// 300,310,320,390, field 5, bit 2]
    trigger: Option<String>,

    /// Type of calibration: STEP, SINE, PSEUDORANDOM, GENERIC.
    #[serde(rename = "Type")]
    sequence_type: Option<String>,
}

/// Description of clock system
///
/// Description of data logger
///
/// Description of sensor
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Clock {
    /// Model of equipment
    model: Option<String>,

    /// Serial number of equipment
    serial: Option<String>,
}

/// Headers related to event detection and progression
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Event {
    /// An event starts in this record. [same as SEED 2.4 FSDH, field 12, bit 2]
    begin: Option<bool>,

    /// List of event detections
    detection: Option<Vec<Detection>>,

    /// An in-progress event ends in this record, i.e. the detection algorithm de-triggers. [same
    /// as SEED 2.4 FSDH, field 12, bit 3]
    end: Option<bool>,

    /// An event is in progress. [same as SEED 2.4 FSDH, field 12, bit 6]
    in_progress: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Detection {
    /// Event background estimate in counts. [same as SEED 2.4 Blockettews 200 and 201, field 5]
    background_estimate: Option<f64>,

    /// Name of the event detector. [same as SEED 2.4 Blockette 200, field 9 and Blockette 201,
    /// field 12]
    detector: Option<String>,

    /// Murdock event detction lookback value: 0, 1 or 2. [same as SEED 2.4 Blockette 201, field
    /// 10]
    #[serde(rename = "MEDLookback")]
    med_lookback: Option<i64>,

    /// Murdock event detection pick algorithm: 0 or 1. [same as SEED 2.4 Blockette 201, field 11]
    #[serde(rename = "MEDPickAlgorithm")]
    med_pick_algorithm: Option<i64>,

    /// Murdock event detection signal-to-noise ratios. [same as SEED 2.4 Blockette 201, field 9]
    #[serde(rename = "MEDSNR")]
    medsnr: Option<Vec<f64>>,

    /// Event signal onset time. [same as SEED 2.4 Blockettews 200 and 201, field 8]
    onset_time: Option<String>,

    /// Event amplitude of signal in counts. [same as SEED 2.4 Blockettews 200 and 201, field 3]
    signal_amplitude: Option<f64>,

    /// Event period of signal in seconds. [same as SEED 2.4 Blockettews 200 and 201, field 4]
    signal_period: Option<f64>,

    /// Type of event detection, e.g. MURDOCK
    #[serde(rename = "Type")]
    detection_type: Option<String>,

    /// Units of amplitude band background estimate, use COUNTS for unprocessed signal. [similar
    /// to SEED 2.4 Blockettews 200, field 6, bit 1]
    units: Option<String>,

    /// Event detection wave, use values of DILATATION or COMPRESSION if determined. [same as
    /// SEED 2.4 Blockettews 200 and 201, field 6, bit 0]
    wave: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Flags {
    /// Amplifier saturation detected. [same as SEED 2.4 FSDH, field 14, bit 0]
    amplifier_saturation: Option<bool>,

    /// Digitizer clipping detected. [same as SEED 2.4 FSDH, field 14, bit 1]
    digitizer_clipping: Option<bool>,

    /// DEPRECATED End of time series. [same as SEED 2.4 FDSH, field 13, bit 4]
    end_of_time_series: Option<bool>,

    /// A digital filter may be charging. [same as SEED 2.4 FDSH, field 14, bit 6]
    filter_charging: Option<bool>,

    /// Glitches detected. [same as SEED 2.4 FSDH, field 14, bit 3]
    glitches: Option<bool>,

    /// DEPRECATED Long record read (possibly no problem). [same as SEED 2.4 FDSH, field 13, bit
    /// 1]
    long_record_read: Option<bool>,

    /// Sensor mass position is offscale as defined by the vendor or operator.
    mass_position_offscale: Option<bool>,

    /// DEPRECATED Missing data. [same as SEED 2.4 FDSH, field 14, bit 4]
    missing_data: Option<bool>,

    /// DEPRECATED Short record read (record padded). [same as SEED 2.4 FDSH, field 13, bit 2]
    short_record_read: Option<bool>,

    /// Spikes detected. [same as SEED 2.4 FSDH, field 14, bit 2]
    spikes: Option<bool>,

    /// DEPRECATED Start of time series. [same as SEED 2.4 FDSH, field 13, bit 3]
    start_of_time_series: Option<bool>,

    /// DEPRECATED Station volume parity error possibly present. [same as SEED 2.4 FDSH, field
    /// 13, bit 0]
    station_volume_parity_error: Option<bool>,

    /// DEPRECATED Telemetry synchronization error. [same as SEED 2.4 FDSH, field 14, bit 5]
    telemetry_sync_error: Option<bool>,
}

/// Headers related to sensor recentering (mass, gimble, etc.)
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Recenter {
    /// List of recentering sequences
    sequence: Option<Vec<RecenterSequence>>,
}

/// List of recenterings
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct RecenterSequence {
    /// Recenter begin time.
    begin_time: Option<String>,

    /// Estimate of recenter end time.
    end_time: Option<String>,

    /// Calibration trigger, use AUTOMATIC or MANUAL.
    trigger: Option<String>,

    /// Type of recenter: Mass, Gimbal, etc.  If omitted a mass recenter may be assumed.
    #[serde(rename = "Type")]
    sequence_type: Option<String>,
}

/// Headers related to data timing and recording system clock
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Time {
    /// Time correction applied to record start time in seconds. [same as SEED 2.4 FSDH, field 16]
    correction: Option<f64>,

    /// List of timing exceptions
    exception: Option<Vec<Exception>>,

    /// If present, one or more leap seconds occuring during this record.  The value specifies
    /// the number of leap seconds and direction.  For example, use 1 to specify a single
    /// positive leap second and -1 to specify a single negative leap second. [incorporates SEED
    /// 2.4 FSDH, field 12, bits 4 and 5]
    leap_second: Option<i64>,

    /// Maximum estimated timing error in seconds.
    max_estimated_error: Option<f64>,

    /// DEPRECATED Timing quality.  A vendor specific value from 0 to 100% of maximum accuracy.
    /// [same as SEED 2.4 Blockette 1001, field 3] It is recommended to use MaxEstimatedError
    /// instead.
    quality: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Exception {
    /// Description of clock status, clock specific parameters such as the station for an Omega
    /// clock or satellite signal to noise ratios for GPS clocks. [same as SEED 2.4 Blockette
    /// 500, field 10]
    clock_status: Option<String>,

    /// Exception count, with meaning based on type of exception, such as 15 missing time marks.
    /// [same as SEED 2.4 Blockette 500, field 7]
    count: Option<i64>,

    /// Reception quality as a percent of maximum clock accuracy based only on information from
    /// the clock. [same as SEED 2.4 Blockette 500, field 6]
    reception_quality: Option<i64>,

    /// Time of timing exeption. [same as SEED 2.4 Blockette 500, field 4]
    time: Option<String>,

    /// Description of clock exception, such as MISSING TIMEMARK. [same as SEED 2.4 Blockette
    /// 500, field 8]
    #[serde(rename = "Type")]
    exception_type: Option<String>,

    /// VCO correction, a percentage from 0 to 100% of VCO control value, where 0 is the slowest
    /// and 100 is the fastest.  [same as SEED 204 Blockette 500, field 3]
    #[serde(rename = "VCOCorrection")]
    vco_correction: Option<f64>,
}
