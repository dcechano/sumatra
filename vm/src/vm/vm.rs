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
    vm::{CLASS, CLASS_CLASS_ID, OBJECT, OBJECT_CLASS_ID, STRING, SYSTEM, SYSTEM_CLASS_ID},
    vm_error,
};

use super::DEFAULT_VEC_SIZE;
pub struct VM {
    pub(super) frames: Vec<CallFrame>,
    pub(super) method_area: MethodArea,
    pub(super) heap: Heap,
    pub(super) class_manager: ClassManager,
    pub(crate) native_registry: NativeRegistry,
}

impl VM {
    /// create the VM.
    pub fn init(jdk: PathBuf, c_path: PathBuf) -> Self {
        let mut vm = VM::new(jdk, c_path);

        vm.bootstrap_classes();
        panic!("Bootstrapping complete!");
        vm
    }

    fn new(jdk: PathBuf, c_path: PathBuf) -> Self {
        let method_area = match MethodArea::new() {
            Ok(method_area) => method_area,
            Err(_) => panic!("Memory Allocation Error while starting Sumatra VM"),
        };
        Self {
            frames: Vec::with_capacity(DEFAULT_VEC_SIZE),
            method_area,
            heap: Heap::new(),
            class_manager: ClassManager::new(jdk, c_path),
            native_registry: NativeRegistry::new(),
        }
    }

    fn bootstrap_classes(&mut self) {
        // First we must load the primitive classes because the core
        // classes (Object.class, Class.class, ect) may depenend on them.
        // We DO NOT create their Class.class instance (int.class, ect.) yet
        // because that would depend on Class.class being loaded.
        let prim_classes = self.class_manager.init_prim_classes(&mut self.method_area);
        // Load java/lang/Object so that its class_id is always 8.
        // This makes it easy to make sure all arrays have Object as its class
        // on the Heap.
        let (java_lang_obj, java_lang_obj_id) = self.bootstrap_load(OBJECT);
        let (java_lang_system, java_lang_system_id) = self.bootstrap_load(SYSTEM);
        let (java_lang_class, java_lang_class_id) = self.bootstrap_load(CLASS);
        let (java_lang_string, java_lang_string_id) = self.bootstrap_load(STRING);

        debug_assert!(java_lang_obj_id == OBJECT_CLASS_ID);
        debug_assert!(java_lang_system_id == SYSTEM_CLASS_ID);
        debug_assert!(java_lang_class_id == CLASS_CLASS_ID);

        // NOW create the Class.java objects, since it is now safe.
        let _ = self.create_class_obj(java_lang_obj).unwrap();
        let _ = self.create_class_obj(java_lang_system).unwrap();
        let _ = self.create_class_obj(java_lang_class).unwrap();
        let _ = self.create_class_obj(java_lang_string).unwrap();

        // NOW we create the Class.class instance.
        prim_classes.into_iter().for_each(|class| {
            let _ = self.create_class_obj(class).unwrap();
        });

        // This calls the special initialization method in System.java that is used to
        // initialize the static final InputStream and OutputStream.
        let init_phase1 = java_lang_system.methods.get("initPhase1()V").unwrap();
        let frame = CallFrame::new(java_lang_system, init_phase1, &java_lang_system.cp, vec![]);
        // TODO perhaps do not unwrap but return a VMError when implemented
        self.invoke(java_lang_system, init_phase1).unwrap();
    }

    /*
        Very similar to VM.load_class() except vm.load_class() will attempt to create
        the Class.java instance if initialization is required. Since the java/lang/Class.class
        hasn't been loaded yet, the program will crash. So here we do it in two steps
        just this one time.
    */
    fn bootstrap_load(&mut self, name: &str) -> (&'static Class, usize) {
        match self.class_manager.request(name, &mut self.method_area) {
            Ok(Response::InitReq(class, class_id)) => {
                self.init_class(class).unwrap();
                (class, class_id)
            }
            Ok(_) => panic!("Class already loaded! Improper use of bootstrap_load()."),
            Err(e) => panic!("Error while initializing VM in bootstrap_load(): {:?}", e),
        }
    }

    /// Entry point of the JVM. `c_entry` is the initial class loaded
    /// by the VM to spin up the Java program.
    pub fn run(&mut self, c_entry: &str) -> Result<()> {
        let c_data = if !c_entry.ends_with(".class") {
            self.load_class(&format!("{c_entry}{}", ".class"))?
        } else {
            self.load_class(c_entry)?
        };
        let frame = self.construct_main(c_data)?;
        self.frames.push(frame);
        self.execute_frame()?;
        Ok(())
    }

