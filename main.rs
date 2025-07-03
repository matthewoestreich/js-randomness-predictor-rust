mod v8predictor;

fn main() {
    let mut v8p = v8predictor::V8Predictor::new(
        24,
        vec![
            0.01800425609760259,
            0.19267361208155598,
            0.9892770985784053,
            0.49553307275603264,
            0.7362624704291061,
        ],
    );
    let next_expected = 0.8664993194151147;

    let next = v8p.predict_next();
    println!("next({}) == expected({}) ? {}", next, next_expected, next == next_expected);
}
