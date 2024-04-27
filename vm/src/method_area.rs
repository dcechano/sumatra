use std::{
    alloc::{self, Layout},
    marker::PhantomData,
    mem, ptr,
};

use anyhow::{bail, Result};

use crate::{alloc::static_alloc::StaticAlloc, class::Class};

const STATIC_ALLOC_SIZE: isize = mem::size_of::<StaticAlloc>() as isize;
const STATIC_ALLOC_ALIGN: isize = mem::align_of::<StaticAlloc>() as isize;

const CLASS_SIZE: isize = mem::size_of::<Class>() as isize;
const CLASS_ALIGN: isize = mem::align_of::<Class>() as isize;

const MIN_SIZE: isize = CLASS_SIZE * 32;

pub(crate) struct MethodArea {
    classes: *mut Class,
    fields: *mut StaticAlloc,
    len: usize,
    //TODO rename to cap or capacity to match Vec API
    // since this struct is effectively a Vec. Seems more idiomatic, ig.
    size: usize,
    s_marker: PhantomData<StaticAlloc>,
    c_marker: PhantomData<Class>,
}

impl MethodArea {
    const METHOD_AREA_SIZE: isize = mem::size_of::<u8>() as isize * 1_000_000 * 256;

    pub(crate) fn new() -> Result<Self> { Self::with_size(Self::METHOD_AREA_SIZE) }

    pub(crate) fn with_size(size: isize) -> Result<Self> {
        if size < MIN_SIZE {
            bail!("Method area size must be at least: {MIN_SIZE} bytes.");
        }

        let size = size / 2;
        let s_layout = Layout::array::<StaticAlloc>((size / STATIC_ALLOC_SIZE) as usize).unwrap();
        let c_layout = Layout::array::<Class>((size / CLASS_SIZE) as usize).unwrap();
        // SAFETY: Since the method only takes an isize, which is then used to figure
        // out the number of elements, `s_layout` and `c_layout` are always valid.
        unsafe {
            let s_alloc = alloc::alloc(s_layout) as *mut StaticAlloc;
            let c_alloc = alloc::alloc(c_layout) as *mut Class;
            if s_alloc.is_null() || c_alloc.is_null() {
                bail!("Allocation error while allocating method area.");
            }

            Ok(Self {
                classes: c_alloc,
                fields: s_alloc,
                len: 0,
                size: size as usize,
                s_marker: PhantomData,
                c_marker: PhantomData,
            })
        }
    }

    pub(crate) fn push(&mut self, class: Class) -> Result<usize> {
        if self.len == (self.size / CLASS_SIZE as usize) {
            bail!("Method area is out of memory.");
        }

        // SAFETY: We just checked that there is sufficient room in the method area.
        unsafe {
            let index = self.len;
            ptr::write(self.fields.add(index), StaticAlloc::new(&class, index));
            ptr::write(self.classes.add(index), class);
            self.len += 1;

            Ok(index)
        }
    }

    pub(crate) fn get_mut_fields(&mut self, index: usize) -> Result<&'static mut StaticAlloc> {
        unsafe { Ok(&mut *(self.fields.add(index))) }
    }

    pub(crate) fn get_fields(&self, index: usize) -> Result<&'static StaticAlloc> {
        unsafe { Ok(&*(self.fields.add(index) as *const StaticAlloc)) }
    }

    pub(crate) fn get_class(&self, index: usize) -> Result<&'static Class> {
        unsafe { Ok(&*(self.classes.add(index))) }
    }

    unsafe fn deallocate_objs(&mut self) {
        let c_ptr = self.classes;
        let s_ptr = self.fields;
        for index in 0..self.len {
            let _ = ptr::read(c_ptr.add(index));
            let _ = ptr::read(s_ptr.add(index));
        }
    }
}