    /// Executes the top most `CallFrame` on the stack. Frame
    /// gets popped off the stack before method returns.
    pub(super) fn execute_frame(&mut self) -> Result<Option<Value>> {
        let code = &self.frame().method.code;
        let op_code = &code.op_code;
        let name: &str = self.frame().method.name.as_ref();
        let indents = self.frames.len();
        // if name != "<clinit>" && name != "<init>" {
        println!(
            "\n{}Executing method: {}{} in class: {}",
            "\t".repeat(indents),
            self.frame().method.name,
            self.frame().method.descriptor,
            self.frame().class.get_name()
        );
        //println!(
        //    "{}CURRENT STACK: {:?}",
        //    "\t".repeat(indents + 1),
        //    self.frame().stack
        //);
        //println!(
        //    "{}CURRENT LOCALS: {:?}",
        //    "\t".repeat(indents + 1),
        //    self.frame().locals
        // );

        // }
        while let Some(code) = op_code.get(self.frame().pc) {
            let name: &str = self.frame().method.name.as_ref();
            // if name != "<clinit>" && name != "<init>" {
            println!("{}{code:?}", "\t".repeat(indents),);
            // }
            match code {
                Instruction::AaLoad => self.aaload()?,
                Instruction::AaStore => self.aastore()?,
                Instruction::AConstNull => self.frame_mut().stack.push(Value::Null),
                Instruction::ALoad(index) => self.a_load(*index)?,
                Instruction::ALoad0 => self.a_load(0)?,
                Instruction::ALoad1 => self.a_load(1)?,
                Instruction::ALoad2 => self.a_load(2)?,
                Instruction::ALoad3 => self.a_load(3)?,
                Instruction::ANewArray(class_index) => self.anew_array(*class_index)?,
                Instruction::AReturn => return Ok(Some(self.return_val())),
                Instruction::ArrayLength => self.array_length()?,
                Instruction::AStore(index) => self.a_store_n(*index)?,
                Instruction::AStore0 => self.a_store_n(0)?,
                Instruction::AStore1 => self.a_store_n(1)?,
                Instruction::AStore2 => self.a_store_n(2)?,
                Instruction::AStore3 => self.a_store_n(3)?,
                Instruction::AThrow => self.a_throw()?,
                Instruction::BaLoad => todo!(),
                Instruction::BaStore => self.bastore()?,
                Instruction::BiPush(byte) => self.frame_mut().stack.push(Value::Int(*byte as i32)),
                Instruction::CaLoad => self.caload()?,
                Instruction::CaStore => self.castore()?,
                Instruction::Checkcast(index) => self.check_cast(*index)?,
                Instruction::D2F => todo!(),
                Instruction::D2I => todo!(),
                Instruction::D2L => todo!(),
                Instruction::DAdd => self.dadd()?,
                Instruction::DaLoad => todo!(),
                Instruction::DaStore => todo!(),
                Instruction::Dcmpg => todo!(),
                Instruction::Dcmpl => todo!(),
                Instruction::DConst0 => self.frame_mut().push(Value::Double(0f64)),
                Instruction::DConst1 => self.frame_mut().push(Value::Double(1f64)),
                Instruction::DDiv => todo!(),
                Instruction::DLoad(local_index) => self.dload_n(*local_index)?,
                Instruction::DLoad0 => self.dload_n(0)?,
                Instruction::DLoad1 => self.dload_n(1)?,
                Instruction::DLoad2 => self.dload_n(2)?,
                Instruction::DLoad3 => self.dload_n(3)?,
                Instruction::DMul => todo!(),
                Instruction::DNeg => todo!(),
                Instruction::DRem => todo!(),
                Instruction::DReturn => return Ok(Some(self.return_val())),
                Instruction::DStore(local_index) => self.dstore_n(*local_index)?,
                Instruction::DStore0 => self.dstore_n(0)?,
                Instruction::DStore1 => self.dstore_n(1)?,
                Instruction::DStore2 => self.dstore_n(2)?,
                Instruction::DStore3 => self.dstore_n(3)?,
                Instruction::DSub => todo!(),
                Instruction::Dup => self.dup()?,
                Instruction::DupX1 => self.dup_x1()?,
                Instruction::DupX2 => self.dup_x2()?,
                Instruction::Dup2 => todo!(),
                Instruction::Dup2X1 => todo!(),
                Instruction::Dup2X2 => todo!(),
                Instruction::F2D => self.f2d()?,
                Instruction::F2I => self.f2i()?,
                Instruction::F2L => self.f2l()?,
                Instruction::FAdd => self.fadd()?,
                Instruction::FaLoad => todo!(),
                Instruction::FaStore => todo!(),
                Instruction::Fcmpg => self.fcmp(Compare::GreaterThan)?,
                Instruction::Fcmpl => self.fcmp(Compare::LessThan)?,
                Instruction::FConst0 => self.frame_mut().push(Value::Float(0f32)),
                Instruction::FConst1 => self.frame_mut().push(Value::Float(1f32)),
                Instruction::FConst2 => self.frame_mut().push(Value::Float(2f32)),
                Instruction::FDiv => todo!(),
                Instruction::FLoad(index) => self.fload_n(*index)?,
                Instruction::FLoad0 => self.fload_n(0)?,
                Instruction::FLoad1 => self.fload_n(1)?,
                Instruction::FLoad2 => self.fload_n(2)?,
                Instruction::FLoad3 => self.fload_n(3)?,
                Instruction::FMul => self.fmul()?,
                Instruction::FNeg => todo!(),
                Instruction::FRem => todo!(),
                Instruction::FReturn => return Ok(Some(self.return_val())),
                Instruction::FStore(index) => self.fstore_n(*index)?,
                Instruction::FStore0 => self.fstore_n(0)?,
                Instruction::FStore1 => self.fstore_n(1)?,
                Instruction::FStore2 => self.fstore_n(2)?,
                Instruction::FStore3 => self.fstore_n(3)?,
                Instruction::FSub => todo!(),
                Instruction::GetField(field_index) => self.get_field(*field_index)?,
                Instruction::GetStatic(index) => self.get_static(*index)?,
                Instruction::GoTo(instr) => {
                    self.frame_mut().pc = *instr;
                    continue;
                }
                Instruction::GoToW(_) => todo!(),
                Instruction::I2B => self.i2b()?,
                Instruction::I2C => self.i2c()?,
                Instruction::I2D => self.i2d()?,
                Instruction::I2F => self.i2f()?,
                Instruction::I2L => self.i2l()?,
                Instruction::I2S => self.i2s()?,
                Instruction::IAdd => self.iadd()?,
                Instruction::IaLoad => todo!(),
                Instruction::IAnd => self.iand()?,
                Instruction::IaStore => self.iastore()?,
                Instruction::IConstM1 => self.iconst_n(-1),
                Instruction::IConst0 => self.iconst_n(0),
                Instruction::IConst1 => self.iconst_n(1),
                Instruction::IConst2 => self.iconst_n(2),
                Instruction::IConst3 => self.iconst_n(3),
                Instruction::IConst4 => self.iconst_n(4),
                Instruction::IConst5 => self.iconst_n(5),
                Instruction::IDiv => self.idiv()?,
                Instruction::IfAcmpeq(index) => {
                    if self.ifcmp(*index, Compare::Equal) {
                        continue;
                    }
                }
                Instruction::IfAcmpne(index) => {
                    if self.ifcmp(*index, Compare::NotEqual) {
                        continue;
                    }
                }
                Instruction::IfIcmpeq(index) => {
                    if self.ifcmp(*index, Compare::Equal) {
                        continue;
                    }
                }
                Instruction::IfIcmpne(index) => {
                    if self.ifcmp(*index, Compare::NotEqual) {
                        continue;
                    }
                }
                Instruction::IfIcmplt(index) => {
                    if self.ifcmp(*index, Compare::LessThan) {
                        continue;
                    }
                }
                Instruction::IfIcmpge(index) => {
                    if self.ifcmp(*index, Compare::GreaterOrEqual) {
                        continue;
                    }
                }
                Instruction::IfIcmpgt(index) => {
                    if self.ifcmp(*index, Compare::GreaterThan) {
                        continue;
                    }
                }
                Instruction::IfIcmple(index) => {
                    if self.ifcmp(*index, Compare::LessOrEqual) {
                        continue;
                    }
                }
                Instruction::Ifeq(index) => {
                    if self.if_cond(*index, Compare::Equal) {
                        continue;
                    }
                }
                Instruction::Ifne(index) => {
                    if self.if_cond(*index, Compare::NotEqual) {
                        continue;
                    }
                }
                Instruction::Iflt(index) => {
                    if self.if_cond(*index, Compare::LessThan) {
                        continue;
                    }
                }
                Instruction::Ifge(index) => {
                    if self.if_cond(*index, Compare::GreaterOrEqual) {
                        continue;
                    }
                }
                Instruction::Ifgt(index) => {
                    if self.if_cond(*index, Compare::GreaterThan) {
                        continue;
                    }
                }
                Instruction::Ifle(index) => {
                    if self.if_cond(*index, Compare::LessOrEqual) {
                        continue;
                    }
                }
                Instruction::IfNonNull(index) => {
                    if self.if_nonnull(*index) {
                        continue;
                    }
                }
                Instruction::IfNull(index) => {
                    if self.if_null(*index) {
                        continue;
                    }
                }
                Instruction::Iinc(index, inc) => self.iinc(*index, *inc as i32),
                Instruction::ILoad(local_index) => self.iload_n(*local_index)?,
                Instruction::ILoad0 => self.iload_n(0)?,
                Instruction::ILoad1 => self.iload_n(1)?,
                Instruction::ILoad2 => self.iload_n(2)?,
                Instruction::ILoad3 => self.iload_n(3)?,
                Instruction::IMul => todo!(),
                Instruction::INeg => todo!(),
                Instruction::InstanceOf(index) => self.instance_of(*index)?,
                Instruction::InvokeDynamic(index, _, _) => todo!(),
                Instruction::InvokeInterface(_, _, _) => todo!(),
                Instruction::InvokeSpecial(method_index) => {
                    if let Some(value) = self.invoke_special(*method_index)? {
                        self.frame_mut().push(value);
                    }
                }
                Instruction::InvokeStatic(method_index) => {
                    if let Some(value) = self.invoke_static(*method_index)? {
                        self.frame_mut().push(value);
                    }
                }
                Instruction::InvokeVirtual(method_index) => {
                    if let Some(value) = self.invoke_virtual(*method_index)? {
                        self.frame_mut().push(value);
                    }
                }
                Instruction::IOr => todo!(),
                Instruction::IRem => self.irem()?,
                Instruction::IReturn => return Ok(Some(self.return_val())),
                Instruction::IShL => self.ishl()?,
                Instruction::IShR => self.ishr()?,
                Instruction::IStore(local_index) => self.istore_n(*local_index)?,
                Instruction::IStore0 => self.istore_n(0)?,
                Instruction::IStore1 => self.istore_n(1)?,
                Instruction::IStore2 => self.istore_n(2)?,
                Instruction::IStore3 => self.istore_n(3)?,
                Instruction::ISub => self.isub()?,
                Instruction::IuShR => self.iushr()?,
                Instruction::IxOr => self.ixor()?,
                Instruction::Jsr(_) => todo!(),
                Instruction::JsrW(_) => todo!(),
                Instruction::L2D => todo!(),
                Instruction::L2F => todo!(),
                Instruction::L2I => self.l2i()?,
                Instruction::LAdd => self.ladd()?,
                Instruction::LaLoad => todo!(),
                Instruction::LAnd => self.land()?,
                Instruction::LaStore => todo!(),
                Instruction::Lcmp => self.lcmp()?,
                Instruction::LConst0 => self.lconst_n(0)?,
                Instruction::LConst1 => self.lconst_n(1)?,
                Instruction::Ldc(index) => self.load_const(*index)?,
                Instruction::LdcW(index) => self.load_const(*index)?,
                Instruction::Ldc2W(index) => self.load_const2(*index)?,
                Instruction::LDiv => todo!(),
                Instruction::LLoad(index) => self.lload(*index)?,
                Instruction::LLoad0 => self.lload(0)?,
                Instruction::LLoad1 => self.lload(1)?,
                Instruction::LLoad2 => self.lload(2)?,
                Instruction::LLoad3 => self.lload(3)?,
                Instruction::LMul => todo!(),
                Instruction::LNeg => todo!(),
                Instruction::LookUpSwitch { .. } => todo!(),
                Instruction::LOr => todo!(),
                Instruction::LRem => todo!(),
                Instruction::LReturn => return Ok(Some(self.return_val())),
                Instruction::LShL => self.lshl()?,
                Instruction::LShR => self.lshr()?,
                Instruction::LStore(index) => self.lstore(*index)?,
                Instruction::LStore0 => self.lstore(0)?,
                Instruction::LStore1 => self.lstore(1)?,
                Instruction::LStore2 => self.lstore(2)?,
                Instruction::LStore3 => self.lstore(3)?,
                Instruction::LSub => self.lsub()?,
                Instruction::LuShR => todo!(),
                Instruction::LxOr => todo!(),
                Instruction::MonitorEnter => self.monitor_enter()?,
                Instruction::MonitorExit => self.monitor_exit()?,
                Instruction::MultiaNewArray(_, _) => todo!(),
                Instruction::New(class_index) => self.new_obj(*class_index)?,
                Instruction::NewArray(array_type) => {
                    self.new_array(ArrayComp::try_from(*array_type)?)?
                }
                Instruction::Nop => todo!(),
                Instruction::Pop => self.pop()?,
                Instruction::Pop2 => self.pop2()?,
                Instruction::PutField(field_index) => self.put_field(*field_index)?,
                Instruction::PutStatic(field_index) => self.put_static(*field_index)?,
                Instruction::Ret(_) => todo!(),
                Instruction::Return => break,
                Instruction::SaLoad => todo!(),
                Instruction::SaStore => todo!(),
                Instruction::SiPush(byte) => self.sipush(*byte)?,
                Instruction::Swap => todo!(),
                Instruction::TableSwitch { .. } => todo!(),
                Instruction::Wide(winstr) => todo!(),
            }
            // if name != "<clinit>" && name != "<init>" {
            //println!(
            //    "{}Stack: {:?}",
            //    "\t".repeat(indents + 1),
            //    self.frame().stack
            //);
            //println!(
            //    "{}Locals: {:?}",
            //    "\t".repeat(indents + 1),
            //    self.frame().locals
            //);
            // }
            self.frame_mut().pc += 1;
        }

        // if name != "<clinit>" && name != "<init>" {
        println!(
            "\n{}Exiting method: {}{} in class: {}",
            "\t".repeat(indents),
            self.frame().method.name,
            self.frame().method.descriptor,
            self.frame().class.get_name()
        );
        // }
        self.frames.pop();
        Ok(None)
    }

