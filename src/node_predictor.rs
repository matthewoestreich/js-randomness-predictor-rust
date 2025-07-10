use crate::{NodeJsMajorVersion, Predictor, errors::*};
use std::{
  error::Error,
  sync::{Arc, Mutex},
};
use z3::{Config, Context, SatResult, Solver, ast::*};

pub struct NodePredictor {
  sequence: Vec<f64>,
  internal_sequence: Vec<f64>,
  is_solved: bool,
  node_js_major_version: NodeJsMajorVersion,
  conc_state_0: u64,
  conc_state_1: u64,
  num_predictions_made: Arc<Mutex<u8>>,
}

impl Predictor for NodePredictor {
  fn predict_next(&mut self) -> Result<f64, Box<dyn Error>> {
    self.increment_prediction_count()?;
    self.solve_symbolic_state()?;
    let v = self.xor_shift_128_plus_concrete();
    let p = self.to_double(v);
    return Ok(p);
  }
}

impl NodePredictor {
  pub const MAX_NUM_PREDICTIONS: u8 = 64;
  const SS_0_STR: &str = "sym_state_0";
  const SS_1_STR: &str = "sym_state_1";

  pub fn new(node_js_major_version: NodeJsMajorVersion, seq: Vec<f64>) -> Self {
    let len = seq.len() as u8;
    let mut iseq = seq.clone();
    iseq.reverse();

    return NodePredictor {
      internal_sequence: iseq,
      sequence: seq,
      node_js_major_version,
      conc_state_0: 0,
      conc_state_1: 0,
      num_predictions_made: Arc::new(Mutex::new(len)),
      is_solved: false,
    };
  }

  #[allow(dead_code)]
  pub fn sequence(&self) -> &[f64] {
    return &self.sequence;
  }

  // So consumers don't have to import the Predictor trait as well as the struct.
  pub fn predict_next(&mut self) -> Result<f64, Box<dyn Error>> {
    return <Self as Predictor>::predict_next(self);
  }

  fn xor_shift_128_plus_concrete(&mut self) -> u64 {
    let result = self.conc_state_0;
    let t1 = self.conc_state_0;
    let mut t0 = self.conc_state_1 ^ (self.conc_state_0 >> 26);
    t0 ^= self.conc_state_0;
    t0 ^= (t0 >> 17) ^ (t0 >> 34) ^ (t0 >> 51);
    t0 ^= (t0 << 23) ^ (t0 << 46);
    self.conc_state_0 = t0;
    self.conc_state_1 = t1;
    return result;
  }

  fn to_double(&self, value: u64) -> f64 {
    if self.node_js_major_version as u8 >= 24 {
      return (value >> 11) as f64 / (1u64 << 53) as f64;
    }
    return f64::from_bits((value >> 12) | 0x3FF0000000000000) - 1.0;
  }

  // If our count is below the max, we can increment, otherwise error.
  fn increment_prediction_count(&self) -> Result<(), PredictionLimitError> {
    let mut c = self.num_predictions_made.lock()?;
    if *c >= Self::MAX_NUM_PREDICTIONS {
      return Err(PredictionLimitError);
    }
    *c += 1;
    return Ok(());
  }

  #[allow(dead_code)]
  fn reset(&mut self, new_sequence: Vec<f64>) -> Result<(), PredictionLimitError> {
    let mut c = self.num_predictions_made.lock()?;
    if *c < Self::MAX_NUM_PREDICTIONS {
      return Ok(());
    }
    *c = new_sequence.len() as u8;
    self.is_solved = false;
    self.internal_sequence = new_sequence.to_vec();
    self.sequence = new_sequence.to_vec();
    self.internal_sequence.reverse();
    return Ok(());
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

    s1 ^= s1_shifted_right;
    s1 ^= state_1.clone();
    s1 ^= state_1.bvlshr(&BV::from_u64(context, 26, 64));

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
  mod general {
    use crate::{NodePredictor, errors::PredictionLimitError};
    use std::error::Error;

    #[test]
    fn reset_after_exhaustion() -> Result<(), Box<dyn Error>> {
      let seq_first = vec![
        0.777225464783239,
        0.15637962909874392,
        0.61479550021439,
        0.613383431187081,
      ];

      let exp_first = vec![
        0.13780690875659396,
        0.9982326337150321,
        0.004547103255256535,
        0.14287124304719512,
        0.07193734860746803,
        0.41988043371402806,
        0.2197922772380051,
        0.3919840116873258,
        0.872346223942074,
        0.8706850288116219,
        0.15113105207209843,
        0.6388396452515654,
        0.49440586365264294,
        0.6587982725994921,
        0.18400263468494316,
        0.662415645160952,
        0.004233542647695265,
        0.7850940676778024,
        0.8718140231245509,
        0.6789540919039344,
        0.3903186400622056,
        0.5518644169835116,
        0.5827729085540138,
        0.5554012760270357,
        0.5233538890694638,
        0.9581085436854987,
        0.49105573307668293,
        0.4887541485622109,
        0.03580260719438155,
        0.7486864084447863,
        0.9442814920321353,
        0.279500250517147,
        0.573892252919875,
        0.35303563579361574,
        0.49663075416404756,
        0.3761838996110659,
        0.01940835807427621,
        0.048560429750311496,
        0.12478054659752413,
        0.8748800514290499,
        0.5585005650941148,
        0.861530489078495,
        0.5288744964943755,
        0.6986980332092166,
        0.25771635223672984,
        0.9727178859177362,
        0.6867934573316927,
        0.6970474592601525,
        0.8035245910646631,
        0.34589316291057026,
        0.16026446047340037,
        0.1871389590142859,
        0.5065543089345518,
        0.13565177330674527,
        0.8171462352178724,
        0.9132684591493374,
        0.3537461024035218,
        0.10449476983306794,
        0.8400598276661568,
        0.6256282841337143,
        0.19469967920827957,
      ];

      let seq_second = vec![
        0.1155167115902066,
        0.2738831377473743,
        0.475867049008157,
        0.24131310081058077,
      ];

      let exp_second = vec![
        0.5567280997370845,
        0.09262950949369997,
        0.9774839147267224,
        0.07372009723227202,
        0.8903569034540151,
        0.2559913027687497,
        0.9357996349973149,
        0.10659667352144908,
        0.34537275726933636,
        0.23697119929732424,
        0.1411756579261214,
        0.4397982843668222,
        0.9628074927171562,
        0.15509374502364615,
      ];

      let seq_first_len = seq_first.len();

      let mut np = NodePredictor::new(crate::NodeJsMajorVersion::V24, seq_first);

      let mut first_predictions = vec![];

      for _ in 0..exp_first.len() {
        match np.predict_next() {
          Ok(prediction) => {
            first_predictions.push(prediction);
          }
          Err(e) => {
            if let Some(_pred_limit_err) = e.downcast_ref::<PredictionLimitError>() {
              np.reset(seq_second)?;
              break;
            } else {
              return Err(e);
            }
          }
        }
      }

      assert_eq!(first_predictions.len() + seq_first_len, 64);

      let mut second_preds = vec![];
      for _ in 0..exp_second.len() {
        let pred = np.predict_next()?;
        second_preds.push(pred);
      }

      assert_eq!(second_preds, exp_second);
      return Ok(());
    }
  }

  mod node_v22 {
    use crate::NodePredictor;
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

      let mut v8p_node_v22 = NodePredictor::new(crate::NodeJsMajorVersion::V22, node_v22_seq);

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
