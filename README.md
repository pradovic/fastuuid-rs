# fastuuid

Library `fastuuid` is an oxidized version of https://github.com/rogpeppe/fastuuid, originally written in Go.
It provides fast UUID generation of guessable and unique 192-bit universally unique identifiers and simple support for 128-bit RFC-4122 V4 UUID.
Generated UUIDs are not unguessable as every generated UUID is adjacent to the previously generated UUID.

It avoids generating reading 192 bit from rand on each UUID generation, and offers a API to fetch 128-bit string reference,
with or without allocating a new heap string object as well, with both safe and unsafe versions of the same function.

Benchmarks are included. On my machine generation of a 192-bit UUID takes ~7n, while generating the 128-bit string
without and with additional heap allocation (unsafe version) takes <20ns & ~95ns respectively. Safe versions take additional ~10ns.

## Usage

`fastuuid-rs` can be depended on with:

```toml
[dependencies]
fastuuid = "0.1"
```

## Examples
#### 192-bit UUID
```rust
use fastuuid::Generator;

fn main() {
    let generator = Generator::new();
    let uuid:[u8;24] = generator.next();
}
```

#### 128-bit UUID
- with new string allocation:
```rust
use fastuuid::Generator;

fn main() {
    let generator = Generator::new();
    let uuid = generator.hex128_as_string().unwrap();
}
```
- without new string allocation:
```rust
use fastuuid::Generator;

fn main() {
    let generator = Generator::new();
    let mut buffer: [u8; 36] = [0; 36];
    let uuid = generator.hex128_as_str(&mut buffer).unwrap();
}
```

Note: there is also an unsafe version of both functions, which uses unsafe cast to string from utf8, making them a bit faster.
It is ok to use all of those concurrently.