    /// Executes `Instruction::AaLoad` instruction.
    fn aaload(&mut self) -> crate::result::Result<()> {
        let frame = self.frame_mut();
        let Value::Int(index) = frame.pop() else {
            vm_error!("Expected int for index in aaload.");
        };

        let array_ref = frame.pop();
        if let Value::Null = array_ref {
            todo!("Throw NullPointerException")
        };

        let Value::Ref(RefType::Array(array_ref)) = array_ref else {
            vm_error!("Expected an array for array_ref in aaload.");
        };

        let value = array_ref.get(index as usize);
        Ok(frame.push(value))
    }

    fn aastore(&mut self) -> Result<()> {
        fn handle_obj(vm: &mut VM, obj: ObjRef, mut array: ArrayRef, index: usize) -> Result<()> {
            let class_id = obj.get_class_id();
            let obj_class = vm.method_area.get_class(class_id)?;

            let ArrayComp::Class(comp_class) = array.array_comp() else {
                todo!("Do something when comp_class is not a class type. IDK.");
            };

            let comp_class = vm.assume_load(comp_class);
            if comp_class.is_interface() && vm.is_interface_of(obj_class, comp_class) {
                todo!("Implement Interface inserting")
            }

            if !vm.is_instance_of(obj_class, comp_class) {
                todo!("Do something here! (Shouldn't happen)");
            }

            Ok(array.insert(index, Value::Ref(RefType::Object(obj))))
        }
        let frame = self.frame_mut();

        let value = frame.pop();
        let Value::Int(index) = frame.pop() else {
            bail!("Expected int for index in aastore.");
        };

        let array_ref = frame.pop();
        if let Value::Null = array_ref {
            todo!("Throw NullPointerException");
        };

        let Value::Ref(RefType::Array(mut array)) = array_ref else {
            bail!("Expected an array for arrayref in aastore.");
        };

        match value {
            Value::Null => return Ok(array.insert(index as usize, Value::Null)),
            Value::Ref(RefType::Object(obj)) => {
                let class_id = obj.get_class_id();
                let obj_class = self.method_area.get_class(class_id)?;

                let ArrayComp::Class(comp_class) = array.array_comp() else {
                    todo!("Do something when comp_class is not a class type. IDK.");
                };

                let comp_class = self.assume_load(comp_class);
                if comp_class.is_interface() && self.is_interface_of(obj_class, comp_class) {
                    todo!("Implement Interface inserting")
                }

                if !self.is_instance_of(obj_class, comp_class) {
                    todo!("Do something here! (Shouldn't happen)");
                }

                Ok(array.insert(index as usize, Value::Ref(RefType::Object(obj))))
            }
            Value::Ref(RefType::Array(operand_array)) => {
                match array.array_comp() {
                    ArrayComp::Array(inner_array) => {
                        todo!("Implement arrays within arrays.")
                    }
                    _ => todo!("Do something when inner array has component of primitives"),
                }

                //
                // let comp_class = self.assume_load(comp_class);
                //
                // if comp_class.is_interface() {
                //     todo!("Check if the interface is on that is implemented by arrays.")
                // }

                todo!()
            }
            Value::Byte(_)
            | Value::Double(_)
            | Value::Dynamic { .. }
            | Value::Float(_)
            | Value::Int(_)
            | Value::Long(_)
            | Value::MethodHandle { .. }
            | Value::MethodType(_)
            | Value::ReturnAddress(_)
            | Value::StringConst(_)
            | Value::Short(_) => {
                // Should be impossible.
                bail!("Primitive used for insert in aastore.");
            }
        }
    }

    /// Executes the `Instruction::AaNewArray` instruction.
    fn anew_array(&mut self, class_index: usize) -> Result<()> {
        //TODO Fix this function. It should parse the class_name and do something if it
        // is of the form [<class>;.
        let frame = self.frame();
        let Constant::Class(name_index) = frame.cp.get(class_index).unwrap() else {
            bail!("Expected Constant::Class while executing anewarray.");
        };
        let class_name = frame.cp.get_utf8(*name_index)?;

        if class_name.starts_with('[') {
            todo!();
            let bytes = class_name.as_bytes();
            let mut i = 1;
            while bytes[i] == b'[' {
                i += 1;
            }
        }
        let _ = self.load_class(class_name)?;
        self.new_array(ArrayComp::Class(class_name.to_string()))
    }

    /// Executes the `Instruction::ArrayLength` instruction.
    fn array_length(&mut self) -> Result<()> {
        let frame = self.frame_mut();
        match frame.pop() {
            Value::Ref(RefType::Array(array)) => Ok(frame.push(Value::Int(array.len() as i32))),
            Value::Null => todo!("Throw NullPointerException"),
            value => panic!("Expected a Value::Ref(RefType::Array)) but got {value:?}"),
        }
    }

    /// Executes the `Instruction::ALoad` instruction. `local_index`  
    /// is the index of the local variable in the currently
    /// executing frame's local variable array.
    fn a_load(&mut self, local_index: usize) -> Result<()> {
        let frame = self.frame_mut();
        let object = frame.load(local_index)?;
        if !matches!(
            object,
            (Value::Ref(_) | Value::StringConst(_) | Value::Null)
        ) {
            bail!("Expected ref type for a_load instruction. Received: {object:?}");
        }

        Ok(frame.push(object))
    }

