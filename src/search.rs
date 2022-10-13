use crate::{file_size::FileSize, run_args::RunArgs};
use std::{
    fs::{self, read_dir, DirEntry, ReadDir},
    io::ErrorKind,
    sync::{
        atomic::{AtomicU8, Ordering},
        mpsc::channel,
        Arc, Condvar, Mutex,
    },
    thread, time,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

struct Entry {
    name: String,
    size: FileSize,
}

unsafe impl Sync for Entry {}
unsafe impl Send for Entry {}

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
    let (tx, rx) = channel();
    thread::scope(|s| {
        let cvar_pair = Arc::new((Mutex::new(false), Condvar::new()));
        let (lock, cvar) = &*cvar_pair;
        for entry in dir {
            if running_threads.load(Ordering::Acquire) >= max_threads {
                let mut all_busy = lock.lock().unwrap();
                *all_busy = true;
                let _ = cvar.wait(all_busy).unwrap();
            }
            let cvar_pair = cvar_pair.clone();
            let tx = tx.clone();
            s.spawn(|| {
                let cvar_pair = cvar_pair;
                let tx = tx;
                let entry = match entry {
                    Ok(x) => x,
                    Err(_) => return,
                };
                let ent = match convert_dir_entry(entry) {
                    Ok(x) => x,
                    Err(_) => return,
                };
                running_threads.fetch_sub(1, Ordering::AcqRel);
                let (lock, cvar) = &*cvar_pair;
                let mut all_busy = lock.lock().unwrap();
                *all_busy = false;
                cvar.notify_one();
                tx.send(ent).unwrap();
            });
            running_threads.fetch_add(1, Ordering::Release);
        }
    });
    drop(tx); // Must be manually dropped or else the statement below will never return.
    rx.iter().collect()
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
