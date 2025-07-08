use crate::errors::InitError;
use z3::{self, Config, Context, SatResult, Solver, ast::*};

pub struct ChromePredictor {
  sequence: Vec<f64>,
  internal_sequence: Vec<f64>,
  is_solved: bool,
  conc_state_0: u64,
  conc_state_1: u64,
}

impl ChromePredictor {
  const SS_0_STR: &str = "sym_state_0";
  const SS_1_STR: &str = "sym_state_1";

  pub fn new(seq: Vec<f64>) -> Self {
    let mut iseq = seq.clone();
    iseq.reverse();

    ChromePredictor {
      internal_sequence: iseq,
      sequence: seq,
      is_solved: false,
      conc_state_0: 0,
      conc_state_1: 0,
    }
  }

  #[allow(dead_code)]
  pub fn sequence(&self) -> &[f64] {
    return &self.sequence;
  }

  pub fn predict_next(&mut self) -> Result<f64, InitError> {
    self.solve_symbolic_state()?; // if solving fails, error is returned early
    let v = self.xor_shift_128_plus_concrete();
    Ok(self.to_double(v))
  }

  // Performs XORShift in reverse.
  fn xor_shift_128_plus_concrete(&mut self) -> u64 {
    let result = self.conc_state_0;
    let temp1 = self.conc_state_0;
    let mut temp0 = self.conc_state_1 ^ (self.conc_state_0 >> 26);
    temp0 = temp0 ^ self.conc_state_0;
    temp0 = temp0 ^ (temp0 >> 17) ^ (temp0 >> 34) ^ (temp0 >> 51);
    temp0 = temp0 ^ (temp0 << 23) ^ (temp0 << 46);
    self.conc_state_0 = temp0;
    self.conc_state_1 = temp1;
    return result;
  }

  fn to_double(&self, value: u64) -> f64 {
    return (value >> 11) as f64 / (1u64 << 53) as f64;
  }

  fn solve_symbolic_state(&mut self) -> Result<(), InitError> {
    if self.is_solved {
      return Ok(());
    }

    let config = Config::new();
    let context = Context::new(&config);
    let solver = Solver::new(&context);

    let mut sym_state_0 = z3::ast::BV::new_const(&context, Self::SS_0_STR, 64);
    let mut sym_state_1 = z3::ast::BV::new_const(&context, Self::SS_1_STR, 64);

    for &observed in &self.internal_sequence {
      Self::xor_shift_128_plus_symbolic(&context, &mut sym_state_0, &mut sym_state_1);
      Self::constrain_mantissa(observed, &context, &solver, &sym_state_0);
    }

    if solver.check() != SatResult::Sat {
      return Err(InitError::Unsat);
    }

    let model = solver.get_model().ok_or(InitError::MissingModel)?;

    self.conc_state_0 = model
      .eval(&sym_state_0, true)
      .ok_or(InitError::EvalFailed(Self::SS_0_STR))?
      .as_u64()
      .ok_or(InitError::ConvertFailed(Self::SS_0_STR))?;

    self.conc_state_1 = model
      .eval(&sym_state_1, true)
      .ok_or(InitError::EvalFailed(Self::SS_1_STR))?
      .as_u64()
      .ok_or(InitError::ConvertFailed(Self::SS_1_STR))?;

    for _ in 0..self.internal_sequence.len() {
      self.xor_shift_128_plus_concrete();
    }

    self.is_solved = true;
    return Ok(());
  }

  // Static 'helper' method
  fn xor_shift_128_plus_symbolic<'a>(
    context: &'a Context,
    state_0: &mut BV<'a>,
    state_1: &mut BV<'a>,
  ) {
    let state_0_shifted_left = state_0.bvshl(&BV::from_u64(context, 23, 64));
    let mut s1 = &*state_0 ^ state_0_shifted_left;
    let s1_shifted_right = s1.bvlshr(&BV::from_u64(context, 17, 64));

    s1 = s1 ^ s1_shifted_right;
    s1 = s1 ^ state_1.clone();
    s1 = s1 ^ state_1.bvlshr(&BV::from_u64(context, 26, 64));

    std::mem::swap(state_0, state_1);
    *state_1 = s1;
  }

  // Static 'helper' method
  fn constrain_mantissa(value: f64, context: &Context, solver: &Solver, state_0: &BV) {
    // Recover mantissa
    let mantissa = (value * (1u64 << 53) as f64) as u64;
    // Add mantissa constraint
    solver.assert(
      &state_0
        .bvlshr(&BV::from_u64(context, 11, 64))
        ._eq(&BV::from_u64(context, mantissa, 64)),
    );
  }
}

#[cfg(test)]
mod tests {
  use std::error::Error;
  #[test]
  fn correctly_predicts_sequence() -> Result<(), Box<dyn Error>> {
    let sequence = vec![
      0.32096095967729477,
      0.3940071672626849,
      0.3363374923027722,
      0.7518761096243554,
      0.44201420586496387,
    ];
    let expected = vec![
      0.8199006769436774,
      0.6250240806313154,
      0.9101975676132608,
      0.5889203398264599,
      0.5571161440436232,
      0.9619184649129092,
      0.8385620929536599,
      0.3822042053588621,
      0.5040552869863579,
      0.12014019399083042,
      0.44332968383610927,
      0.37830079319230936,
      0.542449069899975,
      0.0659240460476268,
      0.9589494984837686,
      0.007621633090565627,
      0.14119301022498787,
      0.9964718645470699,
      0.14527130036353442,
      0.6260597083849548,
      0.86354903522581,
      0.7245123107811886,
      0.6565323828155891,
      0.3636039851663503,
      0.5799453712253447,
    ];

    let mut cp = crate::ChromePredictor::new(sequence);
    let mut predictions = vec![];

    for _ in 0..expected.len() {
      let prediction = cp.predict_next()?;
      predictions.push(prediction);
    }

    assert_eq!(predictions, expected);
    return Ok(());
  }
}