    /// Executes the `Instruction::AStore` instruction.`local_index`  
    /// is the index of the local variable in the currently
    /// executing frame's local variable array.
    /// https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-6.html#jvms-6.5.astore_n
    fn a_store_n(&mut self, local_index: usize) -> Result<()> {
        let frame = self.frame_mut();
        let operand = frame.stack.pop().unwrap();
        match operand {
            value @ (Value::ReturnAddress(_)
            | Value::Ref(_)
            | Value::StringConst(_)
            | Value::Null) => {
                *frame.locals.get_mut(dbg!(local_index)).unwrap() = value;
            }
            _ => panic!(
                "Expected a Reference type or Value::ReturnAddress for the operand \
                    of instruction a_store_n.: {operand:?}"
            ),
        };
        Ok(())
    }

    /// Executes the `Instruction::AThrow` instruction.
    fn a_throw(&mut self) -> Result<()> {
        let frame = self.frame_mut();
        println!("CLASS: {}", frame.class.get_name());
        println!("METHOD: {}", frame.method.name);
        let Value::Ref(RefType::Object(obj)) = frame.pop() else {
            bail!("Expected Exception on the stack.");
        };
        let Value::Ref(RefType::Object(jstring)) = obj.get_field("detailMessage").unwrap() else {
            panic!();
        };
        let Value::Ref(RefType::Array(bytes)) = jstring.get_field("value").unwrap() else {
            panic!();
        };
        let bytes = bytes.get_all();
        let msg = bytes
            .iter()
            .map(|byte| {
                let Value::Byte(byte) = byte else {
                    panic!("Maybe it was a Value::Byte. {byte:?}");
                };
                *byte as u8
            })
            .collect::<Vec<u8>>();
        let msg = String::from_utf8(msg).unwrap();
        println!("MSG: {msg}");
        todo!()
    }

    /// Executes the `Instruction::BaStore` instruction.
    fn bastore(&mut self) -> Result<()> {
        let frame = self.frame_mut();
        let value = frame.pop();
        let Value::Int(value) = value else {
            bail!("Expected Value::Int for the value in bastore. {value:?}");
        };
        let Value::Int(index) = frame.pop() else {
            bail!("Expected Value::Int for the index in bastore.");
        };
        let Value::Ref(RefType::Array(mut array_ref)) = frame.pop() else {
            bail!("Expected RefType::Array for the objref in bastore.");
        };
        let value = match array_ref.array_comp() {
            ArrayComp::Boolean => Value::Byte((value & 1) as i8),
            ArrayComp::Byte => Value::Byte(value as i8),
            array_type => bail!("Invalid array type: {array_type:?} in bastore."),
        };
        Ok(array_ref.insert(index as usize, value))
    }

    /// Executes the `Instruction::CaLoad` instruction.
    fn caload(&mut self) -> Result<()> {
        let frame = self.frame_mut();
        let Value::Int(index) = frame.pop() else {
            bail!("Expected Value::Int as index in caload.");
        };
        let Value::Ref(RefType::Array(mut array_ref)) = frame.pop() else {
            bail!("Expected RefType::Array as array_ref in caload.");
        };
        let Value::Byte(char) = array_ref.get(index as usize) else {
            bail!("Expected Value::Int as char in caload.");
        };
        Ok(frame.push(Value::Int(char as i32)))
    }

    /// Executes the `Instruction::CaStore` instruction.
    fn castore(&mut self) -> Result<()> {
        let frame = self.frame_mut();
        let Value::Int(value) = frame.pop() else {
            bail!("Expected Value::Int for the value in castore.");
        };
        let Value::Int(index) = frame.pop() else {
            bail!("Expected Value::Int for the index in castore.");
        };
        let Value::Ref(RefType::Array(mut array_ref)) = frame.pop() else {
            bail!("Expected RefType::Array for the objref in castore.");
        };
        let ArrayComp::Char = array_ref.array_comp() else {
            bail!("Expected char for array type in castore.");
        };
        Ok(array_ref.insert(index as usize, Value::Byte(index as i8)))
    }

    /// Executes the `Instruction::Checkcast` instruction.
    fn check_cast(&mut self, index: usize) -> Result<()> {
        let frame = self.frame_mut();
        let obj = frame.pop();

        if let Value::Null = obj {
            return Ok(frame.push(Value::Null));
        }

        let Constant::Class(name_index) = self.frame_mut().cp.get(index).unwrap() else {
            bail!("Expected Constant::Class for provided index in check_cast.");
        };
        let class_name = self.frame_mut().cp.get_utf8(*name_index)?;
        let test_class = self.load_class(class_name)?.class;

        match obj {
            Value::Ref(RefType::Array(array)) => {
                if class_name == OBJECT {
                    return Ok(self.frame_mut().push(obj));
                }
                if test_class.is_interface() {
                    todo!();
                }
                if !test_class.is_array_class() {
                    todo!("Return ClassCastException");
                }

                let array_comp = array.array_comp();
                if let ArrayComp::Class(array_class) = array_comp {
                    todo!()
                }

                if &class_name.parse::<ArrayComp>()? == array_comp {
                    return Ok(self.frame_mut().push(obj));
                }
                todo!()
            }
            Value::Ref(RefType::Object(obj_ref)) => {
                let class_id = obj_ref.get_class_id();
                let instance_class = self.method_area.get_class(class_id)?;

                if !self.is_instance_of(instance_class, test_class) {
                    todo!("Throw ClassCastException.");
                }
                Ok(self.frame_mut().push(obj))
            }
            ref_type => panic!("Invalid reference type for check_cast: {ref_type:?}"),
        }
    }

    /// Executes the `Instruction::DAdd` instruction.
    fn dadd(&mut self) -> Result<()> {
        let frame = self.frame_mut();
        let Value::Double(value2) = frame.pop() else {
            bail!("Expected double for value2 in dadd.");
        };
        let Value::Double(value1) = frame.pop() else {
            bail!("Expected double for value1 in dadd.");
        };

        let sum = value2 + value1;
        Ok(frame.push(Value::Double(sum)))
    }

    /// Executes the `Instruction::DLoad<local_index>` instruction.
    /// `local_index` is the index of the local variable in the currently
    /// executing frame's local variable array.
    fn dload_n(&mut self, local_index: usize) -> Result<()> {
        let frame = self.frame_mut();
        let Value::Double(value) = frame.load(local_index)? else {
            bail!("Expected double for value in dload_n");
        };

        Ok(frame.push(Value::Double(value)))
    }

    /// Executes the `Instruction::DStore<local_index>` instruction.
    /// `local_index` is the index of the local variable in the currently
    /// executing frame's local variable array.
    fn dstore_n(&mut self, local_index: usize) -> Result<()> {
        let frame = self.frame_mut();
        let Value::Double(value) = frame.load(local_index)? else {
            bail!("Expected double for value in dstore_n");
        };

        frame.insert_local(local_index, Value::Double(value))?;
        frame.insert_local(local_index, Value::Double(value))
    }

    /// Executes the `Instruction::Dup` instruction.
    fn dup(&mut self) -> Result<()> {
        let frame = self.frame_mut();
        let value = frame.clone_top();
        Ok(frame.push(value))
    }

    /// Executes the `Instruction::DupX1` instruction.
    fn dup_x1(&mut self) -> Result<()> {
        let frame = self.frame_mut();
        let value = frame.clone_top();
        if matches!(value, (Value::Double(_) | Value::Long(_))) {
            bail!("value was not a catagory 1 computation type in dup_x1.");
        }

        Ok(frame.insert(1, value))
    }

    /// Executes the `Instruction::DupX2` instruction.
    fn dup_x2(&mut self) -> Result<()> {
        let frame = self.frame_mut();
        let value = frame.clone_top();
        match matches!(value, (Value::Double(_) | Value::Long(_))) {
            true => {
                frame.insert(1, value.clone());
                frame.insert(1, value);
            }
            false => frame.insert(2, value),
        };
        Ok(())
    }

    /// Executes the `Instruction::F2D` instruction
    fn f2d(&mut self) -> Result<()> {
        let frame = self.frame_mut();
        let Value::Float(value) = frame.pop() else {
            bail!("Expected float for value in f2d.");
        };
        frame.push(Value::Double(value as f64));
        Ok(frame.push(Value::Double(value as f64)))
    }

