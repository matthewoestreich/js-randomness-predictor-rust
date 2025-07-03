use std::sync::Arc;

use z3::{self, ast::Ast};

pub struct V8Predictor {
    pub sequence: Vec<f64>,
    internal_sequence: Vec<f64>,
    is_initialized: bool,
    node_js_major_version: i32,
    context: Arc<z3::Context>,
    context_static: &'static z3::Context,
    solver: z3::Solver<'static>,
    conc_state_0: u64,
    conc_state_1: u64,
    sym_state_0: z3::ast::BV<'static>,
    sym_state_1: z3::ast::BV<'static>,
}

impl V8Predictor {
    pub fn new(node_major_version: i32, seq: Vec<f64>) -> Self {
        let config = z3::Config::new();
        let context = Arc::new(z3::Context::new(&config));

        let ctx_static: &'static z3::Context = {
            let ptr = Arc::as_ptr(&context);
            unsafe { &*ptr }
        };

        let solver = z3::Solver::new(ctx_static);
        let sym_state_0 = z3::ast::BV::new_const(ctx_static, "symState0", 64);
        let sym_state_1 = z3::ast::BV::new_const(ctx_static, "symState1", 64);

        V8Predictor {
            sequence: seq.clone(),
            is_initialized: false,
            node_js_major_version: node_major_version,
            conc_state_0: 0,
            conc_state_1: 0,
            internal_sequence: {
                let mut s = seq.clone();
                s.reverse();
                s
            },
            context,
            context_static: ctx_static,
            solver,
            sym_state_0,
            sym_state_1,
        }
    }

    pub fn predict_next(&mut self) -> f64 {
        self.initialize();
        let v = self.xor_shift_128_plus_concrete();
        return self.to_double(v);
    }

    fn initialize(&mut self) -> bool {
        if self.is_initialized {
            return true;
        }

        for observed in self.internal_sequence.clone() {
            self.xor_shift_128_plus_symbolic();
            self.recover_mantissa_and_add_to_solver(observed);
        }

        if self.solver.check() != z3::SatResult::Sat {
            panic!("UNSAT");
        }

        let model = self.solver.get_model().unwrap();
        self.conc_state_0 = model.eval(&self.sym_state_0, true).unwrap().as_u64().unwrap();
        self.conc_state_1 = model.eval(&self.sym_state_1, true).unwrap().as_u64().unwrap();

        for _ in self.internal_sequence.clone() {
            self.xor_shift_128_plus_concrete();
        }

        self.is_initialized = true;
        return true;
    }

    fn recover_mantissa_and_add_to_solver(&mut self, value: f64) {
        if self.node_js_major_version >= 24 {
            let mantissa = (value * (1u64 << 53) as f64) as u64;
            self.solver.assert(
                &self
                    .sym_state_0
                    .bvlshr(&z3::ast::BV::from_u64(&self.context, 11, 64))
                    ._eq(&z3::ast::BV::from_u64(&self.context, mantissa, 64)),
            );
            return;
        }

        let mantissa = (value + 1.0) as u64 & ((1u64 << 52) - 1);
        self.solver.assert(
            &z3::ast::BV::from_u64(&self.context, mantissa, 64)._eq(
                &self
                    .sym_state_0
                    .bvlshr(&z3::ast::BV::from_u64(&self.context, 11, 64)),
            ),
        );
    }

    fn xor_shift_128_plus_symbolic(&mut self) {
        let ctx = &self.context_static;
        let mut s1 = self.sym_state_0.clone();
        let s0 = self.sym_state_1.clone();
        let shift_left = s1.bvshl(&z3::ast::BV::from_u64(ctx, 23, 64));
        s1 = s1 ^ shift_left;
        let shift_right = s1.bvlshr(&z3::ast::BV::from_u64(ctx, 17, 64));
        s1 = s1 ^ shift_right;
        s1 = s1 ^ s0.clone();
        s1 = s1 ^ s0.bvlshr(&z3::ast::BV::from_u64(ctx, 26, 64));
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
