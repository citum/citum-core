# citum-edtf

A modern parser for the **Extended Date/Time Format** (ISO 8601-2:2019),
implementing EDTF Level 0 and Level 1. Built on
[`winnow`](https://crates.io/crates/winnow) for zero-allocation parsing.

Used by the Citum citation engine for date handling in bibliographic
references, but has no Citum-specific dependencies — usable standalone for
any project that needs EDTF date parsing.

## Usage

`citum-edtf` uses [`winnow`](https://crates.io/crates/winnow)-style parsers
that take `&mut &str` so callers can stream or chain them:

```rust
use citum_edtf::parse;

let mut input = "2024-03-15";
let date = parse(&mut input).unwrap();

let mut input = "2024~";
let approximate = parse(&mut input).unwrap();

let mut input = "2020/2024";
let interval = parse(&mut input).unwrap();

let mut input = "2024-22"; // summer
let season = parse(&mut input).unwrap();
```

Use `parse_date` if you specifically need to parse a date-only EDTF value.

Supported syntax includes negative years, year precision, month/day
precision, approximations (`~`), uncertainty (`?`), unspecified digits (`X`),
intervals, seasons, and decade/century precision.

## Project

Part of [Citum](https://github.com/citum/citum-core), a modern citation
engine in Rust.

## License

Dual-licensed under [MIT](../../LICENSE) or [Apache-2.0](../../LICENSE-APACHE)
at your option.
