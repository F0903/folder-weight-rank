use std::{
    env::args,
    fs::{self, read_dir, DirEntry},
    io::ErrorKind,
    time, vec,
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

    let start_time = time::Instant::now();

    let read_dir = match fs::read_dir(&dir_path) {
        Ok(x) => x,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => {
                return Err(
                    "Filepath was invalid. Try wrapping the path in quotation marks.".into(),
                )
            }
            _ => return Err(Box::new(e)),
        },
    };
    let mut dir = vec![];
    for entry in read_dir {
        dir.push(entry?)
    }

    let mut entries = std::thread::scope(|s| {
        let mut handles = vec![];
        for entry in dir {
            let handle = s.spawn(|| {
                let ent = match convert_dir_entry(entry) {
                    Ok(x) => x,
                    Err(_) => return None,
                };
                Some(ent)
            });
            handles.push(handle);
        }
        let mut entries = vec![];
        for handle in handles {
            if let Some(x) = handle.join().unwrap() {
                entries.push(x);
            }
        }
        entries
    });

    entries.sort_unstable_by_key(|x| x.size);
    entries.reverse();
    for entry in entries {
        let size = entry.size;
        let size_mb = (size as f64) / 1000.0 / 1000.0;
        println!("{} [{:.3}MB]", entry.name, size_mb);
    }

    let end_time = time::Instant::now();
    let time_taken = end_time - start_time;
    println!("Time taken: {:.5}ms", time_taken.as_millis());

    Ok(())
}
