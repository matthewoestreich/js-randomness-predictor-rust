mod chrome_predictor;
mod firefox_predictor;
mod node_predictor;
mod safari_predictor;
mod v8_predictor;

mod errors;

pub use chrome_predictor::ChromePredictor;
pub use firefox_predictor::FirefoxPredictor;
pub use node_predictor::NodePredictor;
pub use safari_predictor::SafariPredictor;
pub use v8_predictor::V8Predictor;
