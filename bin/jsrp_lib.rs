use crate::cmd_line_parser::*;
use js_randomness_predictor::*;
use serde_json::to_string_pretty;
use std::{error::Error, fs, path::Path};

pub fn handle_node(node_args: NodeArgs) -> Result<(), Box<dyn Error>> {
  let SharedArgs {
    mut predictions,
    sequence,
    export,
  } = node_args.shared_args;

  let seq_len = sequence.len();

  if seq_len >= NodePredictor::MAX_NUM_PREDICTIONS as usize {
    let err_msg = format!(
      "\x1b[31m[ERROR] Sequence length exceeds limit! Max sequence length is {}!\nSee here for more : https://github.com/matthewoestreich/js-randomness-predictor-rust/blob/master/README.md#random-number-pool-exhaustion\x1b[0m",
      NodePredictor::MAX_NUM_PREDICTIONS - 1
    );
    println!("{err_msg}");
    return Err(Box::new(errors::PredictionLimitError));
  }

  let max_preds_usize = NodePredictor::MAX_NUM_PREDICTIONS as usize;
  let has_limit_error = (seq_len + predictions) > max_preds_usize;

  if has_limit_error {
    predictions = max_preds_usize - seq_len;
  }

  let major_ver = node_args.major_version;
  let predictor = NodePredictor::new(major_ver, sequence.clone());
  let prediction_result = run_predictor_and_maybe_export_predictions(
    predictor,
    format!("Node.js {major_ver}"),
    sequence,
    predictions,
    export,
  );

  // If warning, log warning to console only after results have been logged!
  if has_limit_error {
    let warn_msg = format!(
      "\x1b[33m[WARNING] Results have been truncated to {predictions}. Max prediction limit exceeded!\nSequence length + number of predictions cannot exceend {}!\nSee here for more : https://github.com/matthewoestreich/js-randomness-predictor-rust/blob/master/README.md#random-number-pool-exhaustion\x1b[0m",
      NodePredictor::MAX_NUM_PREDICTIONS
    );
    println!("{warn_msg}");
  }

  return prediction_result;
}

pub fn parse_strict_float(s: &str) -> Result<f64, String> {
  if s.contains('.') {
    return s.parse::<f64>().map_err(|e| format!("Invalid float: {e}"));
  }
  return Err(format!("Expected a float with decimal point, got '{s}'"));
}

pub fn parse_export_path(s: &str) -> Result<ExportPath, String> {
  let p = Path::new(s);
  if let Some(extension) = p.extension() {
    if extension != "json" {
      #[rustfmt::skip]
      let e = format!("Expected 'export <path>' to point to a .json file, but got '{extension:?}'");
      return Err(e);
    }
    if p.is_absolute() || s.contains("/") {
      return Ok(ExportPath { path: s.into() });
    }
  }
  return Err("Invalid export path! Path must be to a .json file!".into());
}

pub fn run_predictor_and_maybe_export_predictions<P: Predictor>(
  mut predictor: P,
  environment: String,
  sequence: Vec<f64>,
  predictions: usize,
  export_path: Option<ExportPath>,
) -> Result<(), Box<dyn Error>> {
  let mut pred_res = PredictionResult {
    environment: &environment,
    sequence,
    predictions: vec![],
  };

  for _ in 0..predictions {
    let pred = predictor.predict_next()?;
    pred_res.predictions.push(pred);
  }

  let formatted = to_string_pretty(&pred_res)?;
  println!("{formatted}");

  if let Some(export) = export_path {
    fs::write(export.path, formatted)?;
  }

  return Ok(());
}
