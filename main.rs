mod v8predictor;

use v8predictor::*;
use z3;

fn main() {
    let cfg = &z3::Config::new();
    let ctx = &z3::Context::new(cfg);

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

    let mut v8p_node_v24 = V8Predictor::new(ctx, 24, node_v24_seq);

    let mut v8_node_v24_predictions = vec![]; 
    for _ in 0..node_v24_expected.len() {
        v8_node_v24_predictions.push(v8p_node_v24.predict_next());
    }

    let mut is_correct = true;
    for i in 0..v8_node_v24_predictions.len() {
        if v8_node_v24_predictions[i] != node_v24_expected[i] {
            println!("expect prediction '{}' to equal '{}'", v8_node_v24_predictions[i], node_v24_expected[i]);
            is_correct = false;
            break;
        }
    }

    println!("{}", is_correct);

}
