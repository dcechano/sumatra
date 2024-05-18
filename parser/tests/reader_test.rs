use std::{
    fs,
    fs::{read_dir, DirEntry},
    path::PathBuf,
};

use anyhow::anyhow;

use sumatra_parser::class_file::ClassFile;

const JAVA_LANG: &str = "../jdk/compiled/java.base/";

// This test attempts to parse every class file in the java runtime.
// While useful for being sure the class file parsing is correct, it is quite
// slow.
#[test]
#[cfg(not(miri))]
fn test_no_panic() {
    let mut errors = Vec::with_capacity(256);
    for file in list_class_files() {
        let cloned = file.clone();
        let name = cloned.file_name().unwrap();
        let buff = fs::read(file);
        let result = match buff {
            Ok(buffer) => match ClassFile::parse_from_buffer(&buffer) {
                Ok(_) => Ok(()),
                Err(e) => Err(anyhow!(
                    "Error while parsing classfile: {name:?} from buffer: {e}."
                )),
            },

            Err(e) => Err(anyhow!("Could not read from file: {name:?}: {e:?}")),
        };
        if result.is_err() {
            errors.push(result);
        }
    }

    if !errors.is_empty() {
        for error in errors {
            eprintln!("{error:?}")
        }
        assert!(false);
    }
}

fn list_class_files() -> Vec<PathBuf> {
    let mut buffers = Vec::with_capacity(256);
    let entries = fs::read_dir(JAVA_LANG).unwrap();
    for entry in entries.flatten() {
        eprintln!("DirEntry: {entry:?}");
        if entry.file_type().unwrap().is_dir() {
            buffers.extend(process_dir(entry))
        }
    }
    if buffers.is_empty() {
        panic!("Zero files were read during class file parsing test");
    }
    buffers
}

// TODO Maybe refactor to not use recursion
fn process_dir(dir: DirEntry) -> Vec<PathBuf> {
    let mut paths = Vec::with_capacity(16);
    let entries = read_dir(dir.path()).unwrap();
    for entry in entries.flatten() {
        if entry.file_type().unwrap().is_dir() {
            paths.extend(process_dir(entry).into_iter())
        } else {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "class" {
                    paths.push(path);
                }
            }
        }
    }
    paths
}
