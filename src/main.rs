mod file_size;
mod run_args;
mod search;

use search::search;
use std::env::args;

use crate::run_args::RunArgs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = RunArgs::from_args(args())?;
    search(args)
}
