extern crate clap;
extern crate rayon;
extern crate walkdir;

use clap::App;
use clap::Arg;
use rayon::prelude::*;
use walkdir::WalkDir;

use std::env;
use std::fs;
use std::path::Path;

enum Entry<'a> {
    Folder(&'a str),
    File(&'a str),
}

const TARGET_DIR: &'static str = "target";

fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::with_name("dry_run")
                .short("d")
                .long("dry-run")
                .help("do not perform cleanup"),
        )
        .arg(
            Arg::with_name("interactive")
                .short("i")
                .long("interactive")
                .help("requires confirmation for each clean"),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("print more information during processing"),
        )
        .arg(
            Arg::with_name("target")
                .short("t")
                .long("target")
                .takes_value(true)
                .help("use specified directory as starting point"),
        )
        // to avoid conflict when run with `cargo rclean`
        .arg(Arg::with_name("rclean").hidden(true))
        .get_matches();

    let dryrun = matches.is_present("dry_run");
    let v = matches.is_present("verbose");
    let dir = matches
        .value_of("target")
        .or(env::current_dir().unwrap().to_str())
        .unwrap()
        .to_owned();
    let required_entries = vec![
        Entry::File("Cargo.toml"),
        Entry::Folder("src"),
        Entry::Folder(TARGET_DIR),
    ];

    let found_dirs = find_applicable_folders(dir, &required_entries);
    if found_dirs.len() > 0 {
        if dryrun {
            vprint(v, "Can clean :");
            for e in found_dirs {
                vprint(v, format!("\t{}", e).as_str());
            }
        } else {
            vprint(
                v,
                format!("Going to clean {} projects...", found_dirs.len()).as_str(),
            );
            clean_folders(found_dirs)
                .iter()
                .for_each(|e| vprint(v, format!("Error in clean folders : {}", e).as_str()));
        }
    } else {
        vprint(v, "Nothing to clean");
    }
}

fn find_applicable_folders<P: AsRef<Path>>(
    target_dir: P,
    required_entries: &Vec<Entry>,
) -> Vec<String> {
    let mut vec = Vec::new();
    for entry in WalkDir::new(target_dir)
        .into_iter()
        .filter_entry(|e| e.file_type().is_dir())
    {
        let entry = entry.unwrap();
        let mut exists = true;

        for e in required_entries {
            match e {
                Entry::Folder(s) => {
                    let joined = entry.path().join(Path::new(s));
                    if !joined.exists() || !joined.is_dir() {
                        exists = false;
                        break;
                    }
                }
                Entry::File(s) => {
                    let joined = entry.path().join(Path::new(s));
                    if !joined.exists() || !joined.is_file() {
                        exists = false;
                        break;
                    }
                }
            };
        }

        if !exists {
            continue;
        }

        vec.push(entry.path().to_string_lossy().into_owned());
    }
    vec
}

fn clean_folders(dirs: Vec<String>) -> Vec<std::io::Error> {
    dirs.par_iter()
        .map(|dir| {
            let path = Path::new(dir).join(Path::new(TARGET_DIR));
            fs::remove_dir_all(path).map_err(|e| e)
        })
        .filter_map(|x| x.err())
        .collect()
}

fn vprint(verbose: bool, msg: &str) {
    if verbose {
        println!("{}", msg);
    }
}
