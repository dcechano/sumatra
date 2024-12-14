use std::{
    alloc::{self, Layout},
    marker::PhantomData,
    mem, ptr,
};

use crate::{
    alloc::static_fields::StaticFields, class::Class, data_types::static_data::StaticData,
    result::Result, vm_error,
};

const STATIC_ALLOC_SIZE: isize = mem::size_of::<StaticFields>() as isize;

const CLASS_SIZE: isize = mem::size_of::<Class>() as isize;

const MIN_SIZE: isize = CLASS_SIZE * 32;

pub(crate) struct MethodArea {
    classes: *mut Class,
    fields: *mut StaticFields,
    len: usize,
    capacity: usize,
    s_marker: PhantomData<StaticFields>,
    c_marker: PhantomData<Class>,
}

impl MethodArea {
    const METHOD_AREA_SIZE: isize = mem::size_of::<u8>() as isize * 1_000_000 * 256;

    pub(crate) fn new() -> Result<Self> { Self::with_size(Self::METHOD_AREA_SIZE) }

    pub(crate) fn with_size(size: isize) -> Result<Self> {
        if size < MIN_SIZE {
            vm_error!("Method area size must be at least: {MIN_SIZE} bytes.");
        }

        let size = size / 2;
        let s_layout = Layout::array::<StaticFields>((size / STATIC_ALLOC_SIZE) as usize).unwrap();
        let c_layout = Layout::array::<Class>((size / CLASS_SIZE) as usize).unwrap();
        // SAFETY: Since the method only takes an isize, which is then used to figure
        // out the number of elements, `s_layout` and `c_layout` are always valid.
        unsafe {
            let s_alloc = alloc::alloc(s_layout) as *mut StaticFields;
            let c_alloc = alloc::alloc(c_layout) as *mut Class;
            if s_alloc.is_null() || c_alloc.is_null() {
                vm_error!("Allocation error while allocating method area.");
            }

            Ok(Self {
                classes: c_alloc,
                fields: s_alloc,
                len: 0,
                capacity: size as usize,
                s_marker: PhantomData,
                c_marker: PhantomData,
            })
        }
    }

    /// Pushes a class to the method area and returns its `class_id`.
    pub(crate) fn push(&mut self, class: Class) -> Result<usize> {
        if self.len == (self.capacity / CLASS_SIZE as usize) {
            vm_error!("Method area is out of memory.");
        }

        // SAFETY: We just checked that there is sufficient room in the method area.
        unsafe {
            let index = self.len;
            ptr::write(self.fields.add(index), StaticFields::new(&class, index));
            ptr::write(self.classes.add(index), class);
            self.len += 1;

            Ok(index)
        }
    }

    pub(crate) fn get_mut_fields(&mut self, class_id: usize) -> Result<&'static mut StaticFields> {
        if class_id >= self.len {
            vm_error!("Invalid class_id {class_id} when retrieving fields!");
        }
        // SAFETY: We confirmed above that this is a valid index into the dynamic array.
        unsafe { Ok(&mut *(self.fields.add(class_id))) }
    }

    #[allow(dead_code)]
    pub(crate) fn get_fields(&self, class_id: usize) -> Result<&'static StaticFields> {
        if class_id >= self.len {
            vm_error!("Invalid class_id {class_id} when retrieving fields!");
        }
        // SAFETY: We confirmed above that this is a valid index into the dynamic array.
        unsafe { Ok(&*(self.fields.add(class_id) as *const StaticFields)) }
    }

    pub(crate) fn get_class(&self, class_id: usize) -> Result<&'static Class> {
        if class_id >= self.len {
            vm_error!("Invalid class_id {class_id} when retrieving class!");
        }
        // SAFETY: We confirmed above that this is a valid index into the dynamic array.
        unsafe { Ok(&*(self.classes.add(class_id))) }
    }

    pub(crate) fn class_data(&mut self, class_id: usize) -> Result<StaticData> {
        let class = self.get_class(class_id)?;
        let fields = self.get_mut_fields(class_id)?;
        Ok(StaticData::new(class_id, class, fields))
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
                Layout::array::<StaticFields>(self.capacity / STATIC_ALLOC_SIZE as usize).unwrap(),
            );
            alloc::dealloc(
                self.classes as *mut u8,
                Layout::array::<Class>(self.capacity / CLASS_SIZE as usize).unwrap(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read, os::unix::fs::MetadataExt};

    use crate::{alloc::method_area::MethodArea, class::Class};
    use sumatra_parser::{
        class_file::ClassFile,
        constant::Constant,
        field::Field,
        flags::{FieldAccessFlags, MethodAccessFlags},
        instruction::Instruction,
        method::Method,
    };

    const OBJECT_FILE: &str = "java/lang/Object.class";
    const JAR_PATH: &str = "../jdk/compiled/java.base/";

    fn get_file() -> ClassFile {
        let mut file = File::open(format!("{JAR_PATH}{OBJECT_FILE}")).unwrap();

        let mut contents = Vec::with_capacity(file.metadata().unwrap().size() as usize);
        file.read_to_end(&mut contents).unwrap();
        ClassFile::parse_from_buffer(&contents).unwrap()
    }

    #[test]
    #[cfg(not(miri))]
    fn get_retrieve() {
        let object_file = get_file();
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
        let field1 = Field {
            parsed_descriptor: "Ljava/lang/Object;".parse().unwrap(),
            name: "foo".to_string(),
            access_flags: FieldAccessFlags::FINAL,
            ..Default::default()
        };

        let field2 = Field {
            parsed_descriptor: "Ljava/lang/Object;".parse().unwrap(),
            name: "bar".to_string(),
            access_flags: FieldAccessFlags::SYNTHETIC,
            ..Default::default()
        };

        let field3 = Field {
            parsed_descriptor: "Ljava/lang/Object;".parse().unwrap(),
            name: "baz".to_string(),
            access_flags: FieldAccessFlags::STATIC,
            ..Default::default()
        };

        [field1, field2, field3]
    }

    #[inline]
    fn methods() -> [Method; 2] {
        let mut method1 = Method {
            name: "getField".to_string(),
            access_flags: MethodAccessFlags::STRICT,
            ..Default::default()
        };

        method1.code.op_code.push(Instruction::LLoad(100));
        method1.code.op_code.push(Instruction::ALoad(42));
        method1.code.op_code.push(Instruction::Ret(20));

        let mut method2 = Method {
            name: "getOut".to_string(),
            access_flags: MethodAccessFlags::FINAL,
            ..Default::default()
        };

        method2.code.op_code.push(Instruction::LLoad(100));
        method2.code.op_code.push(Instruction::ALoad(42));
        method2.code.op_code.push(Instruction::Ret(20));
        [method1, method2]
    }
}
