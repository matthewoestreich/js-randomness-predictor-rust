use crate::{NodeJsMajorVersion, errors::InitError};
use z3::{Config, Context, SatResult, Solver, ast::*};

pub struct NodePredictor {
  sequence: Vec<f64>,
  internal_sequence: Vec<f64>,
  is_solved: bool,
  node_js_major_version: NodeJsMajorVersion,
  conc_state_0: u64,
  conc_state_1: u64,
}

impl NodePredictor {
  const SS_0_STR: &str = "sym_state_0";
  const SS_1_STR: &str = "sym_state_1";

  pub fn new(node_js_major_version: NodeJsMajorVersion, seq: Vec<f64>) -> Self {
    let mut iseq = seq.clone();
    iseq.reverse();

    NodePredictor {
      internal_sequence: iseq,
      sequence: seq,
      is_solved: false,
      node_js_major_version,
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
    if self.node_js_major_version as u8 >= 24 {
      return (value >> 11) as f64 / (1u64 << 53) as f64;
    }
    return f64::from_bits((value >> 12) | 0x3FF0000000000000) - 1.0;
  }

  fn solve_symbolic_state(&mut self) -> Result<(), InitError> {
    if self.is_solved {
      return Ok(());
    }

    let config = Config::new();
    let context = Context::new(&config);
    let solver = Solver::new(&context);

    let mut sym_state_0 = BV::new_const(&context, Self::SS_0_STR, 64);
    let mut sym_state_1 = BV::new_const(&context, Self::SS_1_STR, 64);

    for &observed in &self.internal_sequence {
      Self::xor_shift_128_plus_symbolic(&context, &mut sym_state_0, &mut sym_state_1);
      Self::constrain_mantissa(
        observed,
        self.node_js_major_version,
        &context,
        &solver,
        &sym_state_0,
      );
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
  fn constrain_mantissa(
    value: f64,
    nodejs_version: NodeJsMajorVersion,
    context: &Context,
    solver: &Solver,
    state_0: &BV,
  ) {
    if nodejs_version as u8 >= 24 {
      // Recover mantissa
      let mantissa = (value * (1u64 << 53) as f64) as u64;
      // Add mantissa constraint
      solver.assert(
        &state_0
          .bvlshr(&BV::from_u64(context, 11, 64))
          ._eq(&BV::from_u64(context, mantissa, 64)),
      );
    } else {
      // Recover mantissa
      let mantissa = f64::to_bits(value + 1.0) & ((1u64 << 52) - 1);
      // Add mantissa constraint
      solver.assert(
        &BV::from_u64(context, mantissa, 64)._eq(&state_0.bvlshr(&BV::from_u64(context, 12, 64))),
      );
    }
  }
}

#[cfg(test)]
mod tests {
  mod node_v22 {
    use std::error::Error;

    #[test]
    fn correctly_predicts_sequence() -> Result<(), Box<dyn Error>> {
      let node_v22_seq = vec![
        0.36280726230126614,
        0.32726837947512855,
        0.22834780314989023,
        0.18295517908119385,
      ];
      let node_v22_expected = vec![
        0.8853110028441145,
        0.14326940888839124,
        0.035607792006009165,
        0.6491231376351401,
        0.3345277284146617,
        0.42618019812863417,
      ];

      let mut v8p_node_v22 =
        crate::NodePredictor::new(crate::NodeJsMajorVersion::V22, node_v22_seq);

      let mut v8_node_v22_predictions = vec![];
      for _ in 0..node_v22_expected.len() {
        let prediction = v8p_node_v22.predict_next()?;
        v8_node_v22_predictions.push(prediction);
      }

      assert_eq!(v8_node_v22_predictions, node_v22_expected);
      return Ok(());
    }
  }

  mod node_v24 {
    use std::error::Error;

    #[test]
    fn correctly_predicts_sequence() -> Result<(), Box<dyn Error>> {
      let node_v24_seq = vec![
        0.01800425609760259,
        0.19267361208155598,
        0.9892770985784053,
        0.49553307275603264,
        0.7362624704291061,
      ];
      let node_v24_expected = vec![
        0.8664993194151147,
        0.5549329443482626,
        0.8879559862322086,
        0.9570142746667122,
        0.7514661363382521,
        0.9348208735728415,
      ];

      let mut v8p_node_v24 =
        crate::NodePredictor::new(crate::NodeJsMajorVersion::V24, node_v24_seq);

      let mut v8_node_v24_predictions = vec![];
      for _ in 0..node_v24_expected.len() {
        let prediction = v8p_node_v24.predict_next()?;
        v8_node_v24_predictions.push(prediction)
      }

      assert_eq!(v8_node_v24_predictions, node_v24_expected);
      return Ok(());
    }
  }
}
