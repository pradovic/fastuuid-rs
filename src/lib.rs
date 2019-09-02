//!`fastuuid` provides fast UUID generation of guessable and unique 192-bit universally unique identifiers and simple support for 128-bit RFC-4122 V4 UUID.
//!Generated UUIDs are not unguessable as every generated UUID is adjacent to the previously generated UUID.
//!
//!It avoids generating reading 192 bit from rand on each UUID generation, and offers a API to fetch 128-bit string reference,
//!with or without allocating a new heap string object as well, with both safe and unsafe versions of the same function.
//!
//!Benchmarks are included. On my machine generation of a 192-bit UUID takes ~7n, while generating the 128-bit string
//!without and with additional heap allocation (unsafe version) takes <20ns & ~95ns respectively. Safe versions take additional ~10ns.
//!
//!It can be depended on with:
//!
//!```toml
//![dependencies]
//!fastuuid = "0.3.0"
//!```
//!
//!## Examples
//!#### 192-bit UUID
//!```rust
//!use fastuuid::Generator;
//!
//!fn main() {
//!    let generator = Generator::new();
//!    let uuid:[u8;24] = generator.next();
//!}
//!```
//!
//!#### 128-bit UUID
//!```rust
//!// with new string allocation
//!use fastuuid::Generator;
//! fn main() {
//!    let generator = Generator::new();
//!    let uuid = generator.hex128_as_string().unwrap();
//!}
//!```
//!```rust
//! // without new string allocation
//!use fastuuid::Generator;
//!
//!fn main() {
//!    let generator = Generator::new();
//!    let mut buffer: [u8; 36] = [0; 36];
//!    let uuid = generator.hex128_as_str(&mut buffer).unwrap();
//!}
//!```
//!
//!Note: there is also an unsafe version of both functions, which uses unsafe cast to string from utf8, making them a bit faster.
//!It is ok to use all of those concurrently.

extern crate faster_hex;
extern crate rand;

use rand::Rng;
use std::convert::TryInto;
use std::error::Error;
use std::sync::atomic::{AtomicUsize, Ordering};

// Generator is a uuid generator that generates unique and guessable 192-bit UUIDs, starting from a random sequence.
pub struct Generator {
    // The constant (random) 192-bit seed.
    // the first 8 bytes are stored in the counter and used for generating new UUIDs
    seed: [u8; 24],
    counter: AtomicUsize,
}

impl Generator {
    #[allow(dead_code)]
    pub fn new() -> Generator {
        let seed = rand::thread_rng().gen::<[u8; 24]>();
        Generator {
            seed,
            counter: AtomicUsize::new(
                u64::from_le_bytes(seed[0..8].try_into().unwrap())
                    .try_into()
                    .unwrap(),
            ),
        }
    }

    // Next returns the next UUID from the generator.
    // Only the first 8 bytes differ from the previous one.
    // It can be used concurrently.
    pub fn next(&self) -> [u8; 24] {
        let current = self.counter.fetch_add(1, Ordering::SeqCst);
        let mut uuid: [u8; 24] = Default::default();
        uuid[..8].copy_from_slice(&current.to_le_bytes());
        uuid[8..].copy_from_slice(&self.seed[8..]);
        return uuid;
    }

