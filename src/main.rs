use std::{
    env::args,
    fs::{self, read_dir, DirEntry},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

struct Entry {
    name: String,
    size: u64,
}

fn entry_size(entry: &DirEntry) -> Result<u64> {
    let info = entry.metadata()?;
    let size = if info.is_file() {
        info.len()
    } else {
        let mut dir_sum = 0;
        for entry in read_dir(entry.path())? {
            dir_sum += entry_size(&entry?)?
        }
        dir_sum
    };
    Ok(size)
}

fn convert_dir_entry(entry: DirEntry) -> Result<Entry> {
    let name = entry.file_name().to_string_lossy().into_owned();
    let size = entry_size(&entry)?;
    Ok(Entry { name, size })
}

fn main() -> Result<()> {
    let mut args = args();
    let dir_path = args
        .nth(1)
        .map(|x| x.into())
        .unwrap_or(fs::canonicalize("./")?);

    let dir = fs::read_dir(&dir_path)?;
    let mut entries = vec![];
    for entry in dir {
        entries.push(convert_dir_entry(entry?)?);
    }

    entries.sort_unstable_by_key(|x| x.size);
    entries.reverse();
    println!("[.../{}]", dir_path.file_name().unwrap().to_string_lossy());
    for entry in entries {
        let size = entry.size;
        let size_mb = (size as f64) / 1000.0 / 1000.0;
        println!("{} [{:.3}MB]", entry.name, size_mb);
    }
    Ok(())
}
