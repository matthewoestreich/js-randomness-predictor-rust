#![allow(clippy::needless_return)]
#![warn(clippy::implicit_return)]

use clap::{Args, Parser, Subcommand};
use js_randomness_predictor::*;
use serde::Serialize;
use serde_json::to_string_pretty;
use std::{error::Error, fs, path::PathBuf};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
  #[command(subcommand)]
  environments: Environments,
}

#[derive(Debug, Subcommand)]
#[clap(rename_all = "lower")]
enum Environments {
  /// Node.js
  Node(NodeArgs),
  /// Firefox
  Firefox(SharedArgs),
  /// Chrome
  Chrome(SharedArgs),
  /// Safari
  Safari(SharedArgs),
}

#[derive(Parser, Clone, Debug)]
struct SharedArgs {
  /// Sequence of observed outputs [floating point required]
  #[arg(short, long, required = true, value_parser = parse_strict_float, num_args = 1..)]
  sequence: Vec<f64>,

  /// Number of predictions to make
  #[arg(short, long, required = false, default_value_t = 10)]
  predictions: usize,

  /// Path to export results to. Must be a '.json' file!
  #[arg(short, long, required = false, value_parser = parse_export_path)]
  export: Option<ExportPath>,
}

#[derive(Clone, Debug, Args)]
struct NodeArgs {
  #[clap(flatten)]
  shared_args: SharedArgs,

  #[arg(short, long, required = true)]
  major_version: NodeJsMajorVersion,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ExportPath {
  path: PathBuf,
}

fn parse_strict_float(s: &str) -> Result<f64, String> {
  if s.contains('.') {
    return s.parse::<f64>().map_err(|e| format!("Invalid float: {e}"));
  }
  return Err(format!("Expected a float with decimal point, got '{s}'"));
}

fn parse_export_path(s: &str) -> Result<ExportPath, String> {
  let p = std::path::Path::new(s);
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

#[derive(Serialize)]
struct PredictionResult<'a> {
  environment: &'a str,
  sequence: Vec<f64>,
  predictions: Vec<f64>,
}

fn run_predictor_and_maybe_export_predictions<P: Predictor>(
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

fn main() -> Result<(), Box<dyn Error>> {
  let cli = Cli::parse();
  match cli.environments {
    Environments::Node(node_args) => {
      #[rustfmt::skip]
      let SharedArgs { predictions, sequence, export } = node_args.shared_args;
      let major_ver = node_args.major_version;
      let predictor = NodePredictor::new(major_ver, sequence.clone());
      return run_predictor_and_maybe_export_predictions(
        predictor,
        format!("Node.js {major_ver}"),
        sequence,
        predictions,
        export,
      );
    }
    Environments::Firefox(args) => {
      let predictor = FirefoxPredictor::new(args.sequence.clone());
      return run_predictor_and_maybe_export_predictions(
        predictor,
        String::from("Firefox"),
        args.sequence,
        args.predictions,
        args.export,
      );
    }
    Environments::Chrome(args) => {
      let predictor = ChromePredictor::new(args.sequence.clone());
      return run_predictor_and_maybe_export_predictions(
        predictor,
        String::from("Chrome"),
        args.sequence,
        args.predictions,
        args.export,
      );
    }
    Environments::Safari(args) => {
      let predictor = SafariPredictor::new(args.sequence.clone());
      return run_predictor_and_maybe_export_predictions(
        predictor,
        String::from("Safari"),
        args.sequence,
        args.predictions,
        args.export,
      );
    }
  }
}
