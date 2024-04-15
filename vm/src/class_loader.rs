use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    path::Path,
    sync::atomic::{AtomicUsize, Ordering},
};

use anyhow::{bail, Result};
use zip::ZipArchive;

use sumatra_parser::class_file::ClassFile;

use crate::{alloc::static_alloc::StaticAlloc, class::Class, method_area::MethodArea};

//TODO resolve all instances of the name `offset` to be index to reflect
// that the method area returns indices into its data structure not offsets.

const DEFAULT_CAPACITY: usize = 64;

pub(crate) struct ClassLoader {
    by_name: HashMap<String, usize>,
    by_id: HashMap<usize, usize>,
    count: AtomicUsize,
}

impl ClassLoader {
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            by_name: HashMap::with_capacity(DEFAULT_CAPACITY),
            by_id: HashMap::with_capacity(DEFAULT_CAPACITY),
            count: AtomicUsize::new(0),
        }
    }
    #[inline]
    pub(crate) fn resolve<'met>(
        &mut self,
        name: &str,
        met_area: &'met mut MethodArea,
    ) -> Result<&'met Class> {
        let index = self.resolve_and_index(name, met_area)?;
        met_area.get_class(index)
    }

    #[inline]
    pub(crate) fn resolve_static<'met>(
        &mut self,
        name: &str,
        met_area: &'met mut MethodArea,
    ) -> Result<&'met StaticAlloc> {
        let index = self.resolve_and_index(name, met_area)?;
        met_area.get(index)
    }

    pub(crate) fn resolve_static_mut<'met>(
        &mut self,
        name: &str,
        met_area: &'met mut MethodArea,
    ) -> Result<&'met mut StaticAlloc> {
        let index = self.resolve_and_index(name, met_area)?;
        met_area.get_mut(index)
    }

    #[inline]
    fn resolve_and_index(&mut self, name: &str, met_area: &mut MethodArea) -> Result<usize> {
        if !self.by_name.contains_key(name) {
            let class_file = unzip_jar(name)?;
            let class = Class::from(&class_file);
            self.store_class(class, met_area)
        } else {
            Ok(*self.by_name.get(name).unwrap())
        }
    }

    #[inline]
    fn store_class(&mut self, class: Class, met_area: &mut MethodArea) -> Result<usize> {
        let name = class.get_name();
        let offset = met_area.push(class)?;
        let id = self.count.load(Ordering::SeqCst);
        self.count.store(id + 1, Ordering::SeqCst);
        self.by_id.insert(id, offset);
        self.by_name.insert(name, offset);
        Ok(offset)
    }

    #[inline]
    pub(crate) fn by_name<'met>(
        &self,
        name: &str,
        met_area: &'met mut MethodArea,
    ) -> Result<&'met Class> {
        match self.by_name.get(name) {
            None => {
                bail!("No class found for name: {name}");
            }
            Some(index) => self.by_offset(*index, met_area),
        }
    }

    #[inline]
    pub(crate) fn by_offset<'met>(
        &self,
        offset: usize,
        met_area: &'met mut MethodArea,
    ) -> Result<&'met Class> {
        met_area.get_class(offset)
    }

    #[inline]
    fn by_offset_unchecked<'met>(
        &self,
        offset: usize,
        met_area: &'met mut MethodArea,
    ) -> &'met Class {
        met_area.get_class(offset).unwrap()
    }

    #[inline]
    fn by_name_unchecked<'met>(&self, name: &str, met_area: &'met mut MethodArea) -> &'met Class {
        let index = self.by_name.get(name).unwrap();
        self.by_offset_unchecked(*index, met_area)
    }
}

#[inline]
fn unzip_jar(name: &str) -> Result<ClassFile> {
    let fname = Path::new("./vm/jar/rt.jar");
    let zipfile = File::open(fname).unwrap();

    let mut archive = ZipArchive::new(zipfile)?;

    let mut file = archive
        .by_name(&format!("{name}.class"))
        .or_else(|_| bail!("file: {name} not found in jar."))?;

    let mut contents = Vec::with_capacity(file.size() as usize);
    file.read_to_end(&mut contents).unwrap();
    ClassFile::parse_from_buffer(&contents)
}
