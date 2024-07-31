use anyhow::anyhow;

use crate::{data::DecodedData, lib::Vec};

const FRAME_SIZE: usize = 16;
const DIFF_SIZE: usize = 105;

#[inline]
pub fn is_big_endian() -> bool {
    let num: u32 = 1;
    let bytes = num.to_be_bytes();
    bytes[0] == 1
}

#[inline(always)]
fn sign_extend(value: i32, bits: u32) -> i32 {
    let shift = 32 - bits;
    (value << shift) >> shift
}

#[inline]
// bits needed to represent a value
fn bitwidth_needed(value: i32) -> u8 {
    match value {
        -8..=7 => 4,
        -16..=15 => 5,
        -32..=31 => 6,
        -128..=127 => 8,
        -512..=511 => 10,
        -16384..=16383 => 15,
        -32768..=32767 => 16,
        -536_870_912..=536_870_911 => 30,
        _ => 32,
    }
}

macro_rules! decode_data {
    ($data: expr, $typ: ident, $len: expr) => {
        $data
            .chunks(crate::lib::mem::size_of::<$typ>())
            .map(|b| {
                let mut byte = [0; $len];
                byte.copy_from_slice(b);
                <$typ>::from_le_bytes(byte)
            })
            .collect()
    };
}

impl crate::Steim1Decode for crate::MS3Record {
    fn decode_steim1(&self, swap_flag: bool) -> anyhow::Result<Vec<i32>> {
        let input = self.data_raw();
        let sample_count = self.sample_count() as usize;
        let sid = self.sid();
        let inputlen = input.len();
        let input = input
            .chunks(4)
            .map(|chunk| {
                let mut bytes = [0; 4];
                bytes.copy_from_slice(chunk);
                i32::from_be_bytes(bytes)
            })
            .collect::<Vec<i32>>();
        let mut output = Vec::with_capacity(sample_count);
        if input.is_empty() {
            return Err(anyhow!("Empty input"));
        }
        let max_frames = inputlen / 64;
        let mut frame = [0i32; FRAME_SIZE]; // Frame, 16 x 32-bit quantities = 64 bytes
        let mut diff = [0i32; 60]; // Difference values for a frame, max is 15 x 4 (8-bit samples)
        let mut xn = 0i32; // Reverse integration constant, aka last sample
        let mut output_idx = 0usize;

        if max_frames == 0 {
            return Ok(output);
        }

        for frame_idx in 0..max_frames {
            if output_idx >= sample_count {
                break;
            }

            // Copy frame, each is 16x32-bit quantities = 64 bytes
            frame.copy_from_slice(&input[(frame_idx * FRAME_SIZE)..((frame_idx + 1) * FRAME_SIZE)]);
            let mut diff_idx = 0;
            let start_nibble;

            // Save forward integration constant (X0) and reverse integration constant (Xn)
            // and set the starting nibble index depending on frame.
            if frame_idx == 0 {
                if swap_flag {
                    frame[1] = frame[1].swap_bytes();
                    frame[2] = frame[2].swap_bytes();
                }

                output.push(frame[1]);
                output_idx += 1;
                xn = frame[2];

                start_nibble = 3; // First frame: skip nibbles, X0, and Xn
                log::trace!("Frame {}: X0={}  Xn={}", frame_idx, output[0], xn);
            } else {
                start_nibble = 1; // Subsequent frames: skip nibbles
                log::trace!("Frame {}", frame_idx);
            }

            // Swap 32-bit word containing the nibbles
            if swap_flag {
                frame[0] = frame[0].swap_bytes();
            }

            // Decode each 32-bit word according to nibble
            for widx in start_nibble..FRAME_SIZE {
                let nibble = (frame[0] >> (30 - (2 * widx))) & 0x03;
                let word = frame[widx];

                match nibble {
                    0 => { /* Special flag, no differences */ }
                    1 => {
                        // Four 1-byte differences
                        let bytes = word.to_be_bytes();
                        for b in &bytes {
                            diff[diff_idx] = sign_extend(*b as i32, 8);
                            diff_idx += 1;
                        }
                        log::trace!(
                            "  W{:02}: 01=4x8b  {:?}  {:?}  {:?}  {:?}",
                            widx,
                            diff[diff_idx - 4],
                            diff[diff_idx - 3],
                            diff[diff_idx - 2],
                            diff[diff_idx - 1]
                        );
                    }
                    2 => {
                        // Two 2-byte differences
                        for i in (0..2).rev() {
                            let half_word = if swap_flag {
                                (word >> (16 * (1 - i))) & 0xFFFF
                            } else {
                                (word >> (16 * i)) & 0xFFFF
                            };
                            diff[diff_idx] = sign_extend(half_word as i32, 16);
                            diff_idx += 1;
                        }
                        log::trace!(
                            "  W{:02}: 10=2x16b  {:?}  {:?}",
                            widx,
                            diff[diff_idx - 2],
                            diff[diff_idx - 1]
                        );
                    }
                    3 => {
                        // One 4-byte difference
                        let wd = if swap_flag { word.swap_bytes() } else { word };
                        diff[diff_idx] = sign_extend(wd, 32);
                        diff_idx += 1;
                        log::trace!("  W{:02}: 11=1x32b  {:?}", widx, diff[diff_idx - 1]);
                    }
                    _ => {}
                }
            }

            // Apply differences in this frame to calculate output samples,
            // ignoring first difference for first frame
            for idx in (if frame_idx == 0 { 1 } else { 0 })..diff_idx {
                if output_idx < sample_count {
                    output.push(output[output_idx - 1] + diff[idx]);
                    output_idx += 1;
                }
            }
        }

        // Check data integrity by comparing last sample to Xn (reverse integration constant)
        if output_idx == sample_count && output[(output_idx - 1) as usize] != xn {
            log::warn!(
                "{}: Warning: Data integrity check for Steim1 failed, Last sample={}, Xn={}",
                sid,
                output[(output_idx - 1) as usize],
                xn
            );
        }

        if output_idx != sample_count {
            log::warn!(
                "{}: Warning: Number of samples decompressed doesn't match number in header: {} != {}",
                sid,
                output_idx,
                sample_count
            );
            return Err(anyhow!(
                "Number of samples decompressed doesn't match number in header"
            ));
        }

        Ok(output)
    }
}

