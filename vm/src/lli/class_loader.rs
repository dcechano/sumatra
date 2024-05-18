use std::{
    fs::File,
    io::Read,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail, Result};
use zip::ZipArchive;

use sumatra_parser::class_file::ClassFile;

use crate::lli::loader::ClassLoader;

pub(super) struct BootstrapLoader {
    class_path: PathBuf,
}

impl BootstrapLoader {
    pub(crate) fn new(class_path: PathBuf) -> Self { Self { class_path } }

    fn get_file(&self, name: &str) -> Result<ClassFile> {
        let name = if !name.ends_with(".class") {
            format!("{name}.class")
        } else {
            name.into()
        };

        let mut file = File::open(self.class_path.join(&name))?;

        let mut contents = Vec::with_capacity(file.metadata().unwrap().size() as usize);
        file.read_to_end(&mut contents).unwrap();
        match ClassFile::parse_from_buffer(&contents) {
            Ok(classfile) => Ok(classfile),
            Err(e) => Err(anyhow!("Error parsing class file {name}: {e}")),
        }
    }
}

impl ClassLoader for BootstrapLoader {
    fn get(&mut self, name: &str) -> Result<ClassFile> { self.get_file(name) }
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
