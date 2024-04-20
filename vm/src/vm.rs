use std::path::PathBuf;

use anyhow::{bail, Result};

use sumatra_parser::{
    constant::Constant, flags::MethodAccessFlags, instruction::Instruction, method::Method,
};

use crate::{
    alloc::static_alloc::StaticAlloc,
    call_frame::CallFrame,
    class::Class,
    lli::{class_manager::ClassManager, response::Response},
    method_area::MethodArea,
    value::Value,
};

const MAIN: &str = "main([Ljava/lang/String;)V";
const CLINIT: &str = "<clinit>()V";
const INIT: &str = "<init>()V";

const DEFAULT_VEC_SIZE: usize = 128;

pub struct VM<'vm> {
    pub(crate) frames: Vec<CallFrame<'vm>>,
    pub(crate) method_area: MethodArea,
    pub(crate) stack: Vec<Value>,
    pub(crate) class_manager: ClassManager,
}

impl<'vm> VM<'vm> {
    pub fn init(c_path: PathBuf) -> Self {
        //TODO find good allocation size for vectors
        let method_area = match MethodArea::new() {
            Ok(method_area) => method_area,
            Err(_) => panic!("Memory Allocation Error while starting Sumatra VM"),
        };

        Self {
            frames: Vec::with_capacity(DEFAULT_VEC_SIZE),
            method_area,
            stack: Vec::with_capacity(DEFAULT_VEC_SIZE),
            class_manager: ClassManager::new(c_path),
        }
    }

    pub fn run(&mut self, c_entry: &str) -> Result<()> {
        let main = if !c_entry.ends_with(".class") {
            self.load_class(&format!("{c_entry}{}", ".class"))?
        } else {
            self.load_class(c_entry)?
        };
        let main_class = main.get_class();
        let m_method = find_main(main_class)?;
        let cp = &main_class.cp;
        let frame = CallFrame::construct_cf(m_method, cp);
        self.frames.push(frame);
        while !self.frames.is_empty() {
            self.execute_frame()?;
        }
        let value = main.get_field("name");
        println!("Printing Main.name field: {value:?}");
        Ok(())
    }

