use std::{fs::File, io::Read, path::Path};

use anyhow::{bail, Result};
use zip::ZipArchive;

use sumatra_parser::class_file::ClassFile;

use crate::lli::loader::ClassLoader;

pub(super) struct BootstrapLoader;

impl ClassLoader for BootstrapLoader {
    fn get(&mut self, name: &str) -> Result<ClassFile> { unzip_jar(name) }
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
