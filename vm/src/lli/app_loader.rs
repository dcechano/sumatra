use std::{
    ffi::OsStr,
    fs::{self, DirEntry},
    path::PathBuf,
};

use sumatra_parser::class_file::ClassFile;

use crate::{
    lli::loader::ClassLoader,
    result::{Error, Result},
    vm_error,
};

pub(super) struct AppLoader {
    c_path: PathBuf,
}

impl AppLoader {
    pub(super) fn new(c_path: PathBuf) -> Self { Self { c_path } }

    //TODO Consider a better way than recursion.
    fn find<S: AsRef<OsStr>>(&self, name: S) -> Option<PathBuf> {
        let entries = fs::read_dir(&self.c_path).unwrap();
        for entry in entries.flatten() {
            match entry.file_type().unwrap().is_dir() {
                true => {
                    if let Some(path_to_class) = self.find(entry.path().as_os_str()) {
                        return Some(path_to_class);
                    }
                }
                false => {
                    if let Some(path_to_class) = self.check(entry, name.as_ref()) {
                        return Some(path_to_class);
                    }
                }
            }
        }
        None
    }

    fn check<S: AsRef<OsStr>>(&self, entry: DirEntry, name: S) -> Option<PathBuf> {
        let path = entry.path();
        if path.file_name().unwrap() == name.as_ref() {
            return Some(path);
        }
        None
    }
}

impl ClassLoader for AppLoader {
    fn get(&mut self, name: &str) -> Result<ClassFile> {
        match self.find(name) {
            None => vm_error!("Class {name} not found."),
            Some(path) => Ok(ClassFile::parse_class(path)
                .map_err(|_| Error::ParseError(format!("Error parsing class: {name}")))?),
        }
    }
}