    /// Executes the `Instruction::F2I instruction
    fn f2i(&mut self) -> Result<()> {
        let frame = self.frame_mut();
        let Value::Float(value) = frame.pop() else {
            bail!("Expected float for value in f2i.");
        };
        Ok(frame.push(Value::Int(value as i32)))
    }

    /// Executes the `Instruction::F2L instruction
    fn f2l(&mut self) -> Result<()> {
        let frame = self.frame_mut();
        let Value::Float(value) = frame.pop() else {
            bail!("Expected float for value in f2l.");
        };
        frame.push(Value::Long(value as i64));
        Ok(frame.push(Value::Long(value as i64)))
    }

    /// Executes the `Instruction::FAdd` instruction.
    fn fadd(&mut self) -> Result<()> {
        let frame = self.frame_mut();
        let Value::Float(value2) = frame.pop() else {
            bail!("Expected float for value2 of idiv");
        };
        let Value::Float(value1) = frame.pop() else {
            bail!("Expected float for value1 of idiv");
        };
        Ok(frame.push(Value::Float(value1 + value2)))
    }

    /// Executes the `Instruction::Fcmpl` and `Instruction::Fcmpg` instruction.
    fn fcmp(&mut self, compare: Compare) -> Result<()> {
        let frame = self.frame_mut();
        let Value::Float(value2) = frame.pop() else {
            bail!("Expected float for value2 in fcmp.");
        };

        let Value::Float(value1) = frame.pop() else {
            bail!("Expected float for value1 in fcmp.");
        };

        if value2.is_nan() || value1.is_nan() {
            return match compare {
                Compare::LessThan => Ok(frame.push(Value::Int(1))),
                Compare::GreaterThan => Ok(frame.push(Value::Int(-1))),
                _ => panic!("fmcp not used properly. Only Compare::LessThan or Compare::GreaterThan are possible"),
            };
        }

        return if value1 > value2 {
            Ok(frame.push(Value::Int(1)))
        } else if value2 == value1 {
            Ok(frame.push(Value::Int(0)))
        } else {
            Ok(frame.push(Value::Int(-1)))
        };
    }

    /// Executes the `Instruction::Fload` and `Instruction::Fload<n>`
    /// instructions
    fn fload_n(&mut self, index: usize) -> Result<()> {
        let frame = self.frame_mut();
        let Value::Float(float) = frame.load(index)? else {
            bail!("Expected float in fload_n");
        };
        Ok(frame.push(Value::Float(float)))
    }

    /// Executes the `Instruction::FMul` instruction.
    fn fmul(&mut self) -> Result<()> {
        let frame = self.frame_mut();
        let Value::Float(value2) = frame.pop() else {
            bail!("Expected float for value2 in fmul.");
        };
        let Value::Float(value1) = frame.pop() else {
            bail!("Expected float for value1 in fmul.");
        };

        let result = value2 * value1;
        if !result.is_nan() && result < f32::EPSILON {
            Ok(frame.push(Value::Float(0_f32)))
        } else {
            Ok(frame.push(Value::Float(result)))
        }
    }

    /// Executes the `Instruction::FStore<n>` instruction.
    fn fstore_n(&mut self, index: usize) -> Result<()> {
        let Value::Float(value) = self.frame_mut().pop() else {
            bail!("Expected float for value in fstore_n.");
        };
        self.frame_mut().insert_local(index, Value::Float(value))
    }

    /// Executes the `Instruction::GetField` instruction.
    /// `field_index` is the index of the `Constant::FieldRef` in the
    /// runtime constant pool.
    fn get_field(&mut self, field_index: usize) -> Result<()> {
        let Constant::FieldRef {
            class_index,
            name_and_type_index,
        } = self.frame().cp.get(field_index).unwrap()
        else {
            bail!("Expected symbolic reference to a field at index: {field_index} for get_field.");
        };

        let value = self.frame_mut().pop();
        if let Value::Null = value {
            todo!("Throw NullPointerException")
        }

        let Value::Ref(RefType::Object(obj)) = value else {
            bail!("Expected Value::Ref(Object) in get_field. Got {value:?}");
        };

        let (name_index, _, _) = self.unpack(*class_index, *name_and_type_index)?;
        let field_val = obj.get_field(self.frame().cp.get_utf8(name_index)?)?;
        if let Value::Long(_) | Value::Double(_) = field_val {
            self.frame_mut().stack.push(field_val.clone());
        }
        self.frame_mut().stack.push(field_val.clone());
        Ok(())
    }

    /// Executes the `Instruction::GetStatic` instruction.
    /// `index` is the index of the `Constant::FieldRef` in the
    /// runtime constant pool.
    fn get_static(&mut self, index: usize) -> Result<()> {
        let Constant::FieldRef {
            class_index,
            name_and_type_index,
        } = self.frame().cp.get(index).unwrap()
        else {
            bail!("Expected symbolic reference to a field at index: {index}");
        };

        let (name_index, _, alloc) = self.unpack(*class_index, *name_and_type_index)?;
        let field_val = alloc.get_field(self.frame().cp.get_utf8(name_index)?)?;
        if matches!(field_val, (Value::Double(_) | Value::Long(_))) {
            self.frame_mut().stack.push(field_val.clone());
        }
        self.frame_mut().stack.push(field_val.clone());
        Ok(())
    }

    /// Executes the `Instruction::I2B` instruction.
    fn i2b(&mut self) -> Result<()> {
        let Value::Int(int) = self.frame_mut().pop() else {
            bail!("Expected int in i2b.");
        };

        Ok(self.frame_mut().push(Value::Int((int as i8) as i32)))
    }

    /// Executes the `Instruction::I2C` instruction.
    fn i2c(&mut self) -> Result<()> {
        let Value::Int(int) = self.frame_mut().pop() else {
            bail!("Expected int in i2c.");
        };

        Ok(self.frame_mut().push(Value::Int((int as u16) as i32)))
    }

    /// Executes the `Instruction::I2D` instruction.
    fn i2d(&mut self) -> Result<()> {
        let Value::Int(int) = self.frame_mut().pop() else {
            bail!("Expected int in i2d.");
        };
        Ok(self.frame_mut().push(Value::Double(int as f64)))
    }

    /// Executes the `Instruction::I2F` instruction.
    fn i2f(&mut self) -> Result<()> {
        let Value::Int(int) = self.frame_mut().pop() else {
            bail!("Expected int in i2f.");
        };

        Ok(self.frame_mut().push(Value::Float(int as f32)))
    }

    /// Executes the `Instruction::I2L` instruction.
    fn i2l(&mut self) -> Result<()> {
        let Value::Int(int) = self.frame_mut().pop() else {
            bail!("Expected int in i2l.");
        };
        Ok(self.frame_mut().push(Value::Long(int as i64)))
    }

    /// Executes the `Instruction::I2S` instruction.
    fn i2s(&mut self) -> Result<()> {
        let Value::Int(int) = self.frame_mut().pop() else {
            bail!("Expected int in i2s.");
        };

        Ok(self.frame_mut().push(Value::Short(int as i16)))
    }

    /// Executes the `Instruction::IAdd` instruction
    fn iadd(&mut self) -> Result<()> {
        let frame = self.frame_mut();

        let Value::Int(value2) = frame.pop() else {
            bail!("Expected int for value2 of iadd");
        };
        let Value::Int(value1) = frame.pop() else {
            bail!("Expected int for value1 of iadd");
        };

        let result = value1.wrapping_add(value2);
        Ok(frame.push(Value::Int(result)))
    }

    /// Executes the `Instruction::IAnd` instruction.
    fn iand(&mut self) -> Result<()> {
        let frame = self.frame_mut();
        let Value::Int(value2) = frame.pop() else {
            bail!("Expected int for value2 in iand.");
        };
        let Value::Int(value1) = frame.pop() else {
            bail!("Expected int for value1 in iand.");
        };
        Ok(frame.push(Value::Int(value2 & value1)))
    }

