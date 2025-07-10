use crate::jsrp_lib::*;
use clap::{Args, Parser, Subcommand};
use js_randomness_predictor::*;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
  #[command(subcommand)]
  pub environments: Environments,
}

#[derive(Debug, Subcommand)]
#[clap(rename_all = "lower")]
pub enum Environments {
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
pub struct SharedArgs {
  /// Sequence of observed outputs [floating point required]
  #[arg(short, long, required = true, value_parser = parse_strict_float, num_args = 1..)]
  pub sequence: Vec<f64>,

  /// Number of predictions to make
  #[arg(
    short,
    long,
    required = false,
    default_value_t = 10,
    group = "preds_or_expected"
  )]
  pub predictions: usize,

  /// Expected prediction values
  #[arg(short = 'x', long, required = false, value_parser = parse_strict_float, num_args = 1.., group = "preds_or_expected")]
  pub expected: Option<Vec<f64>>,

  /// Path to export results to. Must be a '.json' file!
  #[arg(short, long, required = false, value_parser = parse_export_path)]
  pub export: Option<ExportPath>,
}

#[derive(Clone, Debug, Args)]
pub struct NodeArgs {
  #[clap(flatten)]
  pub shared_args: SharedArgs,

  #[arg(short, long, required = true)]
  pub major_version: NodeJsMajorVersion,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ExportPath {
  pub path: PathBuf,
}
