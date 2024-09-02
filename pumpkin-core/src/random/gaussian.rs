use super::Random;

pub trait GaussianGenerator: Random {
    fn has_next_gaussian(&self) -> bool;

    fn set_has_next_gaussian(&mut self, value: bool);

    fn stored_next_gaussian(&self) -> f64;

    fn set_stored_next_gaussian(&mut self, value: f64);

    fn calculate_gaussian(&mut self) -> f64 {
        if self.has_next_gaussian() {
            self.set_has_next_gaussian(false);
            self.stored_next_gaussian()
        } else {
            loop {
                let d = 2f64 * self.next_f64() - 1f64;
                let e = 2f64 * self.next_f64() - 1f64;
                let f = d * d + e * e;

                if f < 1f64 && f != 0f64 {
                    let g = (-2f64 * f.ln() / f).sqrt();
                    self.set_stored_next_gaussian(e * g);
                    self.set_has_next_gaussian(true);
                    return d * g;
                }
            }
        }
    }
}
