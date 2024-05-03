use std::path::PathBuf;

use anyhow::{bail, Result};

use sumatra_parser::{
    constant::Constant, flags::MethodAccessFlags, instruction::Instruction, method::Method,
};

use crate::{
    call_frame::CallFrame,
    class::Class,
    lli::{class_manager::ClassManager, response::Response},
    method_area::MethodArea,
    static_data::StaticData,
    value::Value,
};

const MAIN: &str = "main([Ljava/lang/String;)V";
const CLINIT: &str = "<clinit>()V";
const INIT: &str = "<init>()V";

const DEFAULT_VEC_SIZE: usize = 128;

pub struct VM {
    pub(crate) frames: Vec<CallFrame>,
    pub(crate) method_area: MethodArea,
    pub(crate) stack: Vec<Value>,
    pub(crate) class_manager: ClassManager,
}

impl VM {
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
        let c_data = if !c_entry.ends_with(".class") {
            self.load_class(&format!("{c_entry}{}", ".class"))?
        } else {
            self.load_class(c_entry)?
        };
        let main = c_data.class;
        let m_method = find_main(main)?;
        let cp = &main.cp;
        let num_locals = m_method.parsed_descriptor.num_params();
        //TODO implement arguments to pass into main function
        let frame = CallFrame::new(m_method, cp, num_locals, vec![Value::default()]);
        self.frames.push(frame);
        while !self.frames.is_empty() {
            self.execute_frame()?;
        }

        let value = c_data.get_field("name")?;
        println!("Printing Main.name : {value:?}");

        let value = c_data.get_field("field")?;
        println!("Printing Simple.field : {value:?}");

        let value = c_data.get_field("pi")?;
        println!("Printing Simple.field : {value:?}");

