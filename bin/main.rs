#![allow(clippy::needless_return)]
#![warn(clippy::implicit_return)]

mod cmd_line_parser;
mod jsrp_lib;

use clap::Parser;
use cmd_line_parser::*;
use js_randomness_predictor::*;
use jsrp_lib::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
  match Cli::parse().environments {
    /*
      Node
    */
    Environments::Node(node_args) => {
      return handle_node(node_args);
    }

    /*
      Firefox
    */
    Environments::Firefox(args) => {
      let predictor = FirefoxPredictor::new(args.sequence.clone());
      return run_predictor_and_maybe_export_predictions(
        predictor,
        "Firefox".to_string(),
        args.sequence,
        args.predictions,
        args.expected,
        args.export,
      );
    }

    /*
      Chrome
    */
    Environments::Chrome(args) => {
      let predictor = ChromePredictor::new(args.sequence.clone());
      return run_predictor_and_maybe_export_predictions(
        predictor,
        "Chrome".to_string(),
        args.sequence,
        args.predictions,
        args.expected,
        args.export,
      );
    }

    /*
      Safari
    */
    Environments::Safari(args) => {
      let predictor = SafariPredictor::new(args.sequence.clone());
      return run_predictor_and_maybe_export_predictions(
        predictor,
        "Safari".to_string(),
        args.sequence,
        args.predictions,
        args.expected,
        args.export,
      );
    }

    // Should never reach here, but still.
    #[allow(unreachable_patterns)]
    _ => {
      return Err(Box::from("[ERROR] UNKNOWN ENVIRONMENT!"));
    }
  }
}
