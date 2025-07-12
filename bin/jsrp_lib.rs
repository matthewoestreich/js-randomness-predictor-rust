use crate::cmd_line_parser::*;
use js_randomness_predictor::*;
use serde::Serialize;
use serde_json::{to_string_pretty, to_value};
use std::{error::Error, fs, path::Path};

#[derive(Serialize)]
pub struct PredictionResult {
  pub environment: String,
  pub sequence: Vec<f64>,
  pub predictions: Vec<f64>,
  pub expected: Vec<f64>,
  pub is_accurate: bool,
}

pub fn handle_node(node_args: NodeArgs) -> Result<(), Box<dyn Error>> {
  let SharedArgs {
    mut predictions,
    sequence,
    export,
    mut expected,
  } = node_args.shared_args;

  let seq_len = sequence.len();
  let max_preds_usize = NodePredictor::MAX_NUM_PREDICTIONS as usize;

  if let Some(ref expected_predictions) = expected {
    predictions = expected_predictions.len();
  }

  if seq_len >= max_preds_usize {
    let err_msg = format!(
      "\x1b[31m[ERROR] Sequence length exceeds limit! Max sequence length is {}!\nSee here for more : https://github.com/matthewoestreich/js-randomness-predictor-rust/blob/master/README.md#random-number-pool-exhaustion\x1b[0m",
      max_preds_usize - 1
    );
    println!("{err_msg}");
    return Ok(());
  }

  let has_limit_error = (seq_len + predictions) > max_preds_usize;
  if has_limit_error {
    predictions = max_preds_usize - seq_len;
    if let Some(ref mut exp) = expected {
      exp.truncate(predictions);
    }
  }

  let major_ver = node_args.major_version;
  let predictor = NodePredictor::new(major_ver, sequence.clone());

  let prediction_result = run_predictor(
    predictor,
    format!("Node.js {major_ver}"),
    sequence,
    predictions,
    expected,
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

pub fn run_predictor<P: Predictor>(
  mut predictor: P,
  environment: String,
  sequence: Vec<f64>,
  num_of_predictions: usize,
  expected: Option<Vec<f64>>,
  export_path: Option<ExportPath>,
) -> Result<(), Box<dyn Error>> {
  let mut pred_res = PredictionResult {
    environment,
    sequence,
    predictions: vec![],
    is_accurate: false,
    expected: vec![],
  };

  let mut total_num_predictions = num_of_predictions;

  // If user provided expected results, use the length of
  // them as 'predictions' (aka num of predictions, which is 10 by default).
  // Since users cannot use --predictions and --expected flags at the same
  // time, we only need to check for 'expected' flag.
  if let Some(expected_preds) = expected {
    pred_res.expected = expected_preds;
    total_num_predictions = pred_res.expected.len();
  }

  // Make predictions.
  for _ in 0..total_num_predictions {
    let pred = predictor.predict_next()?;
    pred_res.predictions.push(pred);
  }

  // If user provided expected results, validate them.
  if !pred_res.expected.is_empty() {
    pred_res.is_accurate = true;
    for (idx, pred) in pred_res.predictions.iter().enumerate() {
      if *pred != pred_res.expected[idx] {
        pred_res.is_accurate = false;
        break;
      }
    }
  }

  // Converts our struct to a JSON object.
  let mut json_pred_res = to_value(&pred_res)?;

  // If user did not provide expected results, remove
  // unnecessary fields from our JSON results/report.
  if pred_res.expected.is_empty()
    && let Some(json) = json_pred_res.as_object_mut()
  {
    json.remove("expected");
    json.remove("is_accurate");
  }

  // Log results to console so user can view them.
  let formatted = to_string_pretty(&json_pred_res)?;
  println!("{formatted}");

  // Export if user specified.
  if let Some(export) = export_path {
    fs::write(export.path, formatted)?;
  }

  return Ok(());
}
