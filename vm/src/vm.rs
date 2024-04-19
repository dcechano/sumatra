use std::path::Path;
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
    pub(crate) stack: Vec<&'vm Value>,
    pub(crate) class_manager: ClassManager,
}

impl<'vm> VM<'vm> {
    pub fn init(c_path: &Path) -> Self {
        //TODO find good allocation size for vectors
        let method_area= match MethodArea::new() {
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

    pub fn run(&mut self, class: &'vm mut Class) -> Result<()> {
        let main = find_main(class)?;
        let cp = &class.cp;
        let frame = CallFrame::construct_cf(main, cp);
        self.frames.push(frame);
        while !self.frames.is_empty() {
            self.execute_frame()?;
        }
        Ok(())
    }

    fn execute_frame(&mut self) -> Result<()> {
        let top = self.frames.len() - 1;
        let code = &self.frames[top].method.code;
        let op_code = &code.op_code;
        while let Some(code) = op_code.get(self.frames[top].pc) {
            println!("Executing code: {code:?}");
            match code {
                Instruction::AaLoad => {}
                Instruction::AaStore => {}
                Instruction::AaConstNull => self.frames[top].stack.push(&Value::Null),
                Instruction::ALoad(_) => {}
                Instruction::ALoad0 => {}
                Instruction::ALoad1 => {}
                Instruction::ALoad2 => {}
                Instruction::ALoad3 => {}
                Instruction::ANewArray(_) => {}
                Instruction::AReturn => {}
                Instruction::ArrayLength => {}
                Instruction::AStore(_) => {}
                Instruction::AStore0 => {}
                Instruction::AStore1 => {}
                Instruction::AStore2 => {}
                Instruction::AStore3 => {}
                Instruction::AThrow => {}
                Instruction::BaLoad => {}
                Instruction::BaStore => {}
                Instruction::BiPush(_) => {}
                Instruction::CaLoad => {}
                Instruction::CaStore => {}
                Instruction::Checkcast(_) => {}
                Instruction::D2F => {}
                Instruction::D2I => {}
                Instruction::D2L => {}
                Instruction::DAdd => {}
                Instruction::DaLoad => {}
                Instruction::DaStore => {}
                Instruction::Dcmpg => {}
                Instruction::Dcmpl => {}
                Instruction::DConst0 => {}
                Instruction::DConst1 => {}
                Instruction::DDiv => {}
                Instruction::DLoad(_) => {}
                Instruction::DLoad0 => {}
                Instruction::DLoad1 => {}
                Instruction::DLoad2 => {}
                Instruction::DLoad3 => {}
                Instruction::DMul => {}
                Instruction::DNeg => {}
                Instruction::DRem => {}
                Instruction::DReturn => {}
                Instruction::DStore(_) => {}
                Instruction::DStore0 => {}
                Instruction::DStore1 => {}
                Instruction::DStore2 => {}
                Instruction::DStore3 => {}
                Instruction::DSub => {}
                Instruction::Dup => {}
                Instruction::DupX1 => {}
                Instruction::DupX2 => {}
                Instruction::Dup2 => {}
                Instruction::Dup2X1 => {}
                Instruction::Dup2X2 => {}
                Instruction::F2D => {}
                Instruction::F2I => {}
                Instruction::F2L => {}
                Instruction::FAdd => {}
                Instruction::FaLoad => {}
                Instruction::FaStore => {}
                Instruction::Fcmpg => {}
                Instruction::Fcmpl => {}
                Instruction::FConst0 => {}
                Instruction::FConst1 => {}
                Instruction::FConst2 => {}
                Instruction::FDiv => {}
                Instruction::FLoad(_) => {}
                Instruction::FLoad0 => {}
                Instruction::FLoad1 => {}
                Instruction::FLoad2 => {}
                Instruction::FLoad3 => {}
                Instruction::FMul => {}
                Instruction::FNeg => {}
                Instruction::FRem => {}
                Instruction::FReturn => {}
                Instruction::FStore(_) => {}
                Instruction::FStore0 => {}
                Instruction::FStore1 => {}
                Instruction::FStore2 => {}
                Instruction::FStore3 => {}
                Instruction::FSub => {}
                Instruction::GetField(_) => {}
                Instruction::GetStatic(index) => {
                    self.get_static(*index)?;
                    println!("Operand stack: {:?}", self.stack);
                }
                Instruction::GoTo(_) => {}
                Instruction::GoToW(_) => {}
                Instruction::I2B => {}
                Instruction::I2C => {}
                Instruction::I2D => {}
                Instruction::I2F => {}
                Instruction::I2L => {}
                Instruction::I2S => {}
                Instruction::IAdd => {}
                Instruction::IaLoad => {}
                Instruction::IAnd => {}
                Instruction::IaStore => {}
                Instruction::IConstM1 => {}
                Instruction::IConst0 => {}
                Instruction::IConst1 => {}
                Instruction::IConst2 => {}
                Instruction::IConst3 => {}
                Instruction::IConst4 => {}
                Instruction::IConst5 => {}
                Instruction::IDiv => {}
                Instruction::IfAcmpeq(_) => {}
                Instruction::IfAcmpne(_) => {}
                Instruction::IfIcmpeq(_) => {}
                Instruction::IfIcmpne(_) => {}
                Instruction::IfIcmplt(_) => {}
                Instruction::IfIcmpge(_) => {}
                Instruction::IfIcmpgt(_) => {}
                Instruction::IfIcmple(_) => {}
                Instruction::Ifeq(_) => {}
                Instruction::Ifne(_) => {}
                Instruction::Iflt(_) => {}
                Instruction::Ifge(_) => {}
                Instruction::Ifgt(_) => {}
                Instruction::Ifle(_) => {}
                Instruction::IfNonNull(_) => {}
                Instruction::IfNull(_) => {}
                Instruction::Iinc(_, _) => {}
                Instruction::ILoad(_) => {}
                Instruction::ILoad0 => {}
                Instruction::ILoad1 => {}
                Instruction::ILoad2 => {}
                Instruction::ILoad3 => {}
                Instruction::IMul => {}
                Instruction::INeg => {}
                Instruction::InstanceOf(_) => {}
                Instruction::InvokeDynamic(_, _, _) => {}
                Instruction::InvokeInterface(_, _, _) => {}
                Instruction::InvokeSpecial(_) => {}
                Instruction::InvokeStatic(method_index) => {
                    self.invoke_static(&(*method_index as usize))?
                }
                Instruction::InvokeVirtual(method_index) => {
                    println!("INVOKE_VIRTUAL!: #{method_index}");
                }
                Instruction::IOr => {}
                Instruction::IRem => {}
                Instruction::IReturn => {}
                Instruction::IShL => {}
                Instruction::IShR => {}
                Instruction::IStore(_) => {}
                Instruction::IStore0 => {}
                Instruction::IStore1 => {}
                Instruction::IStore2 => {}
                Instruction::IStore3 => {}
                Instruction::ISub => {}
                Instruction::IuShR => {}
                Instruction::IxOr => {}
                Instruction::Jsr(_) => {}
                Instruction::JsrW(_) => {}
                Instruction::L2D => {}
                Instruction::L2F => {}
                Instruction::L2I => {}
                Instruction::LAdd => {}
                Instruction::LaLoad => {}
                Instruction::LAnd => {}
                Instruction::LaStore => {}
                Instruction::Lcmp => {}
                Instruction::LConst0 => {}
                Instruction::LConst1 => {}
                Instruction::Ldc(index) => {
                    /*let constant = call_frame.cp.get(*index).unwrap();
                    match constant {
                        Constant::Dummy => {
                            todo!()
                        }
                        Constant::UTF8(_) => {
                            todo!()
                        }
                        Constant::Integer(_) => {
                            todo!()
                        }
                        Constant::Float(_) => {
                            todo!()
                        }
                        Constant::Long(_) => {
                            todo!()
                        }
                        Constant::Double(_) => {
                            todo!()
                        }
                        Constant::Class(_) => {
                            todo!()
                        }
                        Constant::String(string_index) => {
                            todo!()
                            // call_frame.op_stack.push(Value::Object(string.
                            // into()));
                        }
                        Constant::FieldRef { .. } => {
                            todo!()
                        }
                        Constant::MethodRef { .. } => {
                            todo!()
                        }
                        Constant::InterfaceMethodRef { .. } => {
                            todo!()
                        }
                        Constant::NameAndType { .. } => {
                            todo!()
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
                        Constant::InvokeDynamic { .. } => {
                            todo!()
                        }
                        Constant::Module(_) => {
                            todo!()
                        }
                        Constant::Package(_) => {
                            todo!()
                        }
                        Constant::UnRecCharSet(_) => bail!("Unrecognized charset"),
                    }*/
                }
                Instruction::LdcW(_) => {}
                Instruction::Ldc2W(_) => {}
                Instruction::LDiv => {}
                Instruction::LLoad(_) => {}
                Instruction::LLoad0 => {}
                Instruction::LLoad1 => {}
                Instruction::LLoad2 => {}
                Instruction::LLoad3 => {}
                Instruction::LMul => {}
                Instruction::LNeg => {}
                Instruction::LookUpSwitch { .. } => {}
                Instruction::LOr => {}
                Instruction::LRem => {}
                Instruction::LReturn => {}
                Instruction::LShL => {}
                Instruction::LShR => {}
                Instruction::LStore(_) => {}
                Instruction::LStore0 => {}
                Instruction::LStore1 => {}
                Instruction::LStore2 => {}
                Instruction::LStore3 => {}
                Instruction::LSub => {}
                Instruction::LuShR => {}
                Instruction::LxOr => {}
                Instruction::MonitorEnter => {}
                Instruction::MonitorExit => {}
                Instruction::MultiaNewArray(_, _) => {}
                Instruction::New(_) => {}
                Instruction::NewArray(_) => {}
                Instruction::Nop => {}
                Instruction::Pop => {}
                Instruction::Pop2 => {}
                Instruction::PutField(_) => {}
                Instruction::PutStatic(field_index) => self.put_static(*field_index as usize)?,
                Instruction::Ret(_) => {}
                Instruction::Return => {}
                Instruction::SaLoad => {}
                Instruction::SaStore => {}
                Instruction::SiPush(_) => {}
                Instruction::Swap => {}
                Instruction::TableSwitch { .. } => {}
                Instruction::Wide(winstr) => {}
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
        } = self.frames[top].cp.get(*method_index as usize).unwrap()
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
            self.frames[top].stack.push(field_val);
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
            *field = *self.frames[top].stack.pop().unwrap();
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
        let clinit = class.methods.get(CLINIT).unwrap();
        let frame = CallFrame::construct_cf(clinit, &class.cp);
        self.frames.push(frame);
        self.execute_frame()
    }

    /// Takes in constant pool indices for the `Constant::Class(class_name)` and
    /// the `Constant::NameAndType` and returns the `name_index`,
    /// `descpriptor_index`,  and a `&mut StaticAlloc` of the class pointed
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
