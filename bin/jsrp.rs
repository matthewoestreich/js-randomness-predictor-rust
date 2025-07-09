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
    s.parse::<f64>()
      .map_err(|e| format!("Invalid float: {}", e))
  } else {
    Err(format!("Expected a float with decimal point, got '{}'", s))
  }
}

fn parse_export_path(s: &str) -> Result<ExportPath, String> {
  let p = std::path::Path::new(s);
  if let Some(extension) = p.extension() {
    if extension != "json" {
      #[rustfmt::skip]
      let e = format!("Expected 'export <path>' to point to a .json file, but got '{:?}'", extension);
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

// TODO: clean this up!
fn main() -> Result<(), Box<dyn Error>> {
  let cli = Cli::parse();
  match cli.environments {
    Environments::Node(node_args) => {
      #[rustfmt::skip]
      let SharedArgs { predictions, sequence, export } = node_args.shared_args;
      let major_ver = node_args.major_version;
      let mut node = NodePredictor::new(major_ver, sequence.clone());
      let mut pred_res = PredictionResult {
        environment: &format!("Node.js {}", major_ver),
        sequence: vec![],
        predictions: vec![],
      };
      pred_res.sequence = sequence;
      for _ in 0..predictions {
        let pred = node.predict_next()?;
        pred_res.predictions.push(pred);
      }
      let formatted = format!("{}", to_string_pretty(&pred_res)?);
      println!("{}", formatted);
      if let Some(exportp) = export {
        fs::write(exportp.path, formatted)?;
      }
      return Ok(());
    }
    Environments::Firefox(args) => {
      let mut predictor = FirefoxPredictor::new(args.sequence.clone());
      let mut pred_res = PredictionResult {
        environment: "Firefox",
        sequence: args.sequence,
        predictions: vec![],
      };
      for _ in 0..args.predictions {
        let pred = predictor.predict_next()?;
        pred_res.predictions.push(pred);
      }
      let formatted = format!("{}", to_string_pretty(&pred_res)?);
      println!("{}", formatted);
      if let Some(export) = args.export {
        fs::write(export.path, formatted)?;
      }
      return Ok(());
    }
    Environments::Chrome(args) => {
      let mut predictor = ChromePredictor::new(args.sequence.clone());
      let mut pred_res = PredictionResult {
        environment: "Chrome",
        sequence: args.sequence,
        predictions: vec![],
      };
      for _ in 0..args.predictions {
        let pred = predictor.predict_next()?;
        pred_res.predictions.push(pred);
      }
      let formatted = format!("{}", to_string_pretty(&pred_res)?);
      println!("{}", formatted);
      if let Some(export) = args.export {
        fs::write(export.path, formatted)?;
      }
      return Ok(());
    }
    Environments::Safari(args) => {
      let mut predictor = SafariPredictor::new(args.sequence.clone());
      let mut pred_res = PredictionResult {
        environment: "Safari",
        sequence: args.sequence,
        predictions: vec![],
      };
      for _ in 0..args.predictions {
        let pred = predictor.predict_next()?;
        pred_res.predictions.push(pred);
      }
      let formatted = format!("{}", to_string_pretty(&pred_res)?);
      println!("{}", formatted);
      if let Some(export) = args.export {
        fs::write(export.path, formatted)?;
      }
      return Ok(());
    }
  }
}
