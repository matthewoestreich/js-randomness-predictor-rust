use crate::errors::InitError;

pub trait Predictor {
  fn predict_next(&mut self) -> Result<f64, InitError>;
}