    fn execute_frame(&mut self) -> Result<()> {
        let top = self.frames.len() - 1;
        let code = &self.frames[top].method.code;
        let op_code = &code.op_code;
        while let Some(code) = op_code.get(self.frames[top].pc) {
            println!("Executing code: {code:?}");
            match code {
                Instruction::AaLoad => todo!(),
                Instruction::AaStore => todo!(),
                Instruction::AaConstNull => self.frames[top].stack.push(Value::Null),
                Instruction::ALoad(_) => todo!(),
                Instruction::ALoad0 => todo!(),
                Instruction::ALoad1 => todo!(),
                Instruction::ALoad2 => todo!(),
                Instruction::ALoad3 => todo!(),
                Instruction::ANewArray(_) => todo!(),
                Instruction::AReturn => todo!(),
                Instruction::ArrayLength => todo!(),
                Instruction::AStore(_) => todo!(),
                Instruction::AStore0 => todo!(),
                Instruction::AStore1 => todo!(),
                Instruction::AStore2 => todo!(),
                Instruction::AStore3 => todo!(),
                Instruction::AThrow => todo!(),
                Instruction::BaLoad => todo!(),
                Instruction::BaStore => todo!(),
                Instruction::BiPush(_) => todo!(),
                Instruction::CaLoad => todo!(),
                Instruction::CaStore => todo!(),
                Instruction::Checkcast(_) => todo!(),
                Instruction::D2F => todo!(),
                Instruction::D2I => todo!(),
                Instruction::D2L => todo!(),
                Instruction::DAdd => todo!(),
                Instruction::DaLoad => todo!(),
                Instruction::DaStore => todo!(),
                Instruction::Dcmpg => todo!(),
                Instruction::Dcmpl => todo!(),
                Instruction::DConst0 => todo!(),
                Instruction::DConst1 => todo!(),
                Instruction::DDiv => todo!(),
                Instruction::DLoad(_) => todo!(),
                Instruction::DLoad0 => todo!(),
                Instruction::DLoad1 => todo!(),
                Instruction::DLoad2 => todo!(),
                Instruction::DLoad3 => todo!(),
                Instruction::DMul => todo!(),
                Instruction::DNeg => todo!(),
                Instruction::DRem => todo!(),
                Instruction::DReturn => todo!(),
                Instruction::DStore(_) => todo!(),
                Instruction::DStore0 => todo!(),
                Instruction::DStore1 => todo!(),
                Instruction::DStore2 => todo!(),
                Instruction::DStore3 => todo!(),
                Instruction::DSub => todo!(),
                Instruction::Dup => todo!(),
                Instruction::DupX1 => todo!(),
                Instruction::DupX2 => todo!(),
                Instruction::Dup2 => todo!(),
                Instruction::Dup2X1 => todo!(),
                Instruction::Dup2X2 => todo!(),
                Instruction::F2D => todo!(),
                Instruction::F2I => todo!(),
                Instruction::F2L => todo!(),
                Instruction::FAdd => todo!(),
                Instruction::FaLoad => todo!(),
                Instruction::FaStore => todo!(),
                Instruction::Fcmpg => todo!(),
                Instruction::Fcmpl => todo!(),
                Instruction::FConst0 => todo!(),
                Instruction::FConst1 => todo!(),
                Instruction::FConst2 => todo!(),
                Instruction::FDiv => todo!(),
                Instruction::FLoad(_) => todo!(),
                Instruction::FLoad0 => todo!(),
                Instruction::FLoad1 => todo!(),
                Instruction::FLoad2 => todo!(),
                Instruction::FLoad3 => todo!(),
                Instruction::FMul => todo!(),
                Instruction::FNeg => todo!(),
                Instruction::FRem => todo!(),
                Instruction::FReturn => todo!(),
                Instruction::FStore(_) => todo!(),
                Instruction::FStore0 => todo!(),
                Instruction::FStore1 => todo!(),
                Instruction::FStore2 => todo!(),
                Instruction::FStore3 => todo!(),
                Instruction::FSub => todo!(),
                Instruction::GetField(_) => todo!(),
                Instruction::GetStatic(index) => self.get_static(*index)?,
                Instruction::GoTo(_) => todo!(),
                Instruction::GoToW(_) => todo!(),
                Instruction::I2B => todo!(),
                Instruction::I2C => todo!(),
                Instruction::I2D => todo!(),
                Instruction::I2F => todo!(),
                Instruction::I2L => todo!(),
                Instruction::I2S => todo!(),
                Instruction::IAdd => todo!(),
                Instruction::IaLoad => todo!(),
                Instruction::IAnd => todo!(),
                Instruction::IaStore => todo!(),
                Instruction::IConstM1 => todo!(),
                Instruction::IConst0 => todo!(),
                Instruction::IConst1 => todo!(),
                Instruction::IConst2 => todo!(),
                Instruction::IConst3 => todo!(),
                Instruction::IConst4 => todo!(),
                Instruction::IConst5 => todo!(),
                Instruction::IDiv => todo!(),
                Instruction::IfAcmpeq(_) => todo!(),
                Instruction::IfAcmpne(_) => todo!(),
                Instruction::IfIcmpeq(_) => todo!(),
                Instruction::IfIcmpne(_) => todo!(),
                Instruction::IfIcmplt(_) => todo!(),
                Instruction::IfIcmpge(_) => todo!(),
                Instruction::IfIcmpgt(_) => todo!(),
                Instruction::IfIcmple(_) => todo!(),
                Instruction::Ifeq(_) => todo!(),
                Instruction::Ifne(_) => todo!(),
                Instruction::Iflt(_) => todo!(),
                Instruction::Ifge(_) => todo!(),
                Instruction::Ifgt(_) => todo!(),
                Instruction::Ifle(_) => todo!(),
                Instruction::IfNonNull(_) => todo!(),
                Instruction::IfNull(_) => todo!(),
                Instruction::Iinc(_, _) => todo!(),
                Instruction::ILoad(_) => todo!(),
                Instruction::ILoad0 => todo!(),
                Instruction::ILoad1 => todo!(),
                Instruction::ILoad2 => todo!(),
                Instruction::ILoad3 => todo!(),
                Instruction::IMul => todo!(),
                Instruction::INeg => todo!(),
                Instruction::InstanceOf(_) => todo!(),
                Instruction::InvokeDynamic(_, _, _) => todo!(),
                Instruction::InvokeInterface(_, _, _) => todo!(),
                Instruction::InvokeSpecial(_) => todo!(),
                Instruction::InvokeStatic(method_index) => {
                    self.invoke_static(&(*method_index as usize))?
                }
                Instruction::InvokeVirtual(method_index) => {
                    println!("INVOKE_VIRTUAL!: #{method_index}");
                }
                Instruction::IOr => todo!(),
                Instruction::IRem => todo!(),
                Instruction::IReturn => todo!(),
                Instruction::IShL => todo!(),
                Instruction::IShR => todo!(),
                Instruction::IStore(_) => todo!(),
                Instruction::IStore0 => todo!(),
                Instruction::IStore1 => todo!(),
                Instruction::IStore2 => todo!(),
                Instruction::IStore3 => todo!(),
                Instruction::ISub => todo!(),
                Instruction::IuShR => todo!(),
                Instruction::IxOr => todo!(),
                Instruction::Jsr(_) => todo!(),
                Instruction::JsrW(_) => todo!(),
                Instruction::L2D => todo!(),
                Instruction::L2F => todo!(),
                Instruction::L2I => todo!(),
                Instruction::LAdd => todo!(),
                Instruction::LaLoad => todo!(),
                Instruction::LAnd => todo!(),
                Instruction::LaStore => todo!(),
                Instruction::Lcmp => todo!(),
                Instruction::LConst0 => todo!(),
                Instruction::LConst1 => todo!(),
                Instruction::Ldc(index) => self.load_const(index)?,
                Instruction::LdcW(_) => todo!(),
                Instruction::Ldc2W(_) => todo!(),
                Instruction::LDiv => todo!(),
                Instruction::LLoad(_) => todo!(),
                Instruction::LLoad0 => todo!(),
                Instruction::LLoad1 => todo!(),
                Instruction::LLoad2 => todo!(),
                Instruction::LLoad3 => todo!(),
                Instruction::LMul => todo!(),
                Instruction::LNeg => todo!(),
                Instruction::LookUpSwitch { .. } => todo!(),
                Instruction::LOr => todo!(),
                Instruction::LRem => todo!(),
                Instruction::LReturn => todo!(),
                Instruction::LShL => todo!(),
                Instruction::LShR => todo!(),
                Instruction::LStore(_) => todo!(),
                Instruction::LStore0 => todo!(),
                Instruction::LStore1 => todo!(),
                Instruction::LStore2 => todo!(),
                Instruction::LStore3 => todo!(),
                Instruction::LSub => todo!(),
                Instruction::LuShR => todo!(),
                Instruction::LxOr => todo!(),
                Instruction::MonitorEnter => todo!(),
                Instruction::MonitorExit => todo!(),
                Instruction::MultiaNewArray(_, _) => todo!(),
                Instruction::New(_) => todo!(),
                Instruction::NewArray(_) => todo!(),
                Instruction::Nop => todo!(),
                Instruction::Pop => todo!(),
                Instruction::Pop2 => todo!(),
                Instruction::PutField(_) => todo!(),
                Instruction::PutStatic(field_index) => self.put_static(*field_index as usize)?,
                Instruction::Ret(_) => todo!(),
                Instruction::Return => todo!(),
                Instruction::SaLoad => todo!(),
                Instruction::SaStore => todo!(),
                Instruction::SiPush(_) => todo!(),
                Instruction::Swap => todo!(),
                Instruction::TableSwitch { .. } => todo!(),
                Instruction::Wide(winstr) => todo!(),
            }
            self.frames[top].pc += 1;
        }
        self.frames.pop();
        Ok(())
    }

