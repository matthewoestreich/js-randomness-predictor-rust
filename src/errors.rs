use std::{
  error::Error,
  fmt::{Display, Formatter, Result},
  sync::{MutexGuard, PoisonError},
};

#[derive(Debug)]
pub enum InitError {
  Unsat,
  MissingModel,
  EvalFailed(&'static str),
  ConvertFailed(&'static str),
}

impl Display for InitError {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    use InitError::*;
    #[allow(clippy::implicit_return)]
    match self {
      Unsat => write!(f, "Solver returned UNSAT"),
      MissingModel => write!(f, "Failed to get model from solver"),
      EvalFailed(field) => write!(f, "Failed to evaluate {field}"),
      ConvertFailed(field) => write!(f, "Failed to convert {field} to u64"),
    }
  }
}

impl Error for InitError {}

#[derive(Debug)]
pub struct PredictionLimitError;

impl Display for PredictionLimitError {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    return write!(
      f,
      "Prediction count exceeded maximum. 'Initial sequence length' + 'number of predictions' cannot exceed 64! See README for more info!\nPlease call 'reset' with a 'new sequence'! eg. `<instance>.reset(new_sequence)``"
    );
  }
}

impl Error for PredictionLimitError {}

impl From<PoisonError<MutexGuard<'_, u8>>> for PredictionLimitError {
  fn from(_: PoisonError<MutexGuard<'_, u8>>) -> Self {
    return PredictionLimitError;
  }
}
