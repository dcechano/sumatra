use std::{num::Wrapping, path::PathBuf};

use anyhow::{bail, Result};

use sumatra_parser::{
    constant::Constant,
    instruction::{ArrayType, Instruction},
    method::Method,
};

use crate::{
    alloc::{heap::Heap, method_area::MethodArea},
    call_frame::CallFrame,
    class::Class,
    compare::Compare,
    data_types::{
        array::{ArrayComp, ArrayRef},
        instance_data::InstanceData,
        object::ObjRef,
        static_data::StaticData,
        value::{RefType, Value},
    },
    lli::{class_manager::ClassManager, response::Response},
    native::{
        native_identifier::NativeIdentifier,
        registry::{NativeMethod, NativeRegistry},
    },
    vm_error,
};

use super::{CLASS_CLASS_ID, CLINIT, MAIN, OBJECT, OBJECT_CLASS_ID, STRING, VM};
// Utility functions are seperated into a different impl block for ease of
// navigation.
impl VM {
    /// Loads the class with the given `name`. DOES NOT initialize the class.
    /// If the class is uninitialized the method will panic!
    pub(crate) fn assume_load(&mut self, name: &str) -> &'static Class {
        match self.class_manager.request(name, &mut self.method_area) {
            Ok(Response::InitReq(_, _)) => panic!("Cannot initialize class in VM::assume_load."),
            Ok(Response::Ready(alloc_index)) => {
                self.method_area.class_data(alloc_index).unwrap().class
            }
            Err(e) => panic!("VM::assume_load had an error: {e:?}"),
            _ => panic!("Manager returned a not found!"),
        }
    }

    pub(crate) fn assume_load_id(&mut self, id: usize) -> &'static Class {
        self.method_area
            .get_class(id)
            .unwrap_or_else(|err| panic!("VM::assume_load_id had and error: {err:?}"))
    }

    /// Create a java.lang.Class object from a `sumatra::Class` represented by
    /// its ID.
    pub(crate) fn create_class_obj(&mut self, instance_class: &'static Class) -> Result<ObjRef> {
        let java_lang_class = self.method_area.get_class(CLASS_CLASS_ID)?;
        let java_lang_object = self.method_area.get_class(OBJECT_CLASS_ID)?;
        Ok(self
            .heap
            .new_class_object(instance_class, java_lang_class, java_lang_object))
    }

    /// Create an instance of java.lang.String manually from a Rust &str.
    /// This method works by calling the java.lang.String constructor with a
    /// char array as an argument.
    pub(crate) fn create_java_string(&mut self, string: &str, intern: bool) -> ObjRef {
        let StaticData {
            class_id: string_id,
            class: string_class,
            ..
        } = self.load_class(STRING).unwrap();
        let object_class = self.method_area.get_class(OBJECT_CLASS_ID).unwrap();

        let mut java_string = self.heap.new_object(InstanceData::new(
            string_class,
            string_id,
            vec![object_class],
        ));

        // The internal private constructor, <init>([CIILjava/lang/Void;)V,
        // reference the empty string literal "". This leads to infite
        // recursion if we try to create the string using the <init>([C)V,
        // constructor, so in this case we just create the object and
        // init fields by hand. Strings undergo compression in the
        // private internal constructor that eventually gets called
        // so we can't do this for non-empty strings.
        if string.is_empty() {
            const VALUE_FIELD: &str = "value";
            const CODER_FIELD: &str = "coder";

            let value_bytes = ArrayRef::new(0, ArrayComp::Byte);
            java_string.set_field(VALUE_FIELD, Value::new_array(value_bytes));
            java_string.set_field(CODER_FIELD, Value::Int(0));
            if intern {
                self.heap().intern_string(string, java_string);
            }
            return java_string;
        }

        let mut char_array = ArrayRef::new(string.len(), ArrayComp::Char);
        string
            .encode_utf16()
            .enumerate()
            .for_each(|(index, byte)| char_array.insert(index, Value::Byte(byte as i8)));
        let char_array = Value::new_array(char_array);

        let constructor = string_class.methods.get("<init>([C)V").unwrap();
        let frame = CallFrame::new(
            string_class,
            constructor,
            &string_class.cp,
            vec![Value::new_object(java_string.clone()), char_array],
        );
        self.frames.push(frame);
        let _ = self.execute_frame().unwrap();
        if intern {
            self.heap().intern_string(string, java_string);
        }
        java_string
    }

    /// Create a java array of java.lang.Stacktrace elements. This
    /// array represents the stack trace of the current thread.
    pub(crate) fn create_stack_trace(&mut self) -> Result<ArrayRef> {
        const STACK_TRACE_ELEMENT: &str = "java/lang/StackTraceElement";
        const CONSTRUCTOR: &str =
            "<init>(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;I)V";

        let len = self.frames.len();
        // create the underlying array
        let mut array = ArrayRef::new(len, ArrayComp::Class(STACK_TRACE_ELEMENT.to_string()));

        // load StackTraceElement.class and Object.class
        let StaticData {
            class_id: ste_class_id,
            class: ste_class,
            ..
        } = self.load_class(STACK_TRACE_ELEMENT).unwrap();
        let object_class = self.method_area.get_class(OBJECT_CLASS_ID).unwrap();

        // Get constructor StackTraceElement.
        let constructor = ste_class.methods.get(CONSTRUCTOR).unwrap();

        // create constructor argumnents.
        let declr_class = self.frame().class.get_name();
        let declr_class_jstring = self.create_java_string(&declr_class, false);

        let method_name = &self.frame().method.name;
        let method_name_jstring = self.create_java_string(method_name, false);

        //TODO implement a way to get the file name.
        let file_name_jstring = self.create_java_string("DUMMY_FILE_NAME", false);
        let line_num = -1;

        // call constructor for each element of the array.
        for i in 0..len {
            // create StackTraceElement instance for constructor.
            let ste_obj = self.heap.new_object(InstanceData::new(
                ste_class,
                ste_class_id,
                vec![object_class],
            ));

            // Call the constructor.
            let frame = CallFrame::new(
                ste_class,
                constructor,
                &ste_class.cp,
                vec![
                    Value::new_object(ste_obj),
                    Value::new_object(declr_class_jstring),
                    Value::new_object(method_name_jstring),
                    Value::new_object(file_name_jstring),
                    Value::Int(line_num),
                ],
            );

            self.frames.push(frame);
            self.execute_frame().unwrap();

            // Insert the newly initialized java object into the array.
            array.insert(i, Value::new_object(ste_obj));
        }
        // Finally, return the array.
        Ok(array)
    }

    /// Create a java.lang.Class object for the given `ObjRef`.
    pub(crate) fn get_class_obj(&self, obj: ObjRef) -> Result<ObjRef> {
        let instance_class_id = obj.get_class_id();
        let instance_class = self.method_area.get_class(instance_class_id)?;
        let java_lang_class = self.method_area.get_class(CLASS_CLASS_ID)?;
        let java_lang_object = self.method_area.get_class(OBJECT_CLASS_ID)?;
        Ok(self.heap.get_class_obj(&instance_class.get_name()))
    }

    /// Construct the local variables array and return it. Assumes there is a
    /// call frame on the stack. If constructing the locals for `main` use
    /// `construct_main_locals`
    pub(crate) fn construct_locals(
        &mut self,
        max_locals: usize,
        num_params: usize,
    ) -> Result<Vec<Value>> {
        if num_params > max_locals {
            bail!("number of method parameters was larger than the max locals.");
        }

        Ok(match (num_params, max_locals) {
            (0, 0) => vec![],
            (0, _) => Value::default_vec(max_locals),
            _ => self.frame_mut().populate_locals(max_locals, num_params),
        })
    }

    /// construct the CallFrame for the main method.
    pub(crate) fn construct_main(&self, c_data: StaticData) -> Result<CallFrame> {
        let main = c_data.class;
        let m_method = find_main(main)?;
        let cp = &main.cp;
        let locals = self.construct_main_locals(&m_method);
        let num_locals = locals.len();
        //TODO implement arguments to pass into main function
        Ok(CallFrame::new(main, m_method, cp, locals))
    }

    /// construct local variables for main method's `CallFrame`.
    pub(crate) fn construct_main_locals(&self, m_method: &Method) -> Vec<Value> {
        Value::default_vec(m_method.code.max_locals as usize)
    }

    /// Construct a method name from the index to the name, and the index to the
    /// descriptor.
    #[inline]
    pub(crate) fn construct_m_name(&self, name_index: usize, descr_index: usize) -> Result<String> {
        let cp = self.frame().cp;
        let name = cp.get_utf8(name_index)?;
        let descr = cp.get_utf8(descr_index)?;
        Ok(format!("{name}{descr}"))
    }

    /// Return a mutable reference to the top most call frame.
    #[inline(always)]
    pub(crate) fn frame_mut(&mut self) -> &mut CallFrame { self.frames.last_mut().unwrap() }

    /// Return a shared reference to the top most call frame.
    #[inline(always)]
    pub(crate) fn frame(&self) -> &CallFrame { self.frames.last().unwrap() }

    /// Retrieves the given method for the given class from the NativeRegistry
    /// if it has been registered.
    pub(crate) fn get_native(
        &mut self,
        class: &'static Class,
        method: &'static Method,
    ) -> Result<NativeMethod> {
        let native_id = NativeIdentifier::new(
            class.get_name(),
            format!("{}{}", method.name, method.descriptor),
        );
        self.native_registry.get(&native_id)
    }

    /// Helper method to invoke a method. Will handle logic for if method is
    /// native or not.
    pub(crate) fn handle_invoke(
        &mut self,
        class: &'static Class,
        method: &'static Method,
    ) -> Result<Option<Value>> {
        if method.is_native() {
            self.invoke_native(class, method)
        } else {
            self.invoke(class, method)
        }
    }

    pub(crate) fn heap(&mut self) -> &mut Heap { &mut self.heap }

    /// Take a static reference to a class and push its '<clinit>'
    /// method as a stack frame to `vm.frames`.
    pub(crate) fn init_class(&mut self, class: &'static Class) -> Result<Option<Value>> {
        // A `<clinit>` is not required by the spec.
        let clinit = match class.methods.get(CLINIT) {
            None => return Ok(None),
            Some(clint) => clint,
        };
        // clinit always takes 0 arguments
        let frame = CallFrame::new(class, clinit, &class.cp, vec![]);
        self.frames.push(frame);
        self.execute_frame()
    }

    /// Helper function to construct a `CallFrame` and invoke a non-native
    /// method.
    pub(crate) fn invoke(
        &mut self,
        class: &'static Class,
        method: &'static Method,
    ) -> Result<Option<Value>> {
        let num_params = if !method.is_static() {
            method.parsed_descriptor.num_params() + 1
        } else {
            method.parsed_descriptor.num_params()
        };

        let max_locals = method.code.max_locals as usize;

        let frame = CallFrame::new(
            class,
            method,
            &class.cp,
            self.construct_locals(max_locals, num_params)?,
        );
        self.frames.push(frame);
        self.execute_frame()
    }

    /// Helper function to construct a `CallFrame` and invoke a native method.
    pub(crate) fn invoke_native(
        &mut self,
        class: &'static Class,
        method: &'static Method,
    ) -> Result<Option<Value>> {
        let native = self.get_native(class, method)?;

        let num_params = method.parsed_descriptor.num_params();
        let arguments = self.construct_locals(num_params, num_params)?;

        let this = if method.is_static() {
            None
        } else {
            let value = self.frame_mut().pop();
            let Value::Ref(RefType::Object(obj)) = value else {
                bail!("Expected a ObjRef in invoke_native but got: {value:?}");
            };
            Some(obj)
        };
        native(self, this, arguments)
    }

    /// Helper function to check if the class `instance` is an instance of the
    /// class `test_class`.
    pub(crate) fn is_instance_of(
        &mut self,
        instance: &'static Class,
        test_class: &'static Class,
    ) -> bool {
        if instance.get_name() == test_class.get_name() {
            return true;
        }
        if instance.is_interface() {
            return self.is_interface_of(instance, test_class);
        }
        self.is_subclass_of(instance, test_class)
    }

    /// Helper function to check if the class `instance` implements
    /// `test_class`. Used primarily in `is_instance_of()`.
    pub(crate) fn is_interface_of(
        &mut self,
        instance: &'static Class,
        test_class: &'static Class,
    ) -> bool {
        todo!()
    }

    /// Helper function to check if the class `instance` is a subclass of
    /// `test_class`. Used primarily in `is_instance_of()`.
    pub(crate) fn is_subclass_of(
        &mut self,
        instance: &'static Class,
        test_class: &'static Class,
    ) -> bool {
        let test_class_name = test_class.get_name();
        if &test_class_name == OBJECT {
            return true;
        }

        let mut curr_class = instance;
        loop {
            let crate_index = curr_class.super_class;
            if crate_index == 0 {
                /*
                crateclass index of 0 means that `curr_class` is java.lang.Object.
                There are no more crateclasses to check and therefore `instance` is not
                instance of `test_class`.
                */
                return false;
            };
            let Constant::Class(crate_name_index) = curr_class.cp.get(crate_index).unwrap() else {
                panic!("Expected a Constant::Class in is_subclass_of.");
            };
            let crate_class_name = curr_class.cp.get_utf8(*crate_name_index).unwrap();
            if &test_class_name == crate_class_name {
                return true;
            }
            curr_class = self.load_class(crate_class_name).unwrap().class;
        }
    }
    /// Helper function to determine if the target of a
    /// `Instruction::InvokeSpecial` instruction is defined in the
    /// crateclass of the current class.
    pub(crate) fn superclass_method(
        &mut self,
        class_index: usize,
        name_and_type_index: usize,
    ) -> Result<bool> {
        //TODO add error handling laid out here: https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-6.html#jvms-6.5.invokespecial
        // There are RuntimeExceptions to be returned if the class method is static,
        // ect.
        let frame = self.frame();
        let crate_index = frame.class.super_class;

        // check if the class named by the method symbolically referenced is the
        // crateclass of the current class.
        if !(class_index == crate_index) {
            return Ok(false);
        }

        // from this point on the class named by the method is the crateclass of the
        // current class. There are other factors to check before returning
        // true.
        if !frame.class.is_super() {
            return Ok(false);
        }

        let (name_index, desc_index, static_data) =
            self.unpack(class_index, name_and_type_index)?;
        let (class, method) = self.to_method_class(name_index, desc_index, &static_data)?;
        if class.is_interface() {
            return Ok(false);
        }

        if method.is_static() {
            //TODO replace with proper handling as indicated above.
            panic!("invokespecial target method was static!");
        }

        if method.name == "<init>".to_string() {
            return Ok(false);
        }

        Ok(true)
    }

    /// Load the class definition specified by `name`. If
    /// the class is found in the `MethodArea`, a `StaticData` object
    /// is returned. This function handles initialization if necessary.
    /// It is important to not call this method unless mutable
    /// access to the fields is strictly necessary. Irresponsible use can
    /// lead to aliasing and undefined behavior.
    pub(crate) fn load_class(&mut self, name: &str) -> Result<StaticData> {
        match self.class_manager.request(name, &mut self.method_area) {
            Ok(Response::InitReq(class, alloc_index)) => {
                // Class obj needs to be created before initializing the class
                // so that class initializers can use the class obj if needed.
                let _ = self.create_class_obj(class)?;
                self.init_class(class)?;
                // `self.method_area.class_data()` loads the class, which is
                // unnecessary here, so we construct a `StaticData` manually.
                let fields = self.method_area.get_mut_fields(alloc_index)?;
                Ok(StaticData::new(alloc_index, class, fields))
            }
            Ok(Response::InitReqArray(array_class, array_class_index, comp_data)) => {
                let _ = self.create_class_obj(array_class)?;
                if let Some((comp_class, _)) = comp_data {
                    let _ = self.create_class_obj(comp_class)?;
                    self.init_class(comp_class)?;
                }
                // There are no static fields on an array class instance. This is just here
                // because, this method requires returning a StaticData obj.
                let fields = self.method_area.get_mut_fields(array_class_index)?;
                Ok(StaticData::new(array_class_index, array_class, fields))
            }
            Ok(Response::Ready(alloc_index)) => self.method_area.class_data(alloc_index),
            Err(e) => bail!(e),
            _ => panic!("Manager returned a not found!"),
        }
    }

    /// Similar to `load_class` except this method just returns the class
    /// without mutable access to its fields.
    pub(crate) fn load_class_immut(&mut self, name: &str) -> Result<&'static Class> {
        match self.class_manager.request(name, &mut self.method_area) {
            Ok(Response::InitReq(class, _)) => {
                // Class obj needs to be created before initializing the class
                // so that class initializers can use the class obj if needed.
                let _ = self.create_class_obj(class)?;
                self.init_class(class)?;
                Ok(class)
            }
            Ok(Response::InitReqArray(array_class, array_class_index, comp_data)) => {
                let _ = self.create_class_obj(array_class)?;
                if let Some((comp_class, _)) = comp_data {
                    let _ = self.create_class_obj(comp_class)?;
                    self.init_class(comp_class)?;
                }
                Ok(array_class)
            }
            Ok(Response::Ready(alloc_index)) => self.method_area.get_class(alloc_index),
            Err(e) => bail!(e),
            _ => panic!("Manager returned a not found!"),
        }
    }

    /// Load the class definition specified by `name` and its crateclasses. This
    /// method is primarily to facilitate Java Object initialization. An
    /// instance needs access to its fields, and the (accessible) fields of
    /// its ancestor classes. The class in the first position of the
    /// returned tuple is the immediately requested class, with the
    /// crateclasses being returned as a `Vec<&'static Class>` in the second
    /// position.
    pub(crate) fn load_hierarchy(&mut self, name: &str) -> Result<InstanceData> {
        let StaticData {
            class_id,
            class: primary,
            ..
        } = self.load_class(name)?; //TODO migrate this line to load_class_immut somehow.
        let mut class = primary;
        // Most classes have at least a handful of classes above them so 8 feels like
        // a prudent capacity that avoids reallocations but avoids a reallocations.
        let mut crate_classes = Vec::with_capacity(8);
        loop {
            let crate_index = class.super_class;
            // crate_index == 0 indicates that the immediate crateclass is java.lang.Object.
            if crate_index == 0 {
                break;
            }

            let Constant::Class(name_index) = class.cp.get(crate_index).unwrap() else {
                bail!("Expected Constant::Class while loading class hierarchy.");
            };
            let crate_name = class.cp.get_utf8(*name_index).unwrap();

            let static_data = self.load_class(crate_name)?;
            crate_classes.push(static_data.class);

            class = static_data.class;
        }
        Ok(InstanceData::new(primary, class_id, crate_classes))
    }

    /// Retrieve a static reference to a method and it's class. `name_index` is
    /// the name of the method, `desc_index` is the descriptor of the
    /// method. `static_data` is a reference to a `StaticData` object
    /// holding the class data.
    pub(crate) fn to_method_class(
        &self,
        name_index: usize,
        desc_index: usize,
        static_data: &StaticData,
    ) -> Result<(&'static Class, &'static Method)> {
        let class = static_data.class;
        let met_name = self.construct_m_name(name_index, desc_index)?;
        let method = class.methods.get(&met_name).unwrap();
        Ok((class, method))
    }

    /// Takes in constant pool indices for the `Constant::Class(class_name)` and
    /// the `Constant::NameAndType` and returns the `name_index`,
    /// `descriptor_index`,  and a `StaticData` of the class pointed
    /// to by `class_name`. The returned StaticData will have a fully
    /// initialize Class.
    pub(crate) fn unpack(
        &mut self,
        class_index: usize,
        name_and_type: usize,
    ) -> Result<(usize, usize, StaticData)> {
        let frame = self.frame();
        let Constant::Class(class_name) = frame.cp.get(class_index).unwrap() else {
            bail!(
                "Class index while executing `get_static` \
                    didn't point to a Class in the constant pool."
            );
        };
        let name = frame.cp.get_utf8(*class_name)?;
        let static_data = self.load_class(name)?;

        let Constant::NameAndType {
            name_index,
            descriptor_index,
        } = self.frame().cp.get(name_and_type).unwrap()
        else {
            bail!(
                "Provided name_and_type_index did not point to a \
                NameAndType constant."
            );
        };
        Ok((*name_index, *descriptor_index, static_data))
    }

    /// Takes in constant pool indices for the `Constant::Class(class_name)` and
    /// the `Constant::NameAndType` and returns a `String` and a `StaticData`.
    /// The `String` represents the field name. The returned `StaticData` is a
    /// wrapper around the class (fully initialized) pointed to by
    /// `class_name`, and a `&'static mut StaticFields` that can be used to
    /// mutate the static fields of the class. Although the mutable ref to
    /// `StaticFields` is `'static`, the ref should never be kept around longer
    /// than the lifetime of the (Rust) stack frame that received it. This
    /// is to avoid running afoul of the Rust aliasing rules.
    pub(crate) fn unpack_f_name(
        &mut self,
        class_index: usize,
        name_and_type: usize,
    ) -> Result<(String, StaticData)> {
        let (name_index, _, data) = self.unpack(class_index, name_and_type)?;
        let f_name = self.frame().cp.get_utf8(name_index)?.into();
        Ok((f_name, data))
    }

    /// Takes in constant pool indices for the `Constant::Class(class_name)` and
    /// the `Constant::NameAndType` and returns a `String` and a `StaticData`.
    /// The `String` represents the method name. The returned `StaticData` is a
    /// wrapper around the class (fully initialized) pointed to by
    /// `class_name`, and a `&'static mut StaticFields` that can be used to
    /// mutate the static fields of the class. Although the mutable ref to
    /// `StaticFields` is `'static`, the ref should never be kept around longer
    /// than the lifetime of the (Rust) stack frame that received it. This
    /// is to avoid running afoul of the Rust aliasing rules.
    pub(crate) fn unpack_m_name(
        &mut self,
        class_index: usize,
        name_and_type: usize,
    ) -> Result<(String, StaticData)> {
        let (name_index, descr_index, data) = self.unpack(class_index, name_and_type)?;
        let name = self.construct_m_name(name_index, descr_index)?;
        Ok((name, data))
    }
}

/// Find the main method of a java class if it exists.
pub(crate) fn find_main(class: &Class) -> Result<&Method> {
    match class.methods.get(MAIN) {
        None => {
            bail!("No main method found.");
        }
        Some(method) => Ok(method),
    }
}
