use std::{collections::HashMap, fs::File, io::Read, path::Path};

use anyhow::{bail, Result};
use zip::ZipArchive;

use sumatra_parser::class_file::ClassFile;

use crate::class::Class;

const DEFAULT_CAPACITY: usize = 64;

#[derive(Default)]
pub(crate) struct ClassLoader {
    loaded: Vec<Class>,
    bookkeeper: HashMap<String, usize>,
}

impl ClassLoader {
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            loaded: Vec::with_capacity(DEFAULT_CAPACITY),
            bookkeeper: HashMap::with_capacity(DEFAULT_CAPACITY),
        }
    }
    pub(crate) fn resolve(&mut self, name: &str) -> Result<&Class> {
        if !self.bookkeeper.contains_key(name) {
            let class_file = unzip_jar(name)?;
            self.loaded.push(Class::from(&class_file));

            self.bookkeeper
                .insert(name.to_string(), self.loaded.len() - 1);
            Ok(self.loaded.last().unwrap())
        } else {
            Ok(self.by_name_unchecked(name))
        }
    }

    #[inline]
    pub(crate) fn by_name(&self, name: &str) -> Result<&Class> {
        match self.bookkeeper.get(name) {
            None => {
                bail!("No class found for name: {name}");
            }
            Some(index) => self.by_index(*index),
        }
    }

    #[inline]
    pub(crate) fn by_index(&self, index: usize) -> Result<&Class> {
        match self.loaded.get(index) {
            None => {
                bail!("No class found for index: {index}");
            }
            Some(class) => Ok(class),
        }
    }

    #[inline]
    fn by_index_unchecked(&self, index: usize) -> &Class { self.loaded.get(index).unwrap() }

    #[inline]
    fn by_name_unchecked(&self, name: &str) -> &Class {
        let index = self.bookkeeper.get(name).unwrap();
        self.by_index_unchecked(*index)
    }
}

#[inline]
fn unzip_jar(name: &str) -> Result<ClassFile> {
    let fname = Path::new("../jar/rt.jar");
    let zipfile = File::open(fname).unwrap();

    let mut archive = ZipArchive::new(zipfile)?;

    let mut file = archive.by_name(name)?;

    let mut contents = Vec::with_capacity(file.size() as usize);
    file.read_to_end(&mut contents).unwrap();
    ClassFile::parse_from_buffer(&contents)
}
