#[derive(Debug)]
pub enum InitError {
  Unsat,
  MissingModel,
  EvalFailed(&'static str),
  ConvertFailed(&'static str),
}

impl std::fmt::Display for InitError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

impl std::error::Error for InitError {}
