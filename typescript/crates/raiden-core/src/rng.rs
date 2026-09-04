//! A stand-in for `love.math.random`.
//!
//! xoshiro128**: four words of state, no allocation, and the same stream on
//! every machine — which is what lets a test seed the game and assert on what
//! comes out. The distribution only has to be good enough for sparks and
//! spawn jitter.

pub struct Rng {
    s: [u32; 4],
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        // SplitMix64 to fill the state, so a small or zero seed still starts
        // out well mixed. An all-zero state is a fixed point of xoshiro.
        let mut z = seed ^ 0x9e37_79b9_7f4a_7c15;
        let mut s = [0u32; 4];
        for slot in s.iter_mut() {
            z = z.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut x = z;
            x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            *slot = ((x ^ (x >> 31)) as u32) | 1;
        }
        Rng { s }
    }

    fn next_u32(&mut self) -> u32 {
        let result = self.s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = self.s[1] << 9;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(11);
        result
    }

    /// `love.math.random()` — a float in `[0, 1)`.
    pub fn f(&mut self) -> f32 {
        // 24 bits: every value is exactly representable in an f32.
        (self.next_u32() >> 8) as f32 / (1u32 << 24) as f32
    }

    /// A float in `[lo, hi)`.
    pub fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + self.f() * (hi - lo)
    }

    /// `love.math.random(lo, hi)` — an integer in `[lo, hi]`.
    pub fn int(&mut self, lo: i32, hi: i32) -> i32 {
        if hi <= lo {
            return lo;
        }
        let span = (hi - lo + 1) as u32;
        lo + (self.next_u32() % span) as i32
    }

    /// True with probability `p`. `chance(0.0)` is never, `chance(1.0)` always.
    pub fn chance(&mut self, p: f32) -> bool {
        self.f() < p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floats_stay_in_range() {
        let mut r = Rng::new(7);
        for _ in 0..10_000 {
            let v = r.f();
            assert!((0.0..1.0).contains(&v), "{v} out of range");
        }
    }

    #[test]
    fn integers_are_inclusive_on_both_ends() {
        let mut r = Rng::new(11);
        let mut lo = false;
        let mut hi = false;
        for _ in 0..2000 {
            let v = r.int(3, 6);
            assert!((3..=6).contains(&v), "{v} out of range");
            lo |= v == 3;
            hi |= v == 6;
        }
        assert!(lo && hi, "endpoints never came up");
    }

    #[test]
    fn empty_integer_range_is_the_low_end() {
        let mut r = Rng::new(1);
        assert_eq!(r.int(5, 5), 5);
        assert_eq!(r.int(5, 2), 5);
    }

    #[test]
    fn the_same_seed_replays() {
        let a: Vec<f32> = (0..64).map(|_| Rng::new(99).f()).take(1).collect();
        let mut r1 = Rng::new(1234);
        let mut r2 = Rng::new(1234);
        assert_eq!(a.len(), 1);
        for _ in 0..256 {
            assert_eq!(r1.next_u32(), r2.next_u32());
        }
    }

    #[test]
    fn different_seeds_diverge() {
        let mut a = Rng::new(1);
        let mut b = Rng::new(2);
        assert!((0..16).any(|_| a.next_u32() != b.next_u32()));
    }

    #[test]
    fn chance_bounds_are_absolute() {
        let mut r = Rng::new(5);
        for _ in 0..1000 {
            assert!(!r.chance(0.0));
            assert!(r.chance(1.0));
        }
    }
}
