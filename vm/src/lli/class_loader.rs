use std::{
    fs::File,
    io::Read,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

use sumatra_parser::class_file::ClassFile;

use crate::{
    invalid_class,
    lli::loader::ClassLoader,
    parse_error,
    result::{Error, Result},
    vm_error,
};

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

        let mut file = File::open(self.class_path.join(&name))
            .map_err(|e| Error::VMError(format!("{e:?}")))?;

        let mut contents = Vec::with_capacity(file.metadata().unwrap().size() as usize);
        file.read_to_end(&mut contents).unwrap();
        match ClassFile::parse_from_buffer(&contents) {
            Ok(classfile) => Ok(classfile),
            Err(e) => {
                parse_error!("Error parsing class file {name}: {e}");
            }
        }
    }
}

impl ClassLoader for BootstrapLoader {
    fn get(&mut self, name: &str) -> Result<ClassFile> { self.get_file(name) }
}
