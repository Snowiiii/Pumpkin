use std::sync::Arc;

use log::warn;

use super::{DensityFunction, DensityFunctionImpl, UnaryDensityFunction};

#[derive(Clone)]
pub enum LinearType {
    Mul,
    Add,
}

#[derive(Clone)]
pub struct LinearFunction<'a> {
    action: LinearType,
    input: Arc<DensityFunction<'a>>,
    min: f64,
    max: f64,
    arg: f64,
}

impl<'a> DensityFunctionImpl<'a> for LinearFunction<'a> {
    fn apply(&'a self, visitor: &'a impl super::Visitor) -> DensityFunction<'a> {
        let new_function = self.input().apply(visitor);
        let d = new_function.min();
        let e = new_function.max();

        let (f, g) = match self.action {
            LinearType::Add => (d + self.arg, e + self.arg),
            LinearType::Mul => {
                if self.arg >= 0f64 {
                    (d * self.arg, e * self.arg)
                } else {
                    (e * self.arg, d * self.arg)
                }
            }
        };

        DensityFunction::Linear(LinearFunction {
            action: self.action.clone(),
            input: Arc::new(new_function),
            min: f,
            max: g,
            arg: self.arg,
        })
    }

    fn sample(&self, pos: &impl super::NoisePos) -> f64 {
        self.apply_density(self.input().sample(pos))
    }

    fn fill(&self, densities: &[f64], applier: &impl super::Applier) -> Vec<f64> {
        let densities = self.input().fill(densities, applier);
        densities
            .iter()
            .map(|val| self.apply_density(*val))
            .collect()
    }

    fn min(&self) -> f64 {
        self.min
    }

    fn max(&self) -> f64 {
        self.max
    }
}

impl<'a> UnaryDensityFunction<'a> for LinearFunction<'a> {
    fn apply_density(&self, density: f64) -> f64 {
        match self.action {
            LinearType::Mul => density * self.arg,
            LinearType::Add => density + self.arg,
        }
    }

    fn input(&self) -> &DensityFunction {
        &self.input
    }
}

#[derive(Clone)]
pub enum BinaryType {
    Mul,
    Add,
    Min,
    Max,
}

#[derive(Clone)]
pub struct BinaryFunction<'a> {
    action: BinaryType,
    arg1: Arc<DensityFunction<'a>>,
    arg2: Arc<DensityFunction<'a>>,
    min: f64,
    max: f64,
}

impl<'a> BinaryFunction<'a> {
    pub fn create(
        action: BinaryType,
        arg1: DensityFunction<'a>,
        arg2: DensityFunction<'a>,
    ) -> DensityFunction<'a> {
        let d = arg1.min();
        let e = arg2.min();
        let f = arg1.max();
        let g = arg2.max();

        match action {
            BinaryType::Min | BinaryType::Max => {
                if d >= e || e >= f {
                    warn!("Density function does not overlap");
                }
            }
            _ => {}
        }

        let h = match action {
            BinaryType::Add => d + e,
            BinaryType::Mul => {
                if d > 0f64 && e > 0f64 {
                    d * e
                } else if f < 0f64 && g < 0f64 {
                    f * g
                } else {
                    (d * g).min(f * e)
                }
            }
            BinaryType::Min => d.min(e),
            BinaryType::Max => d.max(e),
        };

        let i = match action {
            BinaryType::Add => f + g,
            BinaryType::Mul => {
                if d > 0f64 && e > 0f64 {
                    f * g
                } else if f < 0f64 && g < 0f64 {
                    d * e
                } else {
                    (d * e).max(f * g)
                }
            }
            BinaryType::Min => f.min(g),
            BinaryType::Max => f.max(g),
        };

        match action {
            BinaryType::Mul | BinaryType::Add => {
                let action = match action {
                    BinaryType::Add => LinearType::Add,
                    BinaryType::Mul => LinearType::Mul,
                    _ => unreachable!(),
                };

                if let DensityFunction::Constant(func) = arg1 {
                    return DensityFunction::Linear(LinearFunction {
                        action,
                        input: Arc::new(arg2),
                        min: h,
                        max: i,
                        arg: func.value,
                    });
                }

                if let DensityFunction::Constant(func) = arg2 {
                    return DensityFunction::Linear(LinearFunction {
                        action,
                        input: Arc::new(arg1),
                        min: h,
                        max: i,
                        arg: func.value,
                    });
                }
            }
            _ => {}
        }

        DensityFunction::Binary(BinaryFunction {
            action,
            arg1: Arc::new(arg1),
            arg2: Arc::new(arg2),
            min: h,
            max: i,
        })
    }
}

impl<'a> DensityFunctionImpl<'a> for BinaryFunction<'a> {
    fn sample(&self, pos: &impl super::NoisePos) -> f64 {
        let d = self.arg1.sample(pos);
        let e = self.arg2.sample(pos);

        match self.action {
            BinaryType::Add => d + e,
            BinaryType::Mul => d * e,
            BinaryType::Min => {
                if d < self.arg2.min() {
                    d
                } else {
                    d.min(e)
                }
            }
            BinaryType::Max => {
                if d > self.arg2.max() {
                    d
                } else {
                    d.max(e)
                }
            }
        }
    }

    fn fill(&self, densities: &[f64], applier: &impl super::Applier) -> Vec<f64> {
        let densities1 = self.arg1.fill(densities, applier);
        let densities2 = self.arg2.fill(densities, applier);

        match self.action {
            BinaryType::Add => densities1
                .iter()
                .zip(densities2)
                .map(|(x, y)| x + y)
                .collect(),
            BinaryType::Mul => densities1
                .iter()
                .zip(densities2)
                .map(|(x, y)| x * y)
                .collect(),
            BinaryType::Min => densities1
                .iter()
                .zip(densities2)
                .map(|(x, y)| x.min(y))
                .collect(),
            BinaryType::Max => densities1
                .iter()
                .zip(densities2)
                .map(|(x, y)| x.max(y))
                .collect(),
        }
    }

    fn apply(&'a self, visitor: &'a impl super::Visitor) -> DensityFunction<'a> {
        visitor.apply(&BinaryFunction::create(
            self.action.clone(),
            self.arg1.apply(visitor),
            self.arg2.apply(visitor),
        ))
    }

    fn max(&self) -> f64 {
        self.max
    }

    fn min(&self) -> f64 {
        self.min
    }
}
