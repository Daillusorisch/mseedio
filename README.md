# mseedio

A lib that can read/write miniseed file written in rust. Support no_std.

> Only minimaleed3 is supported now
> If you want to use miniseed2 in rust, you can use libmseed's rust binding

Supports all encodings of minimaleed3 except steim3.

## usage

* Read

Use MS3Volume to read a miniseed3 file, and get each records's summary using iter.

```rust,ignore
let ms3 = mseedio::MS3Volume::from_file("path/to/your/file.mseed3").unwrap();
for rcd in ms3 {
    println!("{}", rcd.summary());
}
```

`from_file` needs `std` feature, you can replace it with `from_bytes` in no_std environment easily.

* Write

```rust,ignore
let payload: [i32; 500] = [/*...*/];
let rcd = MS3RecordBuilder::new()
        .data_payload_encoding(DataEncoding::Steim1)
        .data(DecodedData::I32(payload.to_vec()))
        .sample_rate(1.0)
        // "2022-06-05T20:32:38.123456789Z"
        .start_time(MS3Time::from_parts(2022, 6, 5, 20, 32, 38, 123456789)) 
        .flag(FieldFlag::ClockLocked)
        .data_public_version(1)
        .sid("FDSN:XX_TEST__L_H_Z")
        .unwrap()
        .build()
        .unwrap();
```

## todo

* [ ] Optimize steim decoding
* [ ] Make encoding and decoding optional features
* [ ] Add miniseed2 support
* [ ] More tests
* [ ] benchmark

### ref

* [SEED Reference Manual 2.4](https://www.google.com/url?client=internal-element-cse&cx=000587591391783241528:dsb0_jo9rjy&q=http://fdsn.adc1.iris.edu/pdf/SEEDManual_V2.4.pdf&sa=U&ved=2ahUKEwjNkqyKm9CHAxUtk1YBHd6hHWkQFnoECAIQAQ&usg=AOvVaw28l6g2_BAgTJubCoWVlu5F).
* [miniseed3 doc](https://docs.fdsn.org/projects/miniseed3/en/latest/index.html)

### license

[MIT](./LICENSE)