    /// Executes the `Instruction::IaStore` instruction.
    fn iastore(&mut self) -> Result<()> {
        let frame = self.frame_mut();
        let value = frame.pop();
        let Value::Int(index) = frame.pop() else {
            bail!("Expected Value::Int for index on operand stack while executing iastore.");
        };
        match frame.pop() {
            Value::Ref(RefType::Array(mut array)) => {
                if *array.array_comp() != ArrayComp::Int {
                    bail!("Expected ArrayType::Int for iastore instruction.");
                }
                Ok(array.insert(index as usize, value))
            }
            Value::Null => todo!("Throw NullPointerException."),
            value => bail!("Expected a Value::Ref(RefType::Array)) for iastore, got: {value:?}."),
        }
    }

    /// Executes the `Instruction::IConst` instruction. `int` is the integer
    /// to be pushed on the operand stack.
    fn iconst_n(&mut self, int: i32) { self.frame_mut().push(Value::Int(int)); }

    /// Executes the `Instruction::IDiv` instruction
    fn idiv(&mut self) -> Result<()> {
        let frame = self.frame_mut();
        let Value::Int(value2) = frame.pop() else {
            bail!("Expected int for value2 of idiv");
        };
        let Value::Int(value1) = frame.pop() else {
            bail!("Expected int for value1 of idiv");
        };
        //TODO clean this up to the wrapping_add method.
        let result = Wrapping::<i32>(value1) / Wrapping::<i32>(value2);
        Ok(frame.push(Value::Int(result.0)))
    }

    /// Exectutes the `Instruction::Iinc` instruction. `index` is the index of
    /// the local variable to incremented by `inc`.
    fn iinc(&mut self, index: usize, inc: i32) {
        let frame = self.frame_mut();
        if let Value::Int(num) = frame.get_local(index) {
            *num += inc;
        } else {
            panic!("Int was expected for instruction iinc.");
        }
    }

    /// Executes one of the many compare JVM instructions.
    /// The type of comparison is specified by the `compare::Compare` argument.
    /// Returns a `bool` representing the result of the comparison.
    /// Program counter is already updated if necessary.
    fn ifcmp(&mut self, offset: usize, cmp: Compare) -> bool {
        let frame = self.frame_mut();
        let value2 = frame.stack.pop().unwrap();
        let value1 = frame.stack.pop().unwrap();
        let jmp = Self::if_in(value1, value2, cmp);
        if jmp {
            self.frame_mut().pc = offset;
        }
        jmp
    }

    /// Helper function for `VM::ifcmp`.
    fn if_in(value1: Value, value2: Value, cmp: Compare) -> bool {
        match cmp {
            Compare::Equal => value1 == value2,
            Compare::NotEqual => value1 != value2,
            Compare::LessThan => value1 < value2,
            Compare::GreaterOrEqual => value1 >= value2,
            Compare::GreaterThan => value1 > value2,
            Compare::LessOrEqual => value1 <= value2,
        }
    }

    /// Similar to `VM::ifcmp` but only operates on Java ints and compares to 0.
    fn if_cond(&mut self, offset: usize, cmp: Compare) -> bool {
        let frame = self.frame_mut();
        let int = frame.pop();
        //TODO clean this up to use a let else clause.
        if !matches!(int, Value::Int(_)) {
            panic!("Expected int for if_cond instruction.");
        }
        let jmp = Self::if_in(int, Value::Int(0), cmp);
        if jmp {
            frame.pc = offset;
        }
        jmp
    }

    /// Executes the `Instruction::IfNull` instruction. Returns a `bool`
    /// indicating if a branch is required. Program counter is already updated.
    fn if_null(&mut self, index: usize) -> bool {
        let frame = self.frame_mut();
        if let Value::Null = frame.pop() {
            frame.pc = index;
            return true;
        }
        false
    }

    /// Executes the `Instruction::IfNonNull` instruction. Returns a `bool`
    /// indicating if a branch is required. Program counter is already updated.
    fn if_nonnull(&mut self, index: usize) -> bool {
        let frame = self.frame_mut();
        if let Value::Null = frame.pop() {
            return false;
        }
        frame.pc = index;
        true
    }

    /// Execute the `Instruction::InstanceOf` instruction.
    fn instance_of(&mut self, index: usize) -> Result<()> {
        let frame = self.frame_mut();
        let obj = frame.pop();

        match obj {
            Value::Null => Ok(frame.push(Value::Int(0))),
            Value::Ref(RefType::Array(_)) => todo!(),
            Value::Ref(RefType::Object(obj_ref)) => {
                let class_id = obj_ref.get_class_id();
                let instance_class = self.method_area.get_class(class_id)?;

                let Constant::Class(name_index) = self.frame_mut().cp.get(index).unwrap() else {
                    bail!("Expected Constant::Class for provided index in check_cast.");
                };
                let class_name = self.frame_mut().cp.get_utf8(*name_index)?;
                let test_class = self.load_class(class_name)?.class;

                match self.is_instance_of(instance_class, test_class) {
                    true => Ok(self.frame_mut().push(Value::Int(1))),
                    false => Ok(self.frame_mut().push(Value::Int(0))),
                }
            }
            ref_type => panic!("Invalid reference type {ref_type:?}."),
        }
    }

    /// Executed the `Instruction::InvokeSpecial` instruction. `method_index` is
    /// the index to the `Constant::MethodRef` or `Constant::InterfaceMethodRef`
    /// in the runtime constant pool.
    fn invoke_special(&mut self, method_index: usize) -> Result<Option<Value>> {
        // This method is very involved and convoluted. The algorithm
        // for determining the target method of an `Instruction::InvokeSpecial` is
        // defined here: https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-6.html#jvms-6.5.invokespecial
        match self.frame().cp.get(method_index).unwrap() {
            Constant::MethodRef {
                class_index,
                name_and_type_index,
            } => {
                let is_super = self.superclass_method(*class_index, *name_and_type_index)?;
                let mut class = if is_super {
                    // if is_super use check for method in the superclass
                    let super_class_index = self.frame().class.super_class;
                    let Constant::Class(class_name_index) =
                        self.frame().cp.get(super_class_index).unwrap()
                    else {
                        bail!("Expected class while executing invokespecial.");
                    };

                    let class_name = self.frame().cp.get_utf8(*class_name_index)?;
                    self.load_class(class_name)?.class
                } else {
                    // if not is_super use class named in the symbolic reference
                    let Constant::Class(class_name_index) =
                        self.frame().cp.get(*class_index).unwrap()
                    else {
                        bail!("Expected class while executing invokespecial.")
                    };
                    let class_name = self.frame().cp.get_utf8(*class_name_index)?;
                    self.load_class(class_name)?.class
                };

                let Constant::NameAndType {
                    name_index,
                    descriptor_index,
                } = self.frame().cp.get(*name_and_type_index).unwrap()
                else {
                    bail!("Expected NameAndType while executing invokespecial.");
                };
                let method_name = self.construct_m_name(*name_index, *descriptor_index)?;
                loop {
                    match class.methods.get(&method_name) {
                        None => {
                            let super_index = class.super_class;
                            if super_index == 0 {
                                bail!("Index of superclass should not be 0 in invoke_special");
                            }
                            let Constant::Class(super_class) = class.cp.get(super_index).unwrap()
                            else {
                                bail!("Expected class while executing invokespecial.");
                            };
                            let super_name = class.cp.get_utf8(*super_class).unwrap();
                            class = self.load_class(super_name).unwrap().class;
                        }
                        Some(method) => {
                            if method.is_static() {
                                todo!("check for method in superclass of this class")
                            }
                            //TODO spec requires unfound native methods to be ignored.
                            // Additionally, the program should fail if the object
                            // that is being used to call this function is null. handle_invoke()
                            // does not handle this case.
                            return self.handle_invoke(class, method);
                        }
                    }
                }
            }
            Constant::InterfaceMethodRef {
                class_index,
                name_and_type_index,
            } => {
                todo!()
            }
            _ => bail!(
                "Expected symbolic reference to a method or interface method in invoke_special"
            ),
        }
    }

