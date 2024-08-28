pub mod xoroshiro128;

mod gaussian;

pub trait Random {
    fn split(&mut self) -> Self;

    fn next_splitter(&mut self) -> impl RandomSplitter;

    fn set_seed(&mut self, seed: i64);

    fn next_i32(&mut self) -> i32;

    fn next_bounded_i32(&mut self, bound: i32) -> i32;

    fn next_inbetween_i32(&mut self, min: i32, max: i32) -> i32 {
        self.next_bounded_i32(max - min + 1) + min
    }

    fn next_i64(&mut self) -> i64;

    fn next_bool(&mut self) -> bool;

    fn next_f32(&mut self) -> f32;

    fn next_f64(&mut self) -> f64;

    fn next_gaussian(&mut self) -> f64;

    fn next_triangular(&mut self, mode: f64, deviation: f64) -> f64 {
        mode + deviation * (self.next_f64() - self.next_f64())
    }

    fn skip(&mut self, count: i32) {
        for _ in 0..count {
            self.next_i32();
        }
    }

    fn next_inbetween_i32_exclusive(&mut self, min: i32, max: i32) -> i32 {
        min + self.next_bounded_i32(max - min)
    }
}

pub trait RandomSplitter {
    fn split_string(&self, seed: &str) -> impl Random;

    fn split_i64(&self, seed: i64) -> impl Random;

    fn split_pos(&self, x: i32, y: i32, z: i32) -> impl Random;
}
