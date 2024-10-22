use super::RandomImpl;

pub trait GaussianGenerator: RandomImpl {
    fn stored_next_gaussian(&self) -> Option<f64>;

    fn set_stored_next_gaussian(&mut self, value: Option<f64>);

    fn calculate_gaussian(&mut self) -> f64 {
        if let Some(gaussian) = self.stored_next_gaussian() {
            self.set_stored_next_gaussian(None);
            gaussian
        } else {
            loop {
                let d = 2f64.mul_add(self.next_f64(), -1f64);
                let e = 2f64.mul_add(self.next_f64(), -1f64);
                let f = d * d + e * e;

                if f < 1f64 && f != 0f64 {
                    let g = (-2f64 * f.ln() / f).sqrt();
                    self.set_stored_next_gaussian(Some(e * g));
                    return d * g;
                }
            }
        }
    }
}
