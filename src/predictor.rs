use std::error::Error;

pub trait Predictor {
  fn predict_next(&mut self) -> Result<f64, Box<dyn Error>>;
}