    /// Executed the `Instruction::InvokeStatic` instruction. `method_index` is
    /// the index to the `Constant::MethodRef` in the runtime constant pool.
    fn invoke_static(&mut self, method_index: usize) -> Result<Option<Value>> {
        let frame = self.frame();
        let Constant::MethodRef {
            class_index,
            name_and_type_index,
        } = frame.cp.get(method_index).unwrap()
        else {
            bail!("Expected Constant::MethodRef in invoke_static.");
        };

        let (name_index, desc_index, alloc) = self.unpack(*class_index, *name_and_type_index)?;
        let (class, method) = self.to_method_class(name_index, desc_index, &alloc)?;

        debug_assert!(method.is_static());
        self.handle_invoke(class, method)
    }

    /// Executed the `Instruction::InvokeVirtual` instruction. `method_index` is
    /// the index to the `Constant::MethodRef` in the runtime constant pool.
    fn invoke_virtual(&mut self, method_index: usize) -> Result<Option<Value>> {
        let frame = self.frame();
        let Constant::MethodRef {
            class_index,
            name_and_type_index,
        } = frame.cp.get(method_index).unwrap()
        else {
            bail!("Expected Constant::MethodRef in invoke_virtual.");
        };

        let (name_index, desc_index, alloc) = self.unpack(*class_index, *name_and_type_index)?;
        let (class, method) = self.to_method_class(name_index, desc_index, &alloc)?;

        debug_assert!(!method.is_static());
        // Assert the object ref is nonnull
        let num_params = method.parsed_descriptor.num_params();
        let stack_len = self.frame().stack.len();
        let obj_ref = self.frame().stack.get(stack_len - 1 - num_params).unwrap();
        if let Value::Null = obj_ref {
            todo!("Throw NullPointerException!")
        }
        self.handle_invoke(class, method)
    }

    /// Executes the `Instruction::ILoad` instruction.
    /// `local_index` is the index to the local variable in the
    /// locals array.
    fn iload_n(&mut self, local_index: usize) -> Result<()> {
        let frame = self.frame_mut();

        let int = frame.load(local_index)?;
        Ok(frame.push(int))
    }

    /// Executes `Instruction::IStore` instruction. `local_index`
    /// is the index of the local variable in the locals array.
    fn istore_n(&mut self, local_index: usize) -> Result<()> {
        let frame = self.frame_mut();
        //TODO clean this up to use a let else clause.
        let int = frame.pop();
        if !matches!(int, Value::Int(_)) {
            bail!("Expected a int for istore instruction.");
        }
        Ok(*frame.locals.get_mut(local_index).unwrap() = int)
    }

    /// Executes the `Instruction::ISub` instruction.
    fn isub(&mut self) -> Result<()> {
        let frame = self.frame_mut();
        let Value::Int(value2) = frame.pop() else {
            bail!("Expected int for value2 in isub.");
        };

        let Value::Int(value1) = frame.pop() else {
            bail!("Expected int for value1 in isub.");
        };

        let result = Wrapping::<i32>(value1) - Wrapping::<i32>(value2);
        Ok(frame.push(Value::Int(result.0)))
    }

    /// Executes the `Instruction::IShl` instruction.
    fn ishl(&mut self) -> Result<()> {
        let frame = self.frame_mut();
        let Value::Int(value2) = frame.pop() else {
            bail!("Expected int for value2 in ishr");
        };
        let Value::Int(value1) = frame.pop() else {
            bail!("Expected int for value1 in ishr");
        };

        Ok(frame.push(Value::Int(value1 << (value2 & 0x1f))))
    }

    /// Executes the `Instruction::IShr` instruction.
    fn ishr(&mut self) -> Result<()> {
        let frame = self.frame_mut();
        let Value::Int(value2) = frame.pop() else {
            bail!("Expected int for value2 in ishr");
        };
        let Value::Int(value1) = frame.pop() else {
            bail!("Expected int for value1 in ishr");
        };

        Ok(frame.push(Value::Int(value1 >> (value2 & 0x1f))))
    }

    /// Executes `Instruction::IRem` instruction and pushes
    /// result to the operand stack.
    fn irem(&mut self) -> Result<()> {
        let frame = self.frame_mut();
        let value2 = frame.pop();
        let value1 = frame.pop();
        //TODO Clean this up to use let else clauses.
        if let Value::Int(int1) = value1 {
            if let Value::Int(int2) = value2 {
                if int2 == 0 {
                    bail!("Cannot use 0 in modular arithmetic.");
                }
                self.frame_mut().push(Value::Int(int1 % int2));
                return Ok(());
            }
        }
        bail!("expected 2 integers for irem instruction.");
    }

    /// Executes the `Instruction::IuShR` instruction.
    fn iushr(&mut self) -> Result<()> {
        let frame = self.frame_mut();
        let Value::Int(value2) = frame.pop() else {
            bail!("Expected int for value2 in ishur.");
        };
        let Value::Int(value1) = frame.pop() else {
            bail!("Expected int for value1 in ishur.");
        };

        let shift = value2 & 0x1f;
        let result = if value1 < 0 {
            // Check: https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-6.html#jvms-6.5.iushr
            (value1 >> shift) + 2i32.rotate_left(!(shift as u32))
        } else {
            value1 >> shift
        };
        Ok(frame.push(Value::Int(result)))
    }

    /// Executes the `Instruction::IxOr` instruction.
    fn ixor(&mut self) -> Result<()> {
        let frame = self.frame_mut();
        let Value::Int(value2) = frame.pop() else {
            bail!("Expected Value::Int for value2 in ixor.");
        };
        let Value::Int(value1) = frame.pop() else {
            bail!("Expected Value::Int for value1 in ixor.");
        };

        Ok(frame.push(Value::Int(value2 ^ value1)))
    }

    /// Executes the `Instruction::L2I` instruction.
    fn l2i(&mut self) -> Result<()> {
        let frame = self.frame_mut();
        let Value::Long(long) = frame.pop() else {
            bail!("Expected Value::Long at top of operand stack for l2i");
        };

        Ok(frame.push(Value::Int(long as i32)))
    }

    fn ladd(&mut self) -> Result<()> {
        let frame = self.frame_mut();
        let Value::Long(value2) = frame.pop() else {
            bail!("Expected Value::Long for value2 in ladd.");
        };

        let Value::Long(value1) = frame.pop() else {
            bail!("Expected Value::Long for value1 in ladd.");
        };

        Ok(frame.push(Value::Long(value2.wrapping_add(value1))))
    }

    /// Executes the `Instruction::Lcmp` instruction.
    fn lcmp(&mut self) -> Result<()> {
        let frame = self.frame_mut();
        let Value::Long(value2) = frame.pop() else {
            bail!("Expected long for value2 in lcmp.");
        };

        let Value::Long(value1) = frame.pop() else {
            bail!("Expected long for value1 in lcmp.");
        };

        return if value1 > value2 {
            Ok(frame.push(Value::Int(1)))
        } else if value2 == value1 {
            Ok(frame.push(Value::Int(0)))
        } else {
            Ok(frame.push(Value::Int(-1)))
        };
    }

    /// Executes the `Instruction::LConst<n>` instruction.
    fn lconst_n(&mut self, long: i64) -> Result<()> { Ok(self.frame_mut().push(Value::Long(long))) }

    /// Executes the `Instruction::LAnd` instruction.
    fn land(&mut self) -> Result<()> {
        let frame = self.frame_mut();
        let Value::Long(value2) = frame.pop() else {
            bail!("Expected Value::Long for value2 in land.");
        };
        let Value::Long(value1) = frame.pop() else {
            bail!("Expected Value::Long for value1 in land.");
        };

        Ok(frame.push(Value::Long(value2 & value1)))
    }

