use clap::{Args, Parser, Subcommand, ValueEnum};
use js_randomness_predictor::*;
use std::{error::Error, path::PathBuf};

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
  #[group()]
  Node(NodeArgs),
  /// Firefox
  Firefox(SharedArgs),
  /// Chrome
  Chrome(SharedArgs),
  /// Safari
  Safari(SharedArgs),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ExportPath(PathBuf);

#[derive(Parser, Clone, Debug)]
struct SharedArgs {
  /// Sequence of observed outputs [floating point required]
  #[arg(short, long, required = true, value_parser = parse_strict_float, num_args = 1..)]
  sequence: Vec<f64>,

  /// Number of predictions to make
  #[arg(short, long, required = false, default_value_t = 10)]
  predictions: usize,

  /// Path to export results to
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

#[derive(Clone, Debug, ValueEnum)]
enum NodeJsMajorVersion {
  V0 = 0,
  V4 = 4,
  V5 = 5,
  V6 = 6,
  V7 = 7,
  V8 = 8,
  V9 = 9,
  V10 = 10,
  V11 = 11,
  V12 = 12,
  V13 = 13,
  V14 = 14,
  V15 = 15,
  V16 = 16,
  V17 = 17,
  V18 = 18,
  V19 = 19,
  V20 = 20,
  V21 = 21,
  V22 = 22,
  V23 = 23,
  V24 = 24,
}

impl NodeJsMajorVersion {
  #[allow(dead_code)]
  pub fn from_u8(value: u8) -> Option<Self> {
    match value {
      0 => Some(Self::V0),
      4 => Some(Self::V4),
      5 => Some(Self::V5),
      6 => Some(Self::V6),
      7 => Some(Self::V7),
      8 => Some(Self::V8),
      9 => Some(Self::V9),
      10 => Some(Self::V10),
      11 => Some(Self::V11),
      12 => Some(Self::V12),
      13 => Some(Self::V13),
      14 => Some(Self::V14),
      15 => Some(Self::V15),
      16 => Some(Self::V16),
      17 => Some(Self::V17),
      18 => Some(Self::V18),
      19 => Some(Self::V19),
      20 => Some(Self::V20),
      21 => Some(Self::V21),
      22 => Some(Self::V22),
      23 => Some(Self::V23),
      24 => Some(Self::V24),
      _ => None,
    }
  }
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
  // only accept if the value immediately follows the keyword "export"
  if s == "export" {
    Err("Expected a path after 'export', but got nothing.".into())
  } else if std::path::Path::new(s).is_absolute() || s.contains('/') {
    Ok(ExportPath(s.into()))
  } else {
    Err(format!("Expected 'export <path>', but got '{}'", s))
  }
}

#[derive(Debug)]
struct PredictionResult {
  sequence: Vec<f64>,
  predictions: Vec<f64>,
}

fn main() -> Result<(), Box<dyn Error>> {
  let cli = Cli::parse();
  match cli.environments {
    Environments::Node(node_args) => {
      let SharedArgs {
        predictions,
        sequence,
        export,
      } = node_args.shared_args;
      let major_ver = node_args.major_version as i32;
      let mut node = NodePredictor::new(major_ver, sequence.clone());
      let mut pred_res = PredictionResult {
        sequence: vec![],
        predictions: vec![],
      };
      pred_res.sequence = sequence;
      for _ in 0..predictions {
        let pred = node.predict_next()?;
        pred_res.predictions.push(pred);
      }
      let formatted = format!("{:#?}", pred_res);
      println!("{}", formatted);
      return Ok(());
    }
    Environments::Firefox(args) => {
      let mut predictor = FirefoxPredictor::new(args.sequence.clone());
      let mut pred_res = PredictionResult {
        sequence: args.sequence,
        predictions: vec![],
      };
      for _ in 0..args.predictions {
        let pred = predictor.predict_next()?;
        pred_res.predictions.push(pred);
      }
      let formatted = format!("{:#?}", pred_res);
      println!("{}", formatted);
      return Ok(());
    }
    Environments::Chrome(args) => {
      let mut predictor = ChromePredictor::new(args.sequence.clone());
      let mut pred_res = PredictionResult {
        sequence: args.sequence,
        predictions: vec![],
      };
      for _ in 0..args.predictions {
        let pred = predictor.predict_next()?;
        pred_res.predictions.push(pred);
      }
      let formatted = format!("{:#?}", pred_res);
      println!("{}", formatted);
      return Ok(());
    }
    Environments::Safari(args) => {
      let mut predictor = SafariPredictor::new(args.sequence.clone());
      let mut pred_res = PredictionResult {
        sequence: args.sequence,
        predictions: vec![],
      };
      for _ in 0..args.predictions {
        let pred = predictor.predict_next()?;
        pred_res.predictions.push(pred);
      }
      let formatted = format!("{:#?}", pred_res);
      println!("{}", formatted);
      return Ok(());
    }
  }
}