    fn invoke_static(&mut self, method_index: &usize) -> Result<()> {
        let top = self.frames.len() - 1;
        println!("method_index: {method_index}");
        if let Constant::MethodRef {
            class_index,
            name_and_type_index,
        } = self.frames[top].cp.get(*method_index).unwrap()
        {
            let (name_index, desc_index, alloc) = self.unpack(class_index, name_and_type_index)?;
            let class = alloc.get_class();
            let met_name = self.construct_m_name(name_index, desc_index)?;
            let method = class.methods.get(&met_name).unwrap();
            println!("InvokeStatic Method: {method:?}");
            // TODO implement native method calls.
            if method.access_flags.contains(MethodAccessFlags::NATIVE) {
                println!("Method was a native method. Ignoring.");
            }
        } else {
            bail!("Expected Constant::MethodRef in invoke_static.");
        }
        Ok(())
    }

    fn load_const(&mut self, index: &usize) -> Result<()> {
        let top = self.frames.len() - 1;
        let cp = self.frames[top].cp;
        let constant = cp.get(*index).unwrap();
        let value = match constant {
            // Only these Constants are considered to be loadable:
            //https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.4-310
            Constant::Integer(int) => Value::Int(*int),
            Constant::Float(f) => Value::Float(*f),
            Constant::Long(l) => Value::Long(*l),
            Constant::Double(d) => Value::Double(*d),
            Constant::Class(_) => {
                todo!()
            }
            Constant::String(string_index) => {
                Value::StringConst(cp.get_utf8(*string_index)?.into())
            }
            Constant::MethodHandle { .. } => {
                todo!()
            }
            Constant::MethodType(_) => {
                todo!()
            }
            Constant::Dynamic { .. } => {
                todo!()
            }
            _ => panic!("Non loadable constant pointed to by instruction ldc."),
        };
        self.frames[top].stack.push(value);
        Ok(())
    }

