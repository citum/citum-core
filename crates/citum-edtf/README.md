# citum-edtf

A modern parser for the **Extended Date/Time Format** (ISO 8601-2:2019),
implementing EDTF Level 0 and Level 1. Built on
[`winnow`](https://crates.io/crates/winnow) for zero-allocation parsing.

Used by the Citum citation engine for date handling in bibliographic
references, but has no Citum-specific dependencies — usable standalone for
any project that needs EDTF date parsing.

## Usage

The primary entry point is the `FromStr` impl on `Edtf`:

```rust
use citum_edtf::Edtf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let date: Edtf        = "2024-03-15".parse()?;
    let approximate: Edtf = "2024~".parse()?;
    let interval: Edtf    = "2020/2024".parse()?;
    let season: Edtf      = "2024-22".parse()?;  // summer
    Ok(())
}
```

`FromStr` consumes the entire string and returns a `ParseError` on failure.

For streaming or parser-combinator use cases where you need to consume only a
prefix and leave the rest in place, use the lower-level `parse` and
`parse_date` functions, which follow the
[`winnow`](https://crates.io/crates/winnow) `&mut &str` convention.

Supported syntax includes negative years, year precision, month/day
precision, approximations (`~`), uncertainty (`?`), unspecified digits (`X`),
intervals, seasons, and decade/century precision.

## Project

Part of [Citum](https://github.com/citum/citum-core), a modern citation
engine in Rust.

## License

Dual-licensed under [MIT](../../LICENSE) or [Apache-2.0](../../LICENSE-APACHE)
at your option.
