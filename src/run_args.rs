use std::{
    env::Args,
    path::{Path, PathBuf},
    thread::available_parallelism,
};

use crate::file_size::SizeUnit;

type Result<T> = std::result::Result<T, String>;

pub struct RunArgs {
    path: PathBuf,
    timed: bool,
    size_unit: SizeUnit,
    thread_count: u8,
}

impl RunArgs {
    fn with_arg(&mut self, arg: &str, next_arg: Option<&String>) -> Result<()> {
        if !arg.starts_with("-") {
            self.path.push(arg);
            if !self.path.is_dir() {
                self.path = self.path.parent().unwrap().to_owned();
            }
            return Ok(());
        }
        match &arg[1..] {
            "t" => self.timed = true,
            "tc" => {
                self.thread_count = match next_arg {
                    Some(x) => x.parse::<u8>().map_err(|e| e.to_string())?,
                    None => return Err("Please specify a number to use the 'tc' option.".into()),
                }
            }
            "b" | "B" => self.size_unit = SizeUnit::Byte,
            "kb" | "KB" => self.size_unit = SizeUnit::KiloByte,
            "mb" | "MB" => self.size_unit = SizeUnit::MegaByte,
            "gb" | "GB" => self.size_unit = SizeUnit::GigaByte,
            _ => return Err("Unrecognized argument.".to_owned()),
        }
        Ok(())
    }

    pub fn from_args(args: Args) -> Result<Self> {
        let mut run_args = Self::default();
        let mut arg_iter = args.skip(1).peekable();
        loop {
            let arg = match arg_iter.next() {
                Some(x) => x,
                None => break,
            };
            let next_arg = arg_iter.peek();
            run_args.with_arg(&arg, next_arg)?;
        }
        Ok(run_args)
    }

    pub fn get_path(&self) -> &Path {
        &self.path
    }

    pub fn is_timed(&self) -> bool {
        self.timed
    }

    pub fn get_size_unit(&self) -> SizeUnit {
        self.size_unit
    }

    pub fn get_thread_count(&self) -> u8 {
        self.thread_count
    }
}

impl Default for RunArgs {
    fn default() -> Self {
        Self {
            path: std::env::current_dir().expect("Could not get current dir!"),
            timed: false,
            size_unit: SizeUnit::KiloByte,
            thread_count: available_parallelism()
                .map(|x| x.get())
                .unwrap_or(8)
                .clamp(1, u8::MAX as usize) as u8,
        }
    }
}
