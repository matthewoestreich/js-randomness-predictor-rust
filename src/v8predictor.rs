use crate::errors::InitError;
use z3::ast::*;
use z3::{self, Context, SatResult, Solver};

pub struct V8Predictor<'a> {
    original_sequence: Vec<f64>,
    internal_sequence: Vec<f64>,
    is_initialized: bool,
    node_js_major_version: i32,
    context: &'a Context,
    solver: Solver<'a>,
    conc_state_0: u64,
    conc_state_1: u64,
    sym_state_0: BV<'a>,
    sym_state_1: BV<'a>,
}

impl<'a> V8Predictor<'a> {
    pub fn new(context: &'a Context, nodejs_major_version: i32, seq: Vec<f64>) -> Self {
        let solver = Solver::new(context);
        let sym_state_0 = BV::new_const(context, "symState0", 64);
        let sym_state_1 = BV::new_const(context, "symState1", 64);

        V8Predictor {
            internal_sequence: {
                let mut s = seq.clone();
                s.reverse();
                s
            },
            original_sequence: seq,
            is_initialized: false,
            node_js_major_version: nodejs_major_version,
            conc_state_0: 0,
            conc_state_1: 0,
            context,
            solver,
            sym_state_0,
            sym_state_1,
        }
    }

    #[allow(dead_code)]
    pub fn sequence(&self) -> &[f64] {
        return &self.original_sequence;
    }

    pub fn predict_next(&mut self) -> Result<f64, InitError> {
        self.initialize()?; // if initialize fails, error is returned early
        let v = self.xor_shift_128_plus_concrete();
        Ok(self.to_double(v))
    }

    fn initialize(&mut self) -> Result<(), InitError> {
        if self.is_initialized {
            return Ok(());
        }

        for observed in self.internal_sequence.clone() {
            self.xor_shift_128_plus_symbolic();
            self.recover_mantissa_and_add_to_solver(observed);
        }

        if self.solver.check() != SatResult::Sat {
            return Err(InitError::Unsat);
        }

        let model = self.solver.get_model().ok_or(InitError::MissingModel)?;

        self.conc_state_0 = model
            .eval(&self.sym_state_0, true)
            .ok_or(InitError::EvalFailed("sym_state_0"))?
            .as_u64()
            .ok_or(InitError::ConvertFailed("sym_state_0"))?;

        self.conc_state_1 = model
            .eval(&self.sym_state_1, true)
            .ok_or(InitError::EvalFailed("sym_state_1"))?
            .as_u64()
            .ok_or(InitError::ConvertFailed("sym_state_1"))?;

        for _ in self.internal_sequence.clone() {
            self.xor_shift_128_plus_concrete();
        }

        self.is_initialized = true;
        return Ok(());
    }

    fn recover_mantissa_and_add_to_solver(&mut self, value: f64) {
        if self.node_js_major_version >= 24 {
            let mantissa = (value * (1u64 << 53) as f64) as u64;
            self.solver.assert(
                &self
                    .sym_state_0
                    .bvlshr(&BV::from_u64(&self.context, 11, 64))
                    ._eq(&BV::from_u64(&self.context, mantissa, 64)),
            );
        } else {
            let mantissa = (value + 1.0) as u64 & ((1u64 << 52) - 1);
            self.solver.assert(
                &BV::from_u64(&self.context, mantissa, 64)._eq(
                    &self
                        .sym_state_0
                        .bvlshr(&BV::from_u64(&self.context, 11, 64)),
                ),
            );
        }
    }

    fn xor_shift_128_plus_symbolic(&mut self) {
        let ctx = &self.context;
        let mut s1 = self.sym_state_0.clone();
        let s0 = self.sym_state_1.clone();
        let shifted_left = s1.bvshl(&BV::from_u64(ctx, 23, 64));
        s1 = s1 ^ shifted_left;
        let shifted_right = s1.bvlshr(&BV::from_u64(ctx, 17, 64));
        s1 = s1 ^ shifted_right;
        s1 = s1 ^ s0.clone();
        s1 = s1 ^ s0.bvlshr(&BV::from_u64(ctx, 26, 64));
        self.sym_state_0 = self.sym_state_1.clone();
        self.sym_state_1 = s1;
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

    fn to_double(&mut self, value: u64) -> f64 {
        if self.node_js_major_version >= 24 {
            return (value >> 11) as f64 / (1u64 << 53) as f64;
        }
        return f64::from_bits((value >> 12) | 0x3FF0000000000000) - 1.0;
    }
}
