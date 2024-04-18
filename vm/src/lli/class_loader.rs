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

use crate::{class::Class, lli::response::Response, method_area::MethodArea};

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
    pub(crate) fn resolve_and_index(
        &mut self,
        name: &str,
        met_area: &mut MethodArea,
    ) -> Result<Response> {
        if !self.by_name.contains_key(name) {
            let class_file = match unzip_jar(name) {
                Ok(class_file) => class_file,
                Err(_) => {
                    //TODO for now assume an error means the class was not found and
                    // the ClassManager needs to ask another ClassLoader
                    return Ok(Response::NonFound);
                }
            };
            let class = Class::from(&class_file);
            match self.store_class(class, met_area) {
                // since the class had to retrieved and stored it has not been initialized.
                Ok((class, index)) => Ok(Response::InitReq(class, index)),
                Err(_) => {
                    bail!("MethodArea allocation error.")
                }
            }
        } else {
            Ok(Response::Ready(*self.by_name.get(name).unwrap()))
        }
    }

    //FIXME In light of implementing a ClassManager, perhaps it is no longer
    // appropriate for the ClassLoader to keep track ClassIds.
    #[inline]
    fn store_class(
        &mut self,
        class: Class,
        met_area: &mut MethodArea,
    ) -> Result<(&'static Class, usize)> {
        let name = class.get_name();
        let index = met_area.push(class)?;
        let id = self.count.load(Ordering::SeqCst);
        self.count.store(id + 1, Ordering::SeqCst);
        self.by_id.insert(id, index);
        self.by_name.insert(name, index);
        Ok((met_area.get_class(index)?, index))
    }

    // #[inline]
    // pub(crate) fn resolve<'met>(
    //     &mut self,
    //     name: &str,
    //     met_area: &'met mut MethodArea,
    // ) -> Result<&'met Class> {
    //     let index = self.resolve_and_index(name, met_area)?;
    //     met_area.get_class(index)
    // }

    // #[inline]
    // pub(crate) fn resolve_static<'met>(
    //     &mut self,
    //     name: &str,
    //     met_area: &'met mut MethodArea,
    // ) -> Result<&'met StaticAlloc> {
    //     let index = self.resolve_and_index(name, met_area)?;
    //     met_area.get(index)
    // }
    //
    // pub(crate) fn resolve_static_mut<'met>(
    //     &mut self,
    //     name: &str,
    //     met_area: &'met mut MethodArea,
    // ) -> Result<&'met mut StaticAlloc> {
    //     let index = self.resolve_and_index(name, met_area)?;
    //     met_area.get_mut(index)
    // }
    //
    // #[inline]
    // pub(crate) fn by_name<'met>(
    //     &self,
    //     name: &str,
    //     met_area: &'met mut MethodArea,
    // ) -> Result<&'met Class> {
    //     match self.by_name.get(name) {
    //         None => {
    //             bail!("No class found for name: {name}");
    //         }
    //         Some(index) => self.by_offset(*index, met_area),
    //     }
    // }
    //
    // #[inline]
    // pub(crate) fn by_offset<'met>(
    //     &self,
    //     offset: usize,
    //     met_area: &'met mut MethodArea,
    // ) -> Result<&'met Class> {
    //     met_area.get_class(offset)
    // }
    //
    // #[inline]
    // fn by_offset_unchecked<'met>(
    //     &self,
    //     offset: usize,
    //     met_area: &'met mut MethodArea,
    // ) -> &'met Class {
    //     met_area.get_class(offset).unwrap()
    // }
    //
    // #[inline]
    // fn by_name_unchecked<'met>(&self, name: &str, met_area: &'met mut MethodArea)
    // -> &'met Class {     let index = self.by_name.get(name).unwrap();
    //     self.by_offset_unchecked(*index, met_area)
    // }
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
