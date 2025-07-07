use crate::errors::InitError;
use z3::{self, Config, Context, SatResult, Solver, ast::*};

pub struct FirefoxPredictor {
    sequence: Vec<f64>,
    is_solved: bool,
    conc_state_0: u64,
    conc_state_1: u64,
}

impl FirefoxPredictor {
    const SS_0_STR: &str = "sym_state_0";
    const SS_1_STR: &str = "sym_state_1";

    pub fn new(seq: Vec<f64>) -> Self {
        FirefoxPredictor {
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

    fn xor_shift_128_plus_concrete(&mut self) -> u64 {
        let mut s1 = self.conc_state_0;
        let s0 = self.conc_state_1;
        self.conc_state_0 = s0;
        s1 = s1 ^ s1 << 23;
        self.conc_state_1 = s1 ^ s0 ^ (s1 >> 17) ^ (s0 >> 26);
        return self.conc_state_1.wrapping_add(s0);
    }

    fn to_double(&self, value: u64) -> f64 {
        return ((value & 0x1FFFFFFFFFFFFF) as f64) / ((1u64 << 53) as f64);
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

        for &observed in &self.sequence {
            Self::xor_shift_128_plus_symbolic(&context, &mut sym_state_0, &mut sym_state_1);
            Self::constrain_mantissa(observed, &context, &solver, &sym_state_0, &sym_state_1);
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
        context: &Context,
        solver: &Solver,
        state_0: &BV,
        state_1: &BV,
    ) {
        let mantissa = (value * (1u64 << 53) as f64) as u64;
        let symbolic_mask = &BV::from_u64(context, 0x1FFFFFFFFFFFFF, 64);
        solver.assert(
            &BV::from_u64(context, mantissa, 64)._eq(&state_0.bvadd(state_1).bvand(symbolic_mask)),
        );
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    #[test]
    fn correctly_predicts_sequence() -> Result<(), Box<dyn Error>> {
        let sequence = vec![
            0.1321263101773572,
            0.03366887439746058,
            0.032596957696410134,
            0.9986575482138969,
        ];
        let expected = vec![
            0.8479779907956815,
            0.13963871472821332,
            0.25068024611907636,
            0.6656237481612675,
            0.7381091878692425,
            0.8709382509549467,
            0.49171337524788294,
            0.6991749430716799,
            0.9530887478758369,
            0.781511163650037,
            0.699311162730038,
        ];

        let mut ffp = crate::FirefoxPredictor::new(sequence);
        let mut predictions = vec![];

        for _ in 0..expected.len() {
            let prediction = ffp.predict_next()?;
            predictions.push(prediction);
        }

        assert_eq!(predictions, expected);
        return Ok(());
    }
}