impl crate::Steim2Decode for crate::MS3Record {
    fn decode_steim2(&self, swap_flag: bool) -> anyhow::Result<Vec<i32>> {
        let input = self.data_raw();
        let sample_count = self.sample_count();
        let sid = self.sid();
        let inputlen = input.len();
        let input = input
            .chunks(4)
            .map(|chunk| {
                let mut bytes = [0; 4];
                bytes.copy_from_slice(chunk);
                i32::from_be_bytes(bytes)
            })
            .collect::<Vec<i32>>();
        let mut output = Vec::with_capacity(sample_count as usize);
        if input.is_empty() {
            return Err(anyhow!("Empty input"));
        }

        let max_frames = inputlen / 64;
        if max_frames == 0 {
            return Ok(output);
        }

        log::trace!(
            "Decoding {} Steim2 frames, swapflag: {}, sid: {}",
            max_frames,
            swap_flag,
            sid
        );

        let mut frame = [0; FRAME_SIZE];
        let mut diff = [0; DIFF_SIZE];
        let mut xn = 0;
        let mut output_idx = 0;

        for frame_idx in 0..max_frames {
            if output_idx >= sample_count {
                break;
            }

            let start = (frame_idx * 16) as usize;
            let end = start + 16;
            frame.copy_from_slice(&input[start..end]);

            let mut diff_idx = 0;
            let start_nibble: usize;

            if frame_idx == 0 {
                if swap_flag {
                    frame[1] = frame[1].swap_bytes();
                    frame[2] = frame[2].swap_bytes();
                }

                output.push(frame[1]);
                output_idx += 1;
                xn = frame[2];

                start_nibble = 3;

                log::trace!("Frame {}: X0={}  Xn={}", frame_idx, output[0], xn);
            } else {
                start_nibble = 1;
                log::trace!("Frame {}", frame_idx);
            }

            if swap_flag {
                frame[0] = frame[0].swap_bytes();
            }

            for widx in start_nibble..FRAME_SIZE {
                let nibble = (frame[0] >> (30 - (2 * widx))) & 0x03;

                match nibble {
                    0 => {
                        log::trace!("  W{:02}: 00=special", widx);
                    }
                    1 => {
                        let word = frame[widx];
                        let bytes = word.to_be_bytes();
                        for b in bytes.iter().take(4) {
                            diff[diff_idx] = sign_extend(*b as i32, 8);
                            diff_idx += 1;
                        }
                        log::trace!(
                            "  W{:02}: 01=4x8b  {:?}  {:?}  {:?}  {:?}",
                            widx,
                            diff[diff_idx - 4],
                            diff[diff_idx - 3],
                            diff[diff_idx - 2],
                            diff[diff_idx - 1]
                        );
                    }
                    2 => {
                        if swap_flag {
                            frame[widx] = frame[widx].swap_bytes();
                        }
                        let dnib = (frame[widx] >> 30) & 0x03;

                        match dnib {
                            0 => {
                                log::warn!("{}: Impossible Steim2 dnib=00 for nibble=10", sid);
                                return Err(anyhow!("Impossible Steim2 dnib=00 for nibble=10"));
                            }
                            1 => {
                                diff[diff_idx] = sign_extend((frame[widx] & 0x3FFFFFFF) as i32, 30);
                                diff_idx += 1;
                                log::trace!("  W{:02}: 10,01=1x30b  {}", widx, diff[diff_idx - 1]);
                            }
                            2 => {
                                for i in 0..2 {
                                    diff[diff_idx] = sign_extend(
                                        ((frame[widx] >> (15 - i * 15)) & 0x7FFF) as i32,
                                        15,
                                    );
                                    diff_idx += 1;
                                }
                                log::trace!(
                                    "  W{:02}: 10,10=2x15b  {}  {}",
                                    widx,
                                    diff[diff_idx - 2],
                                    diff[diff_idx - 1]
                                );
                            }
                            3 => {
                                for i in 0..3 {
                                    diff[diff_idx] = sign_extend(
                                        ((frame[widx] >> (20 - i * 10)) & 0x3FF) as i32,
                                        10,
                                    );
                                    diff_idx += 1;
                                }
                                log::trace!(
                                    "  W{:02}: 10,11=3x10b  {}  {}  {}",
                                    widx,
                                    diff[diff_idx - 3],
                                    diff[diff_idx - 2],
                                    diff[diff_idx - 1]
                                );
                            }
                            _ => {}
                        }
                    }
                    3 => {
                        if swap_flag {
                            frame[widx] = frame[widx].swap_bytes();
                        }
                        let dnib = (frame[widx] >> 30) & 0x03;

                        match dnib {
                            0 => {
                                for i in 0..5 {
                                    diff[diff_idx] = sign_extend(
                                        ((frame[widx] >> (24 - i * 6)) & 0x3F) as i32,
                                        6,
                                    );
                                    diff_idx += 1;
                                }
                                log::trace!(
                                    "  W{:02}: 11,00=5x6b  {}  {}  {}  {}  {}",
                                    widx,
                                    diff[diff_idx - 5],
                                    diff[diff_idx - 4],
                                    diff[diff_idx - 3],
                                    diff[diff_idx - 2],
                                    diff[diff_idx - 1]
                                );
                            }
                            1 => {
                                for i in 0..6 {
                                    diff[diff_idx] = sign_extend(
                                        ((frame[widx] >> (25 - i * 5)) & 0x1F) as i32,
                                        5,
                                    );
                                    diff_idx += 1;
                                }
                                log::trace!(
                                    "  W{:02}: 11,01=6x5b  {}  {}  {}  {}  {}  {}",
                                    widx,
                                    diff[diff_idx - 6],
                                    diff[diff_idx - 5],
                                    diff[diff_idx - 4],
                                    diff[diff_idx - 3],
                                    diff[diff_idx - 2],
                                    diff[diff_idx - 1]
                                );
                            }
                            2 => {
                                for i in 0..7 {
                                    diff[diff_idx] = sign_extend(
                                        ((frame[widx] >> (24 - i * 4)) & 0xF) as i32,
                                        4,
                                    );
                                    diff_idx += 1;
                                }
                                log::trace!(
                                    "  W{:02}: 11,10=7x4b  {}  {}  {}  {}  {}  {}  {}",
                                    widx,
                                    diff[diff_idx - 7],
                                    diff[diff_idx - 6],
                                    diff[diff_idx - 5],
                                    diff[diff_idx - 4],
                                    diff[diff_idx - 3],
                                    diff[diff_idx - 2],
                                    diff[diff_idx - 1]
                                );
                            }
                            3 => {
                                log::warn!("{}: Impossible Steim2 dnib=11 for nibble=11", sid);
                                return Err(anyhow!("Impossible Steim2 dnib=11 for nibble=11"));
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }

            for i in if frame_idx == 0 { 1 } else { 0 }..diff_idx {
                if output_idx < sample_count {
                    output.push(output[output_idx as usize - 1] + diff[i]);
                    output_idx += 1;
                }
            }
        }

        if output_idx == sample_count && output[(output_idx - 1) as usize] != xn {
            log::warn!(
                "{}: Warning: Data integrity check for Steim2 failed, Last sample={}, Xn={}",
                sid,
                output[(output_idx - 1) as usize],
                xn
            );
            return Err(anyhow!("Data integrity check for Steim2 failed"));
        }

        if output_idx != sample_count {
            log::warn!(
                "{}: Warning: Number of samples decompressed doesn't match number in header: {} != {}",
                sid,
                output_idx,
                sample_count
            );
            return Err(anyhow!(
                "Number of samples decompressed doesn't match number in header"
            ));
        }

        Ok(output)
    }
}

impl crate::Steim3Decode for crate::MS3Record {
    fn decode_steim3(&self, _swap_flag: bool) -> anyhow::Result<Vec<i32>> {
        unimplemented!()
    }
}

macro_rules! tu8b {
    ($num:expr) => {
        ($num as i8).to_be_bytes()[0]
    };
}

macro_rules! tu16b {
    ($num:expr) => {
        ($num as i16).to_be_bytes()
    };
}

impl crate::Steim1Encode for crate::MS3RecordBuilder {
    fn encode_steim1(&self, diff_0: i32) -> anyhow::Result<Vec<u8>> {
        log::debug!("Encoding Steim1 for record {}", self.header.sid);
        let unencoded = self.data.decoded.take();

        let input = match unencoded {
            DecodedData::I32(data) => data,
            _ => return Err(anyhow!("Decoded data is not I32")),
        };

        let sample_count = input.len();
        let mut output = Vec::with_capacity(sample_count);

        if sample_count == 0 {
            return Ok(output);
        }

        if input.is_empty() {
            return Err(anyhow!("Required input not defined",));
        }

        if input.len() == 1 {
            output.extend_from_slice(&input[0].to_le_bytes());
            return Ok(output);
        }

        let mut idx = 0;
        let mut frm_idx = 0;

        let mut diff = Vec::with_capacity(sample_count);
        diff.push(diff_0);

        for i in 1..input.len() {
            diff.push(input[i] - input[i - 1]);
        }

        let diff_width = diff
            .iter()
            .map(|&x| bitwidth_needed(x))
            .collect::<Vec<u8>>();

        let mut wn = [0; 4];
        let mut w0 = 0u32;
        let mut wcount = 16; // will be reset to 3 in the first frame

        // main loop
        'main: loop {
            if wcount == 16 {
                output.extend_from_slice(&[0, 0, 0, 0]); // temp W0

                if frm_idx == 0 {
                    log::trace!(
                        "Frame {}: X0={}  Xn={}",
                        frm_idx,
                        input[0],
                        input[input.len() - 1]
                    );
                    output.extend_from_slice(&input[0].to_be_bytes());
                    output.extend_from_slice(&input[input.len() - 1].to_be_bytes());
                    wcount = 3;
                } else {
                    log::trace!("Frame {}: ", frm_idx);
                    output[(frm_idx - 1) * 64] = w0.to_be_bytes()[0];
                    output[(frm_idx - 1) * 64 + 1] = w0.to_be_bytes()[1];
                    output[(frm_idx - 1) * 64 + 2] = w0.to_be_bytes()[2];
                    output[(frm_idx - 1) * 64 + 3] = w0.to_be_bytes()[3];
                    w0 = 0;
                    wcount = 1;
                }
                frm_idx += 1;
            }

            // for the tail 0~3 samples
            if idx + 3 >= sample_count {
                let mut left = sample_count - idx;
                'coda: loop {
                    if wcount == 16 {
                        output.extend_from_slice(&[0, 0, 0, 0]); // temp W0

                        if frm_idx == 0 {
                            output.extend_from_slice(&input[0].to_be_bytes());
                            output.extend_from_slice(&input[input.len() - 1].to_be_bytes());
                            wcount = 3;
                            log::trace!(
                                "Frame {}: X0={}  Xn={}",
                                frm_idx,
                                input[0],
                                input[input.len() - 1]
                            );
                        } else {
                            output[(frm_idx - 1) * 64] = w0.to_be_bytes()[0];
                            output[(frm_idx - 1) * 64 + 1] = w0.to_be_bytes()[1];
                            output[(frm_idx - 1) * 64 + 2] = w0.to_be_bytes()[2];
                            output[(frm_idx - 1) * 64 + 3] = w0.to_be_bytes()[3];
                            w0 = 0;
                            wcount = 1;
                            log::trace!("Frame {}: ", frm_idx);
                        }
                        frm_idx += 1;
                    }
                    match left {
                        0 => {
                            // end of coda, fill W0
                            output[(frm_idx - 1) * 64] = w0.to_be_bytes()[0];
                            output[(frm_idx - 1) * 64 + 1] = w0.to_be_bytes()[1];
                            output[(frm_idx - 1) * 64 + 2] = w0.to_be_bytes()[2];
                            output[(frm_idx - 1) * 64 + 3] = w0.to_be_bytes()[3];
                            // fill the rest of the frame with 0
                            while wcount % 16 != 0 {
                                output.extend_from_slice(&[0, 0, 0, 0]);
                                wcount += 1;
                            }
                            break 'coda;
                        }
                        1 => {
                            log::trace!("  W{:02}: 1x32b {} ", wcount, diff[idx]);
                            output.extend_from_slice(&diff[idx].to_be_bytes());
                            idx += 1;
                            w0 = w0 | (3 << (30 - 2 * wcount));

                            left -= 1;
                        }
                        2 => match [diff_width[idx], diff_width[idx + 1]] {
                            [17..=32, _] | [_, 17..=32] => {
                                log::trace!("  W{:02}: 1x32b {} ", wcount, diff[idx]);
                                output.extend_from_slice(&diff[idx].to_be_bytes());
                                idx += 1;
                                w0 = w0 | (3 << (30 - 2 * wcount));
                                left -= 1;
                            }
                            _ => {
                                log::trace!(
                                    "  W{:02}: 2x16b {} {} ",
                                    wcount,
                                    diff[idx],
                                    diff[idx + 1]
                                );
                                output.extend_from_slice(
                                    &[tu16b!(diff[idx]), tu16b!(diff[idx + 1])].concat(),
                                );
                                idx += 2;
                                w0 = w0 | (2 << (30 - 2 * wcount));
                                left -= 2;
                            }
                        },
                        3 => match [diff_width[idx], diff_width[idx + 1], diff_width[idx + 2]] {
                            [17..=32, _, _] | [_, 17..=32, _] => {
                                log::trace!("  W{:02}: 1x32b {} ", wcount, diff[idx]);
                                output.extend_from_slice(&diff[idx].to_be_bytes());
                                idx += 1;
                                w0 = w0 | (3 << (30 - 2 * wcount));
                                left -= 1;
                            }
                            _ => {
                                log::trace!(
                                    "  W{:02}: 2x16b {} {} ",
                                    wcount,
                                    diff[idx],
                                    diff[idx + 1]
                                );
                                output.extend_from_slice(
                                    &[tu16b!(diff[idx]), tu16b!(diff[idx + 1])].concat(),
                                );
                                idx += 2;
                                w0 = w0 | (2 << (30 - 2 * wcount));
                                left -= 2;
                            }
                        },

                        _ => unreachable!(),
                    }
                    wcount += 1;
                }

                // encode done
                break 'main;
            }

            // main part, 4 samples a cycle
            match [
                diff_width[idx],
                diff_width[idx + 1],
                diff_width[idx + 2],
                diff_width[idx + 3],
            ] {
                [0..=8, 0..=8, 0..=8, 0..=8] => {
                    log::trace!(
                        "  W{:02}: 4x8b {} {} {} {} ",
                        wcount,
                        diff[idx],
                        diff[idx + 1],
                        diff[idx + 2],
                        diff[idx + 3]
                    );
                    wn[0] = tu8b!(diff[idx]);
                    wn[1] = tu8b!(diff[idx + 1]);
                    wn[2] = tu8b!(diff[idx + 2]);
                    wn[3] = tu8b!(diff[idx + 3]);
                    output.extend_from_slice(&wn);
                    idx += 4;
                    w0 = w0 | (1 << (30 - 2 * wcount));
                }
                [0..=8, 0..=8, _, _] | [0..=8, 9..=16, _, _] => {
                    log::trace!("  W{:02}: 2x16b {} {} ", wcount, diff[idx], diff[idx + 1]);
                    output.extend_from_slice(&[tu16b!(diff[idx]), tu16b!(diff[idx + 1])].concat());
                    idx += 2;
                    w0 = w0 | (2 << (30 - 2 * wcount));
                }
                [0..=8, 17..=u8::MAX, _, _] => {
                    log::trace!("  W{:02}: 1x32b {} ", wcount, diff[idx]);
                    output.extend_from_slice(&diff[idx].to_be_bytes());
                    idx += 1;
                    w0 = w0 | (3 << (30 - 2 * wcount));
                }
                [9..=16, 0..=8, _, _] | [9..=16, 9..=16, _, _] => {
                    log::trace!("  W{:02}: 2x16b {} {} ", wcount, diff[idx], diff[idx + 1]);
                    output.extend_from_slice(&[tu16b!(diff[idx]), tu16b!(diff[idx + 1])].concat());
                    idx += 2;
                    w0 = w0 | (2 << (30 - 2 * wcount));
                }
                // deal 32b in next loop
                [9..=16, 17..=u8::MAX, _, _] => {
                    log::trace!("  W{:02}: 1x32b {} ", wcount, diff[idx]);
                    output.extend_from_slice(&diff[idx].to_be_bytes());
                    idx += 1;
                    w0 = w0 | (3 << (30 - 2 * wcount));
                }
                [17..=32, _, _, _] => {
                    log::trace!("  W{:02}: 1x32b {} ", wcount, diff[idx]);
                    output.extend_from_slice(&diff[idx].to_be_bytes());
                    idx += 1;
                    w0 = w0 | (3 << (30 - 2 * wcount));
                }
                [33..=u8::MAX, _, _, _] => {
                    unreachable!()
                }
            }

            wcount += 1;
        }

        if idx != sample_count {
            log::warn!(
                "Warning: Number of samples compressed doesn't match number in header: {} != {}",
                idx,
                sample_count
            );
            return Err(anyhow!(
                "Number of samples compressed doesn't match number in header"
            ));
        }

        self.data.decoded.set(DecodedData::I32(input));

        Ok(output)
    }
}

macro_rules! t7x4b {
    ($wcount: expr, $diff: expr, $idx: expr, $w0: expr) => {{
        log::trace!(
            "  W{:02}: 7x4b {} {} {} {} {} {} {}",
            $wcount,
            $diff[$idx],
            $diff[$idx + 1],
            $diff[$idx + 2],
            $diff[$idx + 3],
            $diff[$idx + 4],
            $diff[$idx + 5],
            $diff[$idx + 6]
        );
        let mut wn;
        wn = $diff[$idx + 6] as u32 & 0x0F;
        wn |= ($diff[$idx + 5] as u32 & 0x0F) << 4;
        wn |= ($diff[$idx + 4] as u32 & 0x0F) << 8;
        wn |= ($diff[$idx + 3] as u32 & 0x0F) << 12;
        wn |= ($diff[$idx + 2] as u32 & 0x0F) << 16;
        wn |= ($diff[$idx + 1] as u32 & 0x0F) << 20;
        wn |= ($diff[$idx] as u32 & 0x0F) << 24;
        // dnib 0b10
        wn |= 2 << 30;
        $idx += 7;
        // nibble 0b11
        $w0 = $w0 | (3 << (30 - 2 * $wcount));
        wn
    }};
}

macro_rules! t6x5b {
    ($wcount: expr, $diff: expr, $idx: expr, $w0: expr) => {{
        log::trace!(
            "  W{:02}: 6x5b {} {} {} {} {} {}",
            $wcount,
            $diff[$idx],
            $diff[$idx + 1],
            $diff[$idx + 2],
            $diff[$idx + 3],
            $diff[$idx + 4],
            $diff[$idx + 5]
        );
        let mut wn;
        wn = $diff[$idx + 5] as u32 & 0x1F;
        wn |= ($diff[$idx + 4] as u32 & 0x1F) << 5;
        wn |= ($diff[$idx + 3] as u32 & 0x1F) << 10;
        wn |= ($diff[$idx + 2] as u32 & 0x1F) << 15;
        wn |= ($diff[$idx + 1] as u32 & 0x1F) << 20;
        wn |= ($diff[$idx] as u32 & 0x1F) << 25;
        // dnib 0b01
        wn |= 1 << 30;
        $idx += 6;
        // nibble 0b11
        $w0 = $w0 | (3 << (30 - 2 * $wcount));
        wn
    }};
}

macro_rules! t5x6b {
    ($wcount: expr, $diff: expr, $idx: expr, $w0: expr) => {{
        log::trace!(
            "  W{:02}: 5x6b {} {} {} {} {}",
            $wcount,
            $diff[$idx],
            $diff[$idx + 1],
            $diff[$idx + 2],
            $diff[$idx + 3],
            $diff[$idx + 4]
        );
        let mut wn;

        wn = ($diff[$idx + 4] as u32 & 0x3F);
        wn |= ($diff[$idx + 3] as u32 & 0x3F) << 6;
        wn |= ($diff[$idx + 2] as u32 & 0x3F) << 12;
        wn |= ($diff[$idx + 1] as u32 & 0x3F) << 18;
        wn |= ($diff[$idx] as u32 & 0x3F) << 24;
        // dnib 0b00
        // wn |= 0 << 30; no need to set
        $idx += 5;
        // nibble 0b11
        $w0 = $w0 | (3 << (30 - 2 * $wcount));
        wn
    }};
}

macro_rules! t4x8b {
    ($wcount: expr, $diff: expr, $idx: expr, $w0: expr) => {{
        log::trace!(
            "  W{:02}: 4x8b {} {} {} {}",
            $wcount,
            $diff[$idx],
            $diff[$idx + 1],
            $diff[$idx + 2],
            $diff[$idx + 3]
        );
        let mut wn;
        wn = $diff[$idx + 3] as u32 & 0xFF;
        wn |= ($diff[$idx + 2] as u32 & 0xFF) << 8;
        wn |= ($diff[$idx + 1] as u32 & 0xFF) << 16;
        wn |= ($diff[$idx] as u32 & 0xFF) << 24;
        // dnib not need
        $idx += 4;
        // nibble 0b01
        $w0 = $w0 | (1 << (30 - 2 * $wcount));
        wn
    }};
}

macro_rules! t3x10b {
    ($wcount: expr, $diff: expr, $idx: expr, $w0: expr) => {{
        log::trace!(
            "  W{:02}: 3x10b {} {} {}",
            $wcount,
            $diff[$idx],
            $diff[$idx + 1],
            $diff[$idx + 2]
        );
        let mut wn;
        wn = $diff[$idx + 2] as u32 & 0x3FF;
        wn |= ($diff[$idx + 1] as u32 & 0x3FF) << 10;
        wn |= ($diff[$idx] as u32 & 0x3FF) << 20;
        // dnib 0b11
        wn |= 3 << 30;
        $idx += 3;
        // nibble 0b10
        $w0 = $w0 | (2 << (30 - 2 * $wcount));
        wn
    }};
}

macro_rules! t2x15b {
    ($wcount: expr, $diff: expr, $idx: expr, $w0: expr) => {{
        log::trace!(
            "  W{:02}: 2x15b {} {}",
            $wcount,
            $diff[$idx],
            $diff[$idx + 1]
        );
        let mut wn;
        wn = $diff[$idx + 1] as u32 & 0x7FFF;
        wn |= ($diff[$idx] as u32 & 0x7FFF) << 15;
        // dnib 0b10
        wn |= 2 << 30;

        $idx += 2;
        // nibble 0b10
        $w0 = $w0 | (2 << (30 - 2 * $wcount));
        wn
    }};
}

macro_rules! t1x30b {
    ($wcount: expr, $diff: expr, $idx: expr, $w0: expr) => {{
        log::trace!("  W{:02}: 1x30b {} ", $wcount, $diff[$idx]);
        let mut wn;
        wn = $diff[$idx] as u32 & 0x3FFFFFFF;
        // dnib 0b01
        wn |= 1 << 30;
        $idx += 1;
        // nibble 0b10
        $w0 = $w0 | (2 << (30 - 2 * $wcount));
        wn
    }};
}

impl crate::Steim2Encode for crate::MS3RecordBuilder {
    fn encode_steim2(&self, diff_0: i32) -> anyhow::Result<Vec<u8>> {
        log::debug!("Encoding Steim2 for record {}", self.header.sid);
        let unencoded = self.data.decoded.take();
        let sid = &self.header.sid;

        let input_ne = match unencoded {
            DecodedData::I32(data) => data,
            _ => return Err(anyhow!("Decoded data is not I32")),
        };

        let input = input_ne.iter().map(|&x| x.to_be()).collect::<Vec<i32>>();

        let sample_count = input.len();
        let mut output: Vec<u32> = Vec::with_capacity(sample_count);

        if sample_count == 0 {
            return Ok(Vec::new());
        }

        if input.is_empty() {
            return Err(anyhow!("Required input not defined",));
        }

        if input.len() == 1 {
            return Ok(
                [[0, 0, 0, 0], input[0].to_be_bytes(), input[0].to_be_bytes()]
                    .concat()
                    .to_vec(),
            );
        }

        let mut idx = 0;
        let mut frm_idx = 0;

        let mut diff = Vec::with_capacity(sample_count);
        diff.push(diff_0);

        for i in 1..input_ne.len() {
            diff.push(input_ne[i] - input_ne[i - 1]);
        }

        let diff_width = diff
            .iter()
            .map(|&x| bitwidth_needed(x))
            .collect::<Vec<u8>>();

        let mut w0 = 0u32;
        let mut wcount = 16; // will be reset to 3 in the first frame

        // main loop
        'main: loop {
            if wcount == 16 {
                output.push(0); // temp W0

                if frm_idx == 0 {
                    log::trace!(
                        "Frame {}: X0={}  Xn={}",
                        frm_idx,
                        input_ne[0],
                        input_ne[input_ne.len() - 1]
                    );
                    output.push(input_ne[0].to_le() as u32);
                    output.push(input_ne[input_ne.len() - 1].to_le() as u32);
                    wcount = 3;
                } else {
                    log::trace!("Frame {}: ", frm_idx);
                    output[(frm_idx - 1) * 16] = w0;
                    w0 = 0;
                    wcount = 1;
                }
                frm_idx += 1;
            }

            if idx + 6 >= sample_count {
                // encode done
                break 'main;
            }

            // main part, 7 samples a cycle
            match [
                diff_width[idx],
                diff_width[idx + 1],
                diff_width[idx + 2],
                diff_width[idx + 3],
                diff_width[idx + 4],
                diff_width[idx + 5],
                diff_width[idx + 6],
            ] {
                // 7x4bit
                [0..=4, 0..=4, 0..=4, 0..=4, 0..=4, 0..=4, 0..=4] => {
                    output.push(t7x4b!(wcount, diff, idx, w0));
                }
                // 6x5bit
                [0..=5, 0..=5, 0..=5, 0..=5, 0..=5, 0..=5, _] => {
                    output.push(t6x5b!(wcount, diff, idx, w0));
                }
                // 5x6bit
                [0..=6, 0..=6, 0..=6, 0..=6, 0..=6, _, _] => {
                    output.push(t5x6b!(wcount, diff, idx, w0));
                }
                // 4x8bit
                [0..=8, 0..=8, 0..=8, 0..=8, _, _, _] => {
                    output.push(t4x8b!(wcount, diff, idx, w0));
                }
                // 3x10bit
                [0..=10, 0..=10, 0..=10, _, _, _, _] => {
                    output.push(t3x10b!(wcount, diff, idx, w0));
                }
                [0..=15, 0..=15, _, _, _, _, _] => {
                    output.push(t2x15b!(wcount, diff, idx, w0));
                }
                [0..=30, _, _, _, _, _, _] => {
                    output.push(t1x30b!(wcount, diff, idx, w0));
                }
                [31..=u8::MAX, _, _, _, _, _, _] => {
                    log::warn!("{}: Steim2 can not deal with diff > 30bit", sid);
                    return Err(anyhow!("{}: Steim2 can not deal with diff > 30bit", sid));
                }
            }
            // each cycle will generate a Wn
            wcount += 1;
        } // end of main loop

        // for the tail 0~6 samples
        'coda: loop {
            let left = sample_count - idx;
            if wcount == 16 {
                output.push(0); // temp W0

                if frm_idx == 0 {
                    log::trace!(
                        "Frame {}: X0={}  Xn={}",
                        frm_idx,
                        input[0],
                        input[input.len() - 1]
                    );
                    output.push(input[0] as u32);
                    output.push(input[input.len() - 1] as u32);
                    wcount = 3;
                } else {
                    output[(frm_idx - 1) * 16] = w0;
                    w0 = 0;
                    wcount = 1;
                    log::trace!("Frame {}: ", frm_idx);
                }
                frm_idx += 1;
            }
            match left {
                0 => {
                    // end of coda, fill W0
                    output[(frm_idx - 1) * 16] = w0;
                    // fill the rest of the frame with 0
                    while wcount % 16 != 0 {
                        output.push(0);
                        wcount += 1;
                    }
                    break 'coda;
                }
                1 => {
                    if diff_width[idx] > 30 {
                        log::warn!("{}: Steim2 can not deal with diff > 30bit", sid);
                        return Err(anyhow!("Steim2 can not deal with diff > 30bit"));
                    }
                    output.push(t1x30b!(wcount, diff, idx, w0));
                }
                2 => match [diff_width[idx], diff_width[idx + 1]] {
                    [0..=15, 0..=15] => {
                        output.push(t2x15b!(wcount, diff, idx, w0));
                    }
                    [16..=30, _] | [0..=30, 16..=u8::MAX] => {
                        output.push(t1x30b!(wcount, diff, idx, w0));
                    }
                    [31..=u8::MAX, _] => {
                        log::warn!("{}: Steim2 can not deal with diff > 30bit", sid);
                        return Err(anyhow!("{}: Steim2 can not deal with diff > 30bit", sid));
                    }
                },
                3 => match [diff_width[idx], diff_width[idx + 1], diff_width[idx + 2]] {
                    [0..=10, 0..=10, 0..=10] => {
                        output.push(t3x10b!(wcount, diff, idx, w0));
                    }
                    [0..=15, 0..=15, _] => {
                        output.push(t2x15b!(wcount, diff, idx, w0));
                    }
                    [16..=30, _, _] | [0..=30, 16..=u8::MAX, _] => {
                        output.push(t1x30b!(wcount, diff, idx, w0));
                    }
                    [31..=u8::MAX, _, _] => {
                        log::warn!("{}: Steim2 can not deal with diff > 30bit", sid);
                        return Err(anyhow!("{}: Steim2 can not deal with diff > 30bit", sid));
                    }
                },
                4 => match [
                    diff_width[idx],
                    diff_width[idx + 1],
                    diff_width[idx + 2],
                    diff_width[idx + 3],
                ] {
                    [0..=8, 0..=8, 0..=8, 0..=8] => {
                        output.push(t4x8b!(wcount, diff, idx, w0));
                    }
                    [0..=10, 0..=10, 0..=10, _] => {
                        output.push(t3x10b!(wcount, diff, idx, w0));
                    }
                    [0..=15, 0..=15, _, _] => {
                        output.push(t2x15b!(wcount, diff, idx, w0));
                    }
                    [16..=30, _, _, _] | [0..=30, 16..=u8::MAX, _, _] => {
                        output.push(t1x30b!(wcount, diff, idx, w0));
                    }
                    [31..=u8::MAX, _, _, _] => {
                        log::warn!("{}: Steim2 can not deal with diff > 30bit", sid);
                        return Err(anyhow!("{}: Steim2 can not deal with diff > 30bit", sid));
                    }
                },
                5 => match [
                    diff_width[idx],
                    diff_width[idx + 1],
                    diff_width[idx + 2],
                    diff_width[idx + 3],
                    diff_width[idx + 4],
                ] {
                    [0..=6, 0..=6, 0..=6, 0..=6, 0..=6] => {
                        output.push(t5x6b!(wcount, diff, idx, w0));
                    }
                    [0..=8, 0..=8, 0..=8, 0..=8, _] => {
                        output.push(t4x8b!(wcount, diff, idx, w0));
                    }
                    [0..=10, 0..=10, 0..=10, _, _] => {
                        output.push(t3x10b!(wcount, diff, idx, w0));
                    }
                    [0..=15, 0..=15, _, _, _] => {
                        output.push(t2x15b!(wcount, diff, idx, w0));
                    }
                    [16..=30, _, _, _, _] | [0..=30, 16..=u8::MAX, _, _, _] => {
                        output.push(t1x30b!(wcount, diff, idx, w0));
                    }
                    [31..=u8::MAX, _, _, _, _] => {
                        log::warn!("{}: Steim2 can not deal with diff > 30bit", sid);
                        return Err(anyhow!("{}: Steim2 can not deal with diff > 30bit", sid));
                    }
                },
                6 => match [
                    diff_width[idx],
                    diff_width[idx + 1],
                    diff_width[idx + 2],
                    diff_width[idx + 3],
                    diff_width[idx + 4],
                    diff_width[idx + 5],
                ] {
                    [0..=5, 0..=5, 0..=5, 0..=5, 0..=5, 0..=5] => {
                        output.push(t6x5b!(wcount, diff, idx, w0));
                    }
                    [0..=6, 0..=6, 0..=6, 0..=6, 0..=6, _] => {
                        output.push(t5x6b!(wcount, diff, idx, w0));
                    }
                    [0..=8, 0..=8, 0..=8, 0..=8, _, _] => {
                        output.push(t4x8b!(wcount, diff, idx, w0));
                    }
                    [0..=10, 0..=10, 0..=10, _, _, _] => {
                        output.push(t3x10b!(wcount, diff, idx, w0));
                    }
                    [0..=15, 0..=15, _, _, _, _] => {
                        output.push(t2x15b!(wcount, diff, idx, w0));
                    }
                    [16..=30, _, _, _, _, _] | [0..=30, 16..=u8::MAX, _, _, _, _] => {
                        output.push(t1x30b!(wcount, diff, idx, w0));
                    }
                    [31..=u8::MAX, _, _, _, _, _] => {
                        log::warn!("{}: Steim2 can not deal with diff > 30bit", sid);
                        return Err(anyhow!("{}: Steim2 can not deal with diff > 30bit", sid));
                    }
                },
                _ => unreachable!(),
            }
            wcount += 1;
        }

        if idx != sample_count {
            log::warn!(
                "Warning: Number of samples compressed doesn't match number in header on {}: {} != {}",
                sid,
                idx,
                sample_count
            );
            return Err(anyhow!(
                "Number of samples compressed doesn't match number in header"
            ));
        }

        self.data.decoded.set(DecodedData::I32(input));

        let output = output
            .iter()
            .map(|&x| x.to_be_bytes())
            .collect::<Vec<_>>()
            .concat();

        Ok(output)
    }
}

impl crate::Steim3Encode for crate::MS3RecordBuilder {
    fn encode_steim3(&self, _diff_0: i32) -> anyhow::Result<Vec<u8>> {
        unimplemented!()
    }
}
