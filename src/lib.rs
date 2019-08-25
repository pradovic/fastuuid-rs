extern crate rand;

use rand::Rng;
use std::convert::TryInto;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::fmt;


pub struct Generator {
    // The constant (random) 192-bit seed.
    // the first 8 bytes are stored in the counter and used for generating new UUIDs
    seed: [u8; 24],
    counter: AtomicUsize,
}

impl Generator {
    fn new() -> Generator {
        let seed = rand::thread_rng().gen::<[u8; 24]>();
        Generator{
            seed,
            counter: AtomicUsize::new(u64::from_le_bytes(seed[0..8].try_into().unwrap()).try_into().unwrap())
        }
    }

    fn next(&self) -> [u8; 24] {
        let current = self.counter.fetch_add(1, Ordering::SeqCst);
        let mut uuid:[u8; 24] = Default::default();
        uuid[..8].copy_from_slice(&current.to_be_bytes());
        uuid[8..].copy_from_slice(&self.seed[8..]);
        return uuid
    }
}

impl fmt::Display for Generator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}\t{}", self.seed, self.counter.load(Ordering::Relaxed));
        Ok(())
    }
}





#[cfg(test)]
mod tests {
    use crate::Generator;

    #[test]
    fn next() {
        let generator = Generator::new();
        let mut first = generator.next();

        for i in 0..10 {
            let second = generator.next();
            assert_eq!(first.len(), second.len(), "Arrays don't have the same length");

            // todo: check better
            first = second;
            println!("{:?}", first)
        }
    }
}