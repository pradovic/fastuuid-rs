extern crate rand;
extern crate faster_hex;

use rand::Rng;
use std::convert::TryInto;
use std::sync::atomic::{AtomicUsize, Ordering};

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

    pub fn next(&self) -> [u8; 24] {
        let current = self.counter.fetch_add(1, Ordering::SeqCst);
        let mut uuid: [u8; 24] = Default::default();
        uuid[..8].copy_from_slice(&current.to_be_bytes());
        uuid[8..].copy_from_slice(&self.seed[8..]);
        return uuid;
    }

    pub fn hex128(&self) -> String {
        return Generator::hex128_from_bytes(&self.next());
    }

    pub fn hex128_from_bytes(uuid: &[u8; 24]) -> String {
        let mut temp_uuid: [u8; 24] = [0; 24];
        let mut res: [u8; 36] = [0; 36];
        temp_uuid.copy_from_slice(uuid);
        temp_uuid.swap(6, 9);

        // V4
        temp_uuid[6] = (temp_uuid[6] & 0x0f) | 0x40;
        // RFC4122
        temp_uuid[8] = temp_uuid[8] & 0x3f | 0x80;

        faster_hex::hex_encode(&temp_uuid[0..4],  &mut res[0..8]).unwrap();
        res[8] = '-' as u8;
        faster_hex::hex_encode(&temp_uuid[4..6],  &mut res[9..13]).unwrap();
        res[13] = '-' as u8;
        faster_hex::hex_encode(&temp_uuid[6..8],  &mut res[14..18]).unwrap();
        res[18] = '-' as u8;
        faster_hex::hex_encode(&temp_uuid[8..10],  &mut res[19..23]).unwrap();
        res[23] = '-' as u8;
        faster_hex::hex_encode(&temp_uuid[10..16],  &mut res[24..]).unwrap();

        return std::str::from_utf8(&res).unwrap().to_string();
    }

    //  Returns true if provided string is a valid 128-bit UUID
    pub fn is_valid_hex128(uuid: &str) -> bool {
        let uuid_bytes = uuid.as_bytes();
        if uuid.len() != 36
            || uuid_bytes.len() != 36
            || uuid_bytes[8] != '-' as u8
            || uuid_bytes[13] != '-' as u8
            || uuid_bytes[18] != '-' as u8
            || uuid_bytes[23] != '-' as u8
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
        for c in hex.chars() {
            if !('0' <= c && c <= '9' || 'a' <= c && c <= 'f') {
                return false;
            }
        }
        return true;
    }
}

#[cfg(test)]
mod tests {
    use crate::Generator;
    use std::collections::HashMap;

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
        assert!(
            Generator::is_valid_hex128(&generator.hex128()[..]),
            "should be valid hex"
        );
    }

    #[test]
    fn uniqueness() {
        let mut uuids: HashMap<String, bool> = HashMap::new();
        let generator = Generator::new();

        for _ in 0..100000 {
            let next = generator.hex128();
            assert!(!uuids.contains_key(&next), "duplicate found");
            uuids.insert(next, true);
        }
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