    // hex128_as_str returns hex128(Generator::next()) as &str (without heap allocation of the result)
    pub fn hex128_as_str<'a>(&self, buffer: &'a mut [u8; 36]) -> Result<&'a str, Box<dyn Error>> {
        match std::str::from_utf8(Generator::hex128_from_bytes(&self.next(), buffer)) {
            Ok(res) => Ok(res),
            Err(err) => Err(Box::new(err)),
        }
    }

    // hex128_as_str_unchecked returns hex128(Generator::next()) as &str (without heap allocation of the result)
    // Uses unsafe cast to string from utf8
    pub unsafe fn hex128_as_str_unchecked<'a>(&self, buffer: &'a mut [u8; 36]) -> &'a str {
        std::str::from_utf8_unchecked(Generator::hex128_from_bytes(&self.next(), buffer))
    }

    // hex128_as_string returns hex128(Generator::next()) as boxed String value
    pub unsafe fn hex128_as_string_unchecked(&self) -> String {
        let mut buffer: [u8; 36] = [0; 36];
        std::str::from_utf8_unchecked(Generator::hex128_from_bytes(&self.next(), &mut buffer))
            .to_owned()
    }

    // hex128_as_string returns hex128(Generator::next()) as boxed String value
    pub fn hex128_as_string(&self) -> Result<String, Box<dyn Error>> {
        let mut buffer: [u8; 36] = [0; 36];
        match std::str::from_utf8(Generator::hex128_from_bytes(&self.next(), &mut buffer)) {
            Ok(res) => Ok(res.to_owned()),
            Err(err) => Err(Box::new(err)),
        }
    }

    // Hex128 returns an RFC4122 V4 representation of the
    // first 128 bits of the given UUID, with hyphens.
    //
    // Example: 11febf98-c108-4383-bb1e-739ffcd44341
    //
    // Before encoding, it swaps bytes 6 and 9
    // so that all the varying bits of Generator.next()
    // are reflected in the resulting UUID.
    //
    // Note: If you want unpredictable UUIDs, you might want to consider
    // hashing the uuid (using SHA256, for example) before passing it
    // to Hex128.
    fn hex128_from_bytes<'a>(uuid: &[u8; 24], buffer: &'a mut [u8; 36]) -> &'a [u8] {
        let mut temp_uuid: [u8; 24] = [0; 24];
        temp_uuid.copy_from_slice(uuid);
        temp_uuid.swap(6, 9);

        // V4
        temp_uuid[6] = (temp_uuid[6] & 0x0f) | 0x40;
        // RFC4122
        temp_uuid[8] = temp_uuid[8] & 0x3f | 0x80;

        faster_hex::hex_encode(&temp_uuid[0..16], &mut buffer[0..32]).unwrap();
        buffer.copy_within(20..32, 24); // needs rust stable 1.37.0!!
        buffer.copy_within(16..20, 19);
        buffer.copy_within(12..16, 14);
        buffer.copy_within(8..12, 9);
        buffer[8] = b'-';
        buffer[13] = b'-';
        buffer[18] = b'-';
        buffer[23] = b'-';
        &buffer[..]
    }

    //  Returns true if provided string is a valid 128-bit UUID
    pub fn is_valid_hex128(uuid: &str) -> bool {
        let uuid_bytes = uuid.as_bytes();
        if uuid.len() != 36
            || uuid_bytes.len() != 36
            || uuid_bytes[8] != b'-'
            || uuid_bytes[13] != b'-'
            || uuid_bytes[18] != b'-'
            || uuid_bytes[23] != b'-'
        {
            return false;
        }

        return Generator::valid_hex(&uuid[..8])
            && Generator::valid_hex(&uuid[9..13])
            && Generator::valid_hex(&uuid[14..18])
            && Generator::valid_hex(&uuid[19..23])
            && Generator::valid_hex(&uuid[24..]);
    }

    fn valid_hex(hex: &str) -> bool {
        hex.chars()
            .all(|c| '0' <= c && c <= '9' || 'a' <= c && c <= 'f')
    }
}

#[cfg(test)]
mod tests {
    use crate::Generator;
    use std::thread;
    use std::collections::HashMap;
    use std::sync::{RwLock, Arc};

    #[test]
    fn next() {
        let generator = Generator::new();
        let mut first = generator.next();

        for _ in 0..10 {
            let second = generator.next();
            assert_eq!(
                first.len(),
                second.len(),
                "Arrays don't have the same length"
            );

            first = second;
        }
    }

    #[test]
    fn hex128() {
        let generator = Generator::new();
        let mut buffer: [u8; 36] = [0; 36];

        assert!(
            Generator::is_valid_hex128(&generator.hex128_as_str(&mut buffer).unwrap()[..]),
            "should be valid hex"
        );
    }

    #[test]
    fn uniqueness() {
        let mut uuids: HashMap<String, bool> = HashMap::new();
        let generator = Generator::new();
        let mut buffer: [u8; 36] = [0; 36];

        for _ in 0..100000 {
            let next = generator.hex128_as_str(&mut buffer).unwrap();
            assert!(!uuids.contains_key(&next.to_string()), "duplicate found");
            uuids.insert(next.to_string(), true);
        }
    }

    #[test]
    fn uniqueness_concurrent() {
        let generator = Arc::new(Generator::new());
        let data = Arc::new(RwLock::new(HashMap::new()));
        let threads: Vec<_> = (0..100)
            .map(|_| {
                let data = Arc::clone(&data);
                let generator = generator.clone();
                thread::spawn( move || {
                    let mut map = data.write().unwrap();
                    map.insert(generator.hex128_as_string().unwrap(), true);
                })})
            .collect();

        for t in threads {
            t.join().expect("Thread panicked");
        }


        let map = data.read().unwrap();
        assert_eq!(map.len(), 100, "generated non-unique uuids");
    }

    #[test]
    fn valid_hex() {
        // valid v4 uuid
        assert!(
            Generator::is_valid_hex128("11febf98-c108-4383-bb1e-739ffcd44341"),
            "should be valid hex"
        );

        // invalid uuid
        assert!(
            !Generator::is_valid_hex128("11febf98-c108-4383-bb1e-739ffcd4434"),
            "should be invalid hex"
        );
        assert!(
            !Generator::is_valid_hex128("11febf98-c108-4383-bb1e-739ffcd443412"),
            "should be invalid hex"
        );
        assert!(
            !Generator::is_valid_hex128("11febf98c1-08-4383-bb1e-739ffcd44341"),
            "should be invalid hex"
        );
        assert!(
            !Generator::is_valid_hex128("11febf98-c1084-383-bb1e-739ffcd44341"),
            "should be invalid hex"
        );
        assert!(
            !Generator::is_valid_hex128("11febf98-c108-4383bb-1e-739ffcd44341"),
            "should be invalid hex"
        );
        assert!(
            !Generator::is_valid_hex128("11febf98-c108-4383-bb1e7-39ffcd44341"),
            "should be invalid hex"
        );
    }
}
