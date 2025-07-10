#![allow(clippy::needless_return)]
#![warn(clippy::implicit_return)]
#![deny(clippy::unnecessary_mut_passed)]

mod chrome_predictor;
mod firefox_predictor;
mod node_predictor;
mod safari_predictor;

mod nodejs_major_version;
mod predictor;

// Public exports

pub mod errors;

pub use chrome_predictor::ChromePredictor;
pub use firefox_predictor::FirefoxPredictor;
pub use node_predictor::NodePredictor;
pub use nodejs_major_version::NodeJsMajorVersion;
pub use predictor::Predictor;
pub use safari_predictor::SafariPredictor;
