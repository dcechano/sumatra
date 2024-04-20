use anyhow::Result;

use sumatra_parser::class_file::ClassFile;

pub(super) trait ClassLoader {
    fn get(&mut self, name: &str) -> Result<ClassFile>;
}