    /// Executes the `Instruction::Ldc`, and `Instruction::LdcW` instructions.
    /// `index` is the index of the constant in the runtime constant pool.
    fn load_const(&mut self, index: usize) -> Result<()> {
        let frame = self.frame_mut();
        let cp = frame.cp;
        let constant = cp.get(index).unwrap();
        let value = match constant {
            // Only these Constants are considered to be loadable:
            //https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.4-310
            // Java longs or doubles are loaded by the lcd2 and lcd2_w instruction.
            // https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-6.html#jvms-6.5.ldc
            Constant::Integer(int) => Value::Int(*int),
            Constant::Float(f) => Value::Float(*f),
            Constant::Class(class_index) => {
                let Constant::UTF8(class_name) = frame.cp.get(*class_index).unwrap() else {
                    todo!("Implement error handling.");
                };
                // load class, just in case it is not loaded
                let class = self.load_class_immut(class_name).unwrap();
                // Use class.get_name() here because in the case of array classes,
                // class_name has a trailing ';' that is parsed out when loading the class by
                // name.
                Value::new_object(self.heap.get_class_obj(&class.get_name()))
            }
            Constant::String(string_index) => {
                let string = cp.get_utf8(*string_index)?.into();
                let string_obj = match self.heap.interned_string(string) {
                    None => self.create_java_string(string, true),
                    Some(string_obj) => string_obj,
                };
                Value::new_object(string_obj)
            }
            Constant::MethodHandle {
                reference_kind,
                reference_index,
            } => Value::MethodHandle {
                reference_kind: *reference_kind,
                reference_index: *reference_index,
            },
            Constant::MethodType(type_index) => Value::MethodType(*type_index),
            Constant::Dynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            } => Value::Dynamic {
                bootstrap_method_attr_index: *bootstrap_method_attr_index,
                name_and_type_index: *name_and_type_index,
            },
            _ => panic!("Non loadable constant pointed to by instruction ldc."),
        };
        self.frame_mut().stack.push(value);
        Ok(())
    }

    /// Executes the `Instruction::Ldc2W` instruction.
    /// `index` is the index of the Java long or Java double
    /// in the runtime constant pool.
    fn load_const2(&mut self, index: usize) -> Result<()> {
        let frame = self.frame_mut();
        let cp = frame.cp;
        let constant = cp.get(index).unwrap();
        let value = match constant {
            Constant::Long(l) => Value::Long(*l),
            Constant::Double(d) => Value::Double(*d),
            _ => panic!("Non long or double constant pointed to by instruction ldc2w."),
        };
        frame.stack.push(value.clone());
        Ok(frame.stack.push(value))
    }

    /// Executes the `Instruction::LLoad<n>` instruction.
    fn lload(&mut self, index: usize) -> Result<()> {
        let frame = self.frame_mut();
        let Value::Long(long) = frame.load(index)? else {
            bail!("Expected Value::Long for value in lload.");
        };

        Ok(frame.push(Value::Long(long)))
    }

    /// Executes the `Instruction::LShr` instruction.
    fn lshl(&mut self) -> Result<()> {
        let frame = self.frame_mut();

        let Value::Int(value2) = frame.pop() else {
            bail!("Expected Value::Int for value2 in lshl.");
        };
        let Value::Long(value1) = frame.pop() else {
            bail!("Expected Value::Long for value1 in lshl.");
        };
        let value2 = value2 as i64;
        Ok(frame.push(Value::Long(value1 << (value2 & 0x3f))))
    }

    /// Executes the `Instruction::LShl` instruction.
    fn lshr(&mut self) -> Result<()> {
        let frame = self.frame_mut();

        let Value::Int(value2) = frame.pop() else {
            bail!("Expected Value::Int for value2 in lshr.");
        };
        let Value::Long(value1) = frame.pop() else {
            bail!("Expected Value::Long for value1 in lshr.");
        };
        let value2 = value2 as i64;
        Ok(frame.push(Value::Long(value1 >> (value2 & 0x3f))))
    }

    /// Executes the `Instruction::LStore<n>` instruction.
    fn lstore(&mut self, index: usize) -> Result<()> {
        let frame = self.frame_mut();
        let Value::Long(long) = frame.pop() else {
            bail!("Expected Value::Long at top of operand stack in lstore.");
        };

        frame.insert_local(index, Value::Long(long))
    }

    /// Executes the `Instruction::LSub` instruction.
    fn lsub(&mut self) -> Result<()> {
        let frame = self.frame_mut();
        let Value::Long(value2) = frame.pop() else {
            bail!("Expected long for value2 in isub.");
        };

        let Value::Long(value1) = frame.pop() else {
            bail!("Expected long for value1 in isub.");
        };

        let result = Wrapping::<i64>(value1) - Wrapping::<i64>(value2);
        Ok(frame.push(Value::Long(result.0)))
    }

    /// Executes the `Instruction::MonitorEnter` instruction.
    fn monitor_enter(&mut self) -> Result<()> {
        let _ = self.frame_mut().pop();
        println!("Ignoring MonitorEnter.");
        Ok(())
    }

    /// Executes the `Instruction::MonitorExit` instruction.
    fn monitor_exit(&mut self) -> Result<()> {
        let _ = self.frame_mut().pop();
        println!("Ignoring MonitorExit.");
        Ok(())
    }

    /// Executes the `Instruction::NewArray` instruction. Length
    /// is assumed to be immediately on the operand stack.
    fn new_array(&mut self, array_comp: ArrayComp) -> Result<()> {
        let frame = self.frame_mut();
        let Value::Int(array_len) = frame.pop() else {
            bail!("Expected Value::Int when accessing array length for newarray instruction.");
        };
        let array_len = array_len as usize;
        let array = self.heap.new_array(array_len, array_comp);
        Ok(self.frame_mut().push(Value::new_array(array)))
    }

    /// Executes the `Instruction::New` instruction. `class_index` is an index
    /// to the runtime constant pool whose entry is a symbolic reference to
    /// a class or interface. New Java object is pushed on to the operand stack.
    fn new_obj(&mut self, class_index: usize) -> Result<()> {
        let frame = self.frame();
        let Constant::Class(name_index) = frame.cp.get(class_index).unwrap() else {
            bail!("Expected a symbolic class reference at index: {class_index}")
        };

        let class_name = frame.cp.get_utf8(*name_index)?;
        let instance_data = self.load_hierarchy(class_name)?;
        let obj = self.heap.new_object(instance_data);
        let value = Value::new_object(obj);
        Ok(self.frame_mut().push(value))
    }

    /// Executes the `Instruction::Pop` instruction. Nothing is returned.
    fn pop(&mut self) -> Result<()> {
        let value = self.frame_mut().pop();
        if matches!(value, (Value::Double(_) | Value::Long(_))) {
            bail!("pop instruction cannot be executed on longs or doubles.");
        };
        Ok(())
    }

    /// Executes the `Instruction::Pop2` instruction. Nothing is returned.
    fn pop2(&mut self) -> Result<()> {
        let frame = self.frame_mut();
        let value = frame.pop();
        if !matches!(value, (Value::Double(_) | Value::Long(_))) {
            // pop2 stipulates that 2 values are popped if value is NOT a category 2
            // computational type.
            let _ = frame.pop();
        }
        Ok(())
    }

    /// Executes the `Instruction::PutField` instruction.
    /// `field_index` is the index of the `Constant::FieldRef` in the
    /// runtime constant pool.
    fn put_field(&mut self, field_index: usize) -> Result<()> {
        let Constant::FieldRef {
            class_index,
            name_and_type_index,
        } = self.frame().cp.get(field_index).unwrap()
        else {
            bail!("Expected Constant::FieldRef for a put_field instruction.");
        };
        let value = self.frame_mut().pop();
        let Value::Ref(RefType::Object(mut obj)) = self.frame_mut().pop() else {
            bail!("Expected a ref of RefType::Object for put_field instruction.")
        };

        let (f_name, _) = self.unpack_f_name(*class_index, *name_and_type_index)?;
        obj.set_field(&f_name, value)?;
        Ok(())
    }

    /// Executes the `Instruction::PutStatic` instruction.
    /// `field_index` is the index of the `Constant::FieldRef` in the
    /// runtime constant pool.
    fn put_static(&mut self, field_index: usize) -> Result<()> {
        let Constant::FieldRef {
            class_index,
            name_and_type_index,
        } = self.frame().cp.get(field_index).unwrap()
        else {
            bail!("Expected Constant::FieldRef for a put_static instruction.");
        };
        let (f_name, mut data) = self.unpack_f_name(*class_index, *name_and_type_index)?;

        data.set_field(&f_name, self.frame_mut().pop())?;
        Ok(())
    }

    fn return_val(&mut self) -> Value {
        let value = self.frame_mut().pop();
        debug_assert!(self.frame().stack.is_empty());
        self.frames.pop();
        value
    }

    /// Executes the `Instruction::SiPush` instruction.
    fn sipush(&mut self, short: i16) -> Result<()> {
        // per the spec, the short is sign-extended to a Java int.
        Ok(self.frame_mut().push(Value::Int(short as i32)))
    }
}
