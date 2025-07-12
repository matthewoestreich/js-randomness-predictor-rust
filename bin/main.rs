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
      return run_predictor(
        FirefoxPredictor::new(args.sequence.clone()),
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
      return run_predictor(
        ChromePredictor::new(args.sequence.clone()),
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
      return run_predictor(
        SafariPredictor::new(args.sequence.clone()),
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