impl Drop for MethodArea {
    fn drop(&mut self) {
        unsafe {
            self.deallocate_objs();
            alloc::dealloc(
                self.fields as *mut u8,
                Layout::array::<StaticAlloc>(self.size / STATIC_ALLOC_SIZE as usize).unwrap(),
            );
            alloc::dealloc(
                self.classes as *mut u8,
                Layout::array::<Class>(self.size / CLASS_SIZE as usize).unwrap(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read, path::Path};

    use anyhow::bail;
    use zip::ZipArchive;

    use sumatra_parser::{
        class_file::ClassFile,
        constant::Constant,
        field::Field,
        flags::{FieldAccessFlags, MethodAccessFlags},
        instruction::Instruction,
        method::Method,
    };

    use crate::{class::Class, method_area::MethodArea};

    const OBJECT_FILE: &'static str = "java/lang/Object";
    const JAR_PATH: &'static str = "./jar/rt.jar";

    #[inline]
    fn unzip_jar(name: &str) -> ClassFile {
        let fname = Path::new(JAR_PATH);
        let zipfile = File::open(fname).unwrap();

        let mut archive = ZipArchive::new(zipfile).unwrap();

        let mut file = archive
            .by_name(&format!("{name}.class"))
            .or_else(|_| bail!("file: {name} not found in jar."))
            .unwrap();

        let mut contents = Vec::with_capacity(file.size() as usize);
        file.read_to_end(&mut contents).unwrap();
        ClassFile::parse_from_buffer(&contents).unwrap()
    }

    #[test]
    #[cfg(not(miri))]
    fn get_retrieve() {
        let object_file = unzip_jar(OBJECT_FILE);
        let object_class = Class::from(object_file);

        let mut met_area = MethodArea::new().unwrap();

        let index = met_area.push(object_class.clone()).unwrap();
        let retrieved = met_area.get_class(index).unwrap();

        assert_eq!(object_class.this_class, retrieved.this_class);
        assert_eq!(object_class.super_class, retrieved.super_class);
        assert_eq!(object_class.attributes, retrieved.attributes);
        assert_eq!(object_class.interfaces, retrieved.interfaces);
        assert_eq!(object_class.cp, retrieved.cp);
        assert_eq!(object_class.fields, retrieved.fields);
        assert_eq!(object_class.methods, retrieved.methods);
    }

    #[test]
    fn get_retrieve_miri() {
        let class = init_default_class();
        let cloned = class.clone();

        let mut met_area = MethodArea::new().unwrap();

        let index = met_area.push(cloned).unwrap();
        let retrieved = met_area.get_class(index).unwrap();

        assert_eq!(class.this_class, retrieved.this_class);
        assert_eq!(class.super_class, retrieved.super_class);
        assert_eq!(class.attributes, retrieved.attributes);
        assert_eq!(class.interfaces, retrieved.interfaces);
        assert_eq!(class.cp, retrieved.cp);
        assert_eq!(class.fields, retrieved.fields);
        assert_eq!(class.methods, retrieved.methods);
    }

    #[test]
    fn get_retrieve_many() {
        const CAP: usize = 2;

        let class = init_default_class();
        let mut met_area = MethodArea::new().unwrap();
        let mut indices = Vec::with_capacity(CAP);

        for _ in 0..CAP {
            let cloned = class.clone();

            let index = met_area.push(cloned).unwrap();
            indices.push(index);
        }

        for index in indices {
            let retrieved_class = met_area.get_class(index).unwrap();

            assert_eq!(class.this_class, retrieved_class.this_class);
            assert_eq!(class.this_class, retrieved_class.this_class);
            assert_eq!(class.super_class, retrieved_class.super_class);
            assert_eq!(class.attributes, retrieved_class.attributes);
            assert_eq!(class.interfaces, retrieved_class.interfaces);
            assert_eq!(class.cp, retrieved_class.cp);
            assert_eq!(class.fields, retrieved_class.fields);
            assert_eq!(class.methods, retrieved_class.methods);
        }
    }

    #[inline]
    #[allow(clippy::field_reassign_with_default)]
    fn init_default_class() -> Class {
        let mut object_class = Class::default();
        object_class.this_class = 1;
        object_class.cp.push(Constant::Dummy);
        object_class.cp.push(Constant::Class(2));
        object_class
            .cp
            .push(Constant::UTF8("DefaultClass".to_string()));
        for field in fields() {
            object_class.fields.insert(field.name.clone(), field);
        }

        for method in methods() {
            object_class.methods.insert(method.name.clone(), method);
        }

        object_class
    }

    #[inline]
    fn fields() -> [Field; 3] {
        let mut field1 = Field::default();
        field1.name = "foo".to_string();
        field1.access_flags = FieldAccessFlags::FINAL;

        let mut field2 = Field::default();
        field2.name = "bar".to_string();
        field2.access_flags = FieldAccessFlags::SYNTHETIC;

        let mut field3 = Field::default();
        field3.name = "baz".to_string();
        field3.access_flags = FieldAccessFlags::STATIC;

        [field1, field2, field3]
    }

    #[inline]
    fn methods() -> [Method; 2] {
        let mut method1 = Method::default();
        method1.name = "getField".to_string();
        method1.access_flags = MethodAccessFlags::STRICT;
        method1.code.op_code.push(Instruction::LLoad(100));
        method1.code.op_code.push(Instruction::ALoad(42));
        method1.code.op_code.push(Instruction::Ret(20));

        let mut method2 = Method::default();
        method2.name = "getOut".to_string();
        method2.access_flags = MethodAccessFlags::FINAL;
        method2.code.op_code.push(Instruction::LLoad(100));
        method2.code.op_code.push(Instruction::ALoad(42));
        method2.code.op_code.push(Instruction::Ret(20));
        [method1, method2]
    }
}
