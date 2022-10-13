use std::{
    env::Args,
    path::{Path, PathBuf},
};

use crate::file_size::SizeUnit;

type Result<T> = std::result::Result<T, String>;

pub struct RunArgs {
    path: PathBuf,
    timed: bool,
    size_unit: SizeUnit,
}

impl RunArgs {
    fn with_arg(&mut self, arg: &str) -> Result<()> {
        if !arg.starts_with("-") {
            self.path.push(arg);
            if !self.path.is_dir() {
                self.path = self.path.parent().unwrap().to_owned();
            }
            return Ok(());
        }
        match &arg[1..] {
            "t" => self.timed = true,
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
        for arg in args {
            run_args.with_arg(&arg)?;
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
}

impl Default for RunArgs {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            timed: false,
            size_unit: SizeUnit::KiloByte,
        }
    }
}
