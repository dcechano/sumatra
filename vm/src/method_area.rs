use std::{
    alloc::{self, Layout},
    marker::PhantomData,
    mem, ptr,
};

use anyhow::{bail, Result};

use crate::{alloc::static_alloc::StaticAlloc, class::Class};

const STATIC_ALLOC_SIZE: isize = mem::size_of::<StaticAlloc>() as isize;
const STATIC_ALLOC_ALIGN: isize = mem::align_of::<StaticAlloc>() as isize;

pub(crate) struct MethodArea {
    data: *mut StaticAlloc,
    next_aligned: *mut StaticAlloc,
    end: usize,
    size: usize,
    owned: PhantomData<StaticAlloc>,
}

impl MethodArea {
    const METHOD_AREA_SIZE: isize = mem::size_of::<u8>() as isize * 1_000_000 * 128;

    pub(crate) fn new() -> Result<Self> { Self::with_size(Self::METHOD_AREA_SIZE) }

    pub(crate) fn with_size(size: isize) -> Result<Self> {
        if size == 0 {
            bail!("Method area cannot be allocated with a 0 size.");
        }
        let layout = Layout::array::<StaticAlloc>((size / STATIC_ALLOC_SIZE) as usize).unwrap();
        // SAFETY: Since the method only takes an isize, which is then used to figure
        // out the number of elements, `layout` is always valid.
        unsafe {
            let data = alloc::alloc(layout) as *mut StaticAlloc;
            if data.is_null() {
                bail!("Allocation error while allocating method area.");
            }

            Ok(Self {
                data,
                next_aligned: data,
                end: data as usize + size as usize,
                size: size as usize,
                owned: PhantomData,
            })
        }
    }

    pub(crate) fn push(&mut self, class: Class) -> Result<usize> {
        if (self.next_aligned as usize + STATIC_ALLOC_SIZE as usize) >= self.end {
            bail!("Method area is out of memory.");
        }

        // SAFETY: We just checked that there is sufficient room in the method area.
        unsafe {
            let index =
                (self.next_aligned as usize - self.data as usize) / STATIC_ALLOC_SIZE as usize;
            ptr::write(self.next_aligned, StaticAlloc::new(class, index));

            let next = self.next_aligned.offset(1);
            // Prevent a segfault
            if (next as usize) < self.end {
                self.next_aligned = next;
            }
            Ok(index)
        }
    }

    pub(crate) fn get_mut(&mut self, index: usize) -> Result<&mut StaticAlloc> {
        unsafe { Ok(&mut *(self.data.add(index))) }
    }

    pub(crate) fn get(&self, index: usize) -> Result<&StaticAlloc> {
        unsafe { Ok(&*(self.data.add(index) as *const StaticAlloc)) }
    }

    pub(crate) fn get_class(&self, index: usize) -> Result<&Class> {
        unsafe { Ok((*(self.data.add(index) as *const StaticAlloc)).get_class()) }
    }

    unsafe fn deallocate_objs(&mut self) {
        let mut cursor = self.data;
        while (cursor as usize) < self.next_aligned as usize {
            let _ = ptr::read(cursor);
            cursor = cursor.add(1);
        }
    }
}

impl Drop for MethodArea {
    fn drop(&mut self) {
        unsafe {
            self.deallocate_objs();
            alloc::dealloc(
                self.data as *mut u8,
                Layout::array::<StaticAlloc>(self.size / STATIC_ALLOC_SIZE as usize).unwrap(),
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

        let offset = met_area.push(object_class.clone()).unwrap();
        let retrieved = met_area.get(offset).unwrap();

        assert_eq!(object_class.this_class, retrieved.get_class().this_class);
        assert_eq!(object_class.super_class, retrieved.get_class().super_class);
        assert_eq!(object_class.attributes, retrieved.get_class().attributes);
        assert_eq!(object_class.interfaces, retrieved.get_class().interfaces);
        assert_eq!(object_class.cp, retrieved.get_class().cp);
        assert_eq!(object_class.fields, retrieved.get_class().fields);
        assert_eq!(object_class.methods, retrieved.get_class().methods);
    }

    #[test]
    #[cfg(miri)]
    fn get_retrieve_miri() {
        let class = init_default_class();
        let cloned = class.clone();

        let mut met_area = MethodArea::new().unwrap();

        let offset = met_area.push(cloned).unwrap();
        let retrieved = met_area.get(offset).unwrap();

        assert_eq!(class.this_class, retrieved.get_class().this_class);
        assert_eq!(class.super_class, retrieved.get_class().super_class);
        assert_eq!(class.attributes, retrieved.get_class().attributes);
        assert_eq!(class.interfaces, retrieved.get_class().interfaces);
        assert_eq!(class.cp, retrieved.get_class().cp);
        assert_eq!(class.fields, retrieved.get_class().fields);
        assert_eq!(class.methods, retrieved.get_class().methods);
    }

    #[test]
    #[cfg(miri)]
    fn get_retrieve_many() {
        const CAP: usize = 2;

        let class = init_default_class();
        let mut met_area = MethodArea::new().unwrap();
        let mut offsets = Vec::with_capacity(CAP);

        for _ in 0..CAP {
            let cloned = class.clone();

            let offset = met_area.push(cloned).unwrap();
            offsets.push(offset);
        }

        for offset in offsets {
            let retrieved = met_area.get(offset).unwrap();
            let retrieved_class = retrieved.get_class();

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