    fn get_static(&mut self, index: usize) -> Result<()> {
        // This method cannot be called if there is not at least 1 stack frame.
        let top = self.frames.len() - 1;
        if let Constant::FieldRef {
            class_index,
            name_and_type_index,
        } = self.frames[top].cp.get(index).unwrap()
        {
            let (name_index, _, alloc) = self.unpack(class_index, name_and_type_index)?;
            let field_val = alloc.get_field(self.frames[top].cp.get_utf8(name_index)?);
            self.frames[top].stack.push(field_val.clone());
        }
        Ok(())
    }

    fn put_static(&mut self, field_index: usize) -> Result<()> {
        let top = self.frames.len() - 1;
        if let Constant::FieldRef {
            class_index,
            name_and_type_index,
        } = self.frames[top].cp.get(field_index).unwrap()
        {
            let (f_name, alloc) = self.unpack_f_name(class_index, name_and_type_index)?;
            let field = alloc.get_field_mut(&f_name);
            *field = self.frames[top].stack.pop().unwrap();
        } else {
            bail!("Expected Constant::FieldRef for a put_static instruction.");
        };
        Ok(())
    }
}

// Utility functions are seperated into a different impl block for ease of
// navigation.
impl<'vm> VM<'vm> {
    fn load_class(&mut self, name: &str) -> Result<&'static mut StaticAlloc> {
        match self.class_manager.request(name, &mut self.method_area) {
            Ok(Response::InitReq(class, alloc_index)) => {
                self.init_class(class)?;
                self.method_area.get_mut(alloc_index)
            }
            Ok(Response::Ready(alloc_index)) => self.method_area.get_mut(alloc_index),
            Err(e) => bail!(e),
            _ => panic!("Manager returned a not found!"),
        }
    }

    /// Take a static reference to a class and push its '<clinit>'
    /// method as a stack frame to `vm.frames`.
    fn init_class(&mut self, class: &'static Class) -> Result<()> {
        println!("Initializing class: {:?}", class.get_name());
        let clinit = class.methods.get(CLINIT).unwrap();
        let frame = CallFrame::construct_cf(clinit, &class.cp);
        self.frames.push(frame);
        self.execute_frame()
    }

    /// Takes in constant pool indices for the `Constant::Class(class_name)` and
    /// the `Constant::NameAndType` and returns the `name_index`,
    /// `descriptor_index`,  and a `&mut StaticAlloc` of the class pointed
    /// to by `class_name`. The returned &mut StaticAlloc will have a fully
    /// initialize Class.
    fn unpack(
        &mut self,
        class_index: &usize,
        name_and_type: &usize,
    ) -> Result<(usize, usize, &'static mut StaticAlloc)> {
        let top = self.frames.len() - 1;
        if let Constant::Class(class_name) = self.frames[top].cp.get(*class_index).unwrap() {
            let name = self.frames[top].cp.get_utf8(*class_name)?;
            let static_alloc = self.load_class(name)?;

            if let Constant::NameAndType {
                name_index,
                descriptor_index,
            } = self.frames[top].cp.get(*name_and_type).unwrap()
            {
                Ok((*name_index, *descriptor_index, static_alloc))
            } else {
                bail!(
                    "Provided name_and_type_index did not point to a
                NameAndType constant."
                );
            }
        } else {
            bail!(
                "Class index while executing `get_static` \
                    didn't point to a Class in the constant pool."
            );
        }
    }

    /// Takes in constant pool indices for the `Constant::Class(class_name)` and
    /// the `Constant::NameAndType` and returns a `String` and a `&'static mut
    /// StaticAlloc`. The `String` represents a field name.
    /// The returned `&'static mut StaticAlloc` is the allocation of the class
    /// pointed to by `class_name`, with the class already fully
    /// initialized.
    fn unpack_f_name(
        &mut self,
        class_index: &usize,
        name_and_type: &usize,
    ) -> Result<(String, &'static mut StaticAlloc)> {
        let (name_index, _, alloc) = self.unpack(class_index, name_and_type)?;
        let top = self.frames.len() - 1;
        let f_name = self.frames[top].cp.get_utf8(name_index)?.into();
        Ok((f_name, alloc))
    }

    /// Takes in constant pool indices for the `Constant::Class(class_name)` and
    /// the `Constant::NameAndType` and returns a `String` and a `&'static mut
    /// StaticAlloc`. The `String` represents a method name.
    /// The returned `&'static mut StaticAlloc` is the allocation of the class
    /// pointed to by `class_name`, with the class already fully
    /// initialized.
    fn unpack_m_name(
        &mut self,
        class_index: &usize,
        name_and_type: &usize,
    ) -> Result<(String, &'static mut StaticAlloc)> {
        let (name_index, descr_index, alloc) = self.unpack(class_index, name_and_type)?;
        let name = self.construct_m_name(name_index, descr_index)?;
        Ok((name, alloc))
    }

    #[inline]
    fn construct_m_name(&self, name_index: usize, descr_index: usize) -> Result<String> {
        let top = self.frames.len() - 1;
        let cp = self.frames[top].cp;
        let name = cp.get_utf8(name_index)?;
        let descr = cp.get_utf8(descr_index)?;
        Ok(format!("{name}{descr}"))
    }
}

fn find_main(class: &Class) -> Result<&Method> {
    match class.methods.get(MAIN) {
        None => {
            bail!("No main method found.");
        }
        Some(method) => Ok(method),
    }
}