        Ok(())
    }

    fn execute_frame(&mut self) -> Result<()> {
        let top = self.frames.len() - 1;
        let code = &self.frames[top].method.code;
        let op_code = &code.op_code;
        println!("\nExecuting method: {}", self.frames[top].method.name);
        while let Some(code) = op_code.get(self.frames[top].pc) {
            if self.frames[top].method.name != "<clinit>" {
                println!("\tExecuting code: {code:?}");
            }
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
                Instruction::AStore0 => self.a_store_n(0)?,
                Instruction::AStore1 => self.a_store_n(1)?,
                Instruction::AStore2 => self.a_store_n(2)?,
                Instruction::AStore3 => self.a_store_n(3)?,
                Instruction::AThrow => todo!(),
                Instruction::BaLoad => todo!(),
                Instruction::BaStore => todo!(),
                Instruction::BiPush(byte) => self.frames[top].stack.push(Value::Int(*byte as i32)),
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
                Instruction::LdcW(index) => self.load_const(index)?,
                Instruction::Ldc2W(index) => self.load_const2(index)?,
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
                Instruction::Return => break,
                Instruction::SaLoad => todo!(),
                Instruction::SaStore => todo!(),
                Instruction::SiPush(_) => todo!(),
                Instruction::Swap => todo!(),
                Instruction::TableSwitch { .. } => todo!(),
                Instruction::Wide(winstr) => todo!(),
            }
            self.frames[top].pc += 1;
        }
        println!("Exiting method: {}", self.frames[top].method.name);
        self.frames.pop();
        Ok(())
    }

    fn a_store_n(&mut self, local_index: usize) -> Result<()> {
        let top = self.frames.len() - 1;
        // TODO remove print statements.
        println!(
            "Parsed descriptor: {:?}",
            self.frames[top].method.parsed_descriptor
        );
        println!("Local index: {local_index}\n");
        
        let operand = self.frames[top].stack.pop().unwrap();

        match operand {
/*            Value::Ref(obj) => {
                match self.frames[top].locals.get_mut(local_index).unwrap() {
                    Value::Ref(target) => {
                        println!("Target of a_store_n: {:?}", unsafe { &**target; });
                        println!("Operand of a_store_n: {:?}", unsafe { &*obj; });
                        *target = obj;
                    }                            
                    _ => panic!("Expected a Reference type for the target of instruction a_store_n.")
                    
                }
            }
            string @ Value::StringConst(_) => {
                match self.frames[top].locals.get_mut(local_index).unwrap() {
                    Value::Ref(target) => {
                        println!("Target of a_store_n: {:?}", unsafe { &**target; });
                        println!("Operand of a_store_n: {:?}", unsafe { &*obj; });
                        *target = obj
                    }
                    _ => panic!("Expected a Reference type for the target of instruction a_store_n.")

                }
                
                
                if let Value::Ref(target) = self.frames[top].stack.get_mut(local_index).unwrap() {
                    println!("operand was a Value::StringConst.");
                    println!("Target of a_store_n: {:?}", unsafe { &**target; });
                    // println!("Operand of a_store_n: {:?}", unsafe { &*obj; });
                    // *target = obj;
                } else {
                    panic!("Expected a Reference type for the target of instruction a_store_n.");
                };
            }
            */
            value @ ( Value::ReturnAddress(_)  | Value::Ref(_) | Value::StringConst(_)) => {
                println!("Printing locals: {:?}", self.frames[top].locals);
                *self.frames[top].locals.get_mut(local_index).unwrap() = value;
            }
            _ =>  panic!("Expected a Reference type or Value::ReturnAddress for the operand of instruction a_store_n.: {operand:?}")
        };
        Ok(())
    }

    fn invoke_static(&mut self, method_index: &usize) -> Result<()> {
        let top = self.frames.len() - 1;
        let stack_size = self.stack.len();
        if let Constant::MethodRef {
            class_index,
            name_and_type_index,
        } = self.frames[top].cp.get(*method_index).unwrap()
        {
            let (name_index, desc_index, alloc) = self.unpack(class_index, name_and_type_index)?;
            let class = alloc.class;
            let met_name = self.construct_m_name(name_index, desc_index)?;
            let method = class.methods.get(&met_name).unwrap();
            // TODO implement native method calls.
            if method.is_native() {
                println!("Method was a native method. Ignoring.");
                Ok(())
            } else {
                // TODO implement a proper way to pass in the local variable to the stack frame.
                let num_params = method.parsed_descriptor.num_params();
                println!("stack_size: {stack_size} num_locals: {num_params}");
                let params = if num_params == 0 && stack_size == 0 {
                    vec![Value::Null]
                } else {
                    Vec::from(&self.stack[stack_size - num_params..stack_size])
                };
                
                let frame = CallFrame::new(method, &class.cp, num_params, params);
                self.frames.push(frame);
                self.execute_frame()
            }
        } else {
            bail!("Expected Constant::MethodRef in invoke_static.");
        }
    }

    fn load_const(&mut self, index: &usize) -> Result<()> {
        let top = self.frames.len() - 1;
        let cp = self.frames[top].cp;
        let constant = cp.get(*index).unwrap();
        let value = match constant {
            // Only these Constants are considered to be loadable:
            //https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.4-310
            // Java longs or doubles are loaded by the lcd2 and lcd2_w instruction.
            // https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-6.html#jvms-6.5.ldc
            Constant::Integer(int) => Value::Int(*int),
            Constant::Float(f) => Value::Float(*f),
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

    fn load_const2(&mut self, index: &usize) -> Result<()> {
        let top = self.frames.len() - 1;
        let cp = self.frames[top].cp;
        let constant = cp.get(*index).unwrap();
        let value = match constant {
            Constant::Long(l) => Value::Long(*l),
            Constant::Double(d) => Value::Double(*d),
            _ => panic!("Non long or double constant pointed to by instruction ldc2w."),
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
            let field_val = alloc.get_field(self.frames[top].cp.get_utf8(name_index)?)?;
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
            let (f_name, mut data) = self.unpack_f_name(class_index, name_and_type_index)?;
            data.set_field(&f_name, self.frames[top].stack.pop().unwrap())?;
        } else {
            bail!("Expected Constant::FieldRef for a put_static instruction.");
        };
        Ok(())
    }
}

// Utility functions are seperated into a different impl block for ease of
// navigation.
impl VM {
    #[inline(always)]
    fn frame_mut(&mut self) -> &mut CallFrame { self.frames.last_mut().unwrap() }

    #[inline(always)]
    fn frame(&self) -> &CallFrame { self.frames.last().unwrap() }

    fn load_class(&mut self, name: &str) -> Result<StaticData> {
        match self.class_manager.request(name, &mut self.method_area) {
            Ok(Response::InitReq(class, alloc_index)) => {
                self.init_class(class)?;
                self.method_area.class_data(alloc_index)
            }
            Ok(Response::Ready(alloc_index)) => self.method_area.class_data(alloc_index),
            Err(e) => bail!(e),
            _ => panic!("Manager returned a not found!"),
        }
    }

    /// Take a static reference to a class and push its '<clinit>'
    /// method as a stack frame to `vm.frames`.
    fn init_class(&mut self, class: &'static Class) -> Result<()> {
        println!("Initializing class: {:?}", class.get_name());
        let clinit = class.methods.get(CLINIT).unwrap();
        // clinit always takes 0 arguments
        let frame = CallFrame::new(clinit, &class.cp, 0, vec![Value::default()]);
        self.frames.push(frame);
        self.execute_frame()
    }

    /// Takes in constant pool indices for the `Constant::Class(class_name)` and
    /// the `Constant::NameAndType` and returns the `name_index`,
    /// `descriptor_index`,  and a `&mut StaticAlloc` of the class pointed
    /// to by `class_name`. The returned StaticData will have a fully
    /// initialize Class.
    fn unpack(
        &mut self,
        class_index: &usize,
        name_and_type: &usize,
    ) -> Result<(usize, usize, StaticData)> {
        let top = self.frames.len() - 1;
        if let Constant::Class(class_name) = self.frames[top].cp.get(*class_index).unwrap() {
            let name = self.frames[top].cp.get_utf8(*class_name)?;
            println!("unpack;;Loading class {name}");
            let static_data = self.load_class(name)?;

            if let Constant::NameAndType {
                name_index,
                descriptor_index,
            } = self.frames[top].cp.get(*name_and_type).unwrap()
            {
                Ok((*name_index, *descriptor_index, static_data))
            } else {
                bail!(
                    "Provided name_and_type_index did not point to a \
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
    ) -> Result<(String, StaticData)> {
        let (name_index, _, data) = self.unpack(class_index, name_and_type)?;
        let top = self.frames.len() - 1;
        let f_name = self.frames[top].cp.get_utf8(name_index)?.into();
        Ok((f_name, data))
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
    ) -> Result<(String, StaticData)> {
        let (name_index, descr_index, data) = self.unpack(class_index, name_and_type)?;
        let name = self.construct_m_name(name_index, descr_index)?;
        Ok((name, data))
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
