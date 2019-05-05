// Simplified from rand_xorshift crate. Used for producing colorful terminal
// graphics. Not suitable for cryptographic purposes.

use lazy_static::lazy_static;
use std::num::Wrapping;
use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

lazy_static! {
    static ref RNG: Mutex<Rng> = Mutex::new(Rng::new(seed()));
}

pub fn get() -> MutexGuard<'static, Rng> {
    RNG.lock().unwrap()
}

fn seed() -> u32 {
    let start = SystemTime::now();
    let since_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards");
    since_epoch.subsec_nanos()
}

pub struct Rng {
    x: Wrapping<u32>,
    y: Wrapping<u32>,
    z: Wrapping<u32>,
    w: Wrapping<u32>,
}

impl Rng {
    fn new(seed: u32) -> Self {
        Rng {
            x: Wrapping(seed),
            y: Wrapping(seed),
            z: Wrapping(seed),
            w: Wrapping(seed),
        }
    }

    pub fn u32(&mut self) -> u32 {
        let x = self.x;
        let t = x ^ (x << 11);
        self.x = self.y;
        self.y = self.z;
        self.z = self.w;
        let w_ = self.w;
        self.w = w_ ^ (w_ >> 19) ^ (t ^ (t >> 8));
        self.w.0
    }
}
