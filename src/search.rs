use crate::{file_size::FileSize, run_args::RunArgs};
use std::{
    fs::{self, read_dir, DirEntry, ReadDir},
    io::ErrorKind,
    sync::atomic::{AtomicU8, Ordering},
    thread, time, vec,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

struct Entry {
    name: String,
    size: FileSize,
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
    Ok(Entry {
        name,
        size: FileSize::bytes(size as usize),
    })
}

fn get_entries(dir: ReadDir, max_threads: u8) -> Vec<Entry> {
    let running_threads = AtomicU8::new(0);
    thread::scope(|s| {
        let mut handles: Vec<thread::ScopedJoinHandle<Option<Entry>>> = vec![];
        for entry in dir {
            if running_threads.load(Ordering::Relaxed) >= max_threads {
                'handle_loop: loop {
                    for handle in handles.iter() {
                        if handle.is_finished() {
                            running_threads.fetch_sub(1, Ordering::Relaxed);
                            break 'handle_loop;
                        }
                        thread::yield_now();
                    }
                }
            }
            let handle = s.spawn(|| {
                let entry = match entry {
                    Ok(x) => x,
                    Err(_) => return None,
                };
                let ent = match convert_dir_entry(entry) {
                    Ok(x) => x,
                    Err(_) => return None,
                };
                Some(ent)
            });
            handles.push(handle);
            running_threads.fetch_add(1, Ordering::Relaxed);
        }
        let mut entries = vec![];
        for handle in handles {
            if let Some(x) = handle.join().unwrap() {
                entries.push(x);
            }
        }
        entries
    })
}

pub fn search(args: RunArgs) -> Result<()> {
    let start_time = if args.is_timed() {
        Some(time::Instant::now())
    } else {
        None
    };

    let read_dir = match fs::read_dir(args.get_path()) {
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

    let mut entries = get_entries(read_dir, args.get_thread_count());
    entries.sort_unstable_by_key(|x| x.size.get() as usize);
    entries.reverse();
    for entry in entries {
        println!("{} {}", entry.name, entry.size.get_as(args.get_size_unit()));
    }

    if args.is_timed() {
        let start_time = unsafe { start_time.unwrap_unchecked() };
        let end_time = time::Instant::now();
        let time_taken = end_time - start_time;
        println!("Time taken: {:.5}ms", time_taken.as_millis());
    }
    Ok(())
}
