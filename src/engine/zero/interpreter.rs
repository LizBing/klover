/*
 * Copyright 2025 Lei Zaakjyu
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::ptr::{null, null_mut};

use cafebabe::bytecode::{ByteCode, Opcode};
use crate::engine::context::Context;
use crate::oops::obj_desc::ObjDesc;
use crate::oops::oop::ObjPtr;
use crate::runtime::rt_exceptions::RuntimeException;
use crate::utils::global_defs::{addr_cast, address};
// Refs: JVMS 21 Table 2.11.1-B

struct ZeroEngine<'a> {
    _ctx: &'a Context,

}

impl<'a> ZeroEngine<'a> {
    pub fn new<'b: 'a>(ctx: &'b Context) -> Self {
        Self { _ctx: ctx }
    }
}

impl ZeroEngine<'_> {
    fn pop_ref(&self) -> Result<&'_ ObjDesc, RuntimeException> {
        let addr = self._ctx.pop::<1, address>();
        match addr_cast(addr) {
            Some(n) => Ok(n),
            None => Err(RuntimeException::NullPointerException),
        }
    }
    
    fn pop_index(&self) -> i32 {
        self._ctx.pop::<1, _>()
    }
}

impl ZeroEngine<'_> {
    fn array_load<const SLOTS: usize, T: Copy>(&self) -> Result<(), RuntimeException> {
        let index = self.pop_index();
        let arrayref = self.pop_ref()?;

        match arrayref.array_get::<T>(index) {
            Some(value) => self._ctx.push::<SLOTS, _>(value),
            None => return Err(RuntimeException::ArrayIndexOutOfBoundsException)
        }

        Ok(())
    }

    fn array_load_oop(&self) -> Result<(), RuntimeException> {
        self.array_load::<1, ObjPtr>()
    }

    fn array_store<const SLOTS: usize, T: Copy>(&self) -> Result<(), RuntimeException> {
        let value = self._ctx.pop::<SLOTS, _>();
        let index = self.pop_index();
        let arrayref = self.pop_ref()?;

        if !arrayref.array_set::<T>(index, value) {
            return Err(RuntimeException::ArrayIndexOutOfBoundsException);
        }

        Ok(())
    }

    fn array_store_oop(&self) -> Result<(), RuntimeException> {
        // todo: tell if value shares the same type of the element type of arrayref.
        self.array_store::<1, ObjPtr>()
    }
}

impl ZeroEngine<'_> {
    fn process(&self, codes: &ByteCode) -> Result<(), RuntimeException> {
        let mut pc = 0;

        while pc < codes.opcodes.len() {
            match codes.opcodes[pc].clone().1 {
                Opcode::Aaload => {
                    self.array_load_oop()?
                }

                Opcode::Aastore => {
                    self.array_store_oop()?;
                }

                Opcode::AconstNull => {
                    self._ctx.push::<1, ObjPtr>(null_mut());
                }

                Opcode::Aload(index) => {
                    // self.local_load_ptr(*index);
                }
/*
                Opcode::Anewarray(t) => {
                    // ...
                }

                Opcode::Areturn => {
                    if self.return_with_value::<address>(SLOT_PER_PTR) {
                        code = self.code();
                        continue;
                    }
                }

                Opcode::Arraylength => {
                    || -> Option<()> {
                        let arrayref = self.pop_valid_obj::<ArrayObjDesc>()?;
                        let value = arrayref.length();
                        self._stack.push(cal_slots::<jint>(), value);

                        None
                    } ();
                }

                Opcode::Astore(index) => {
                    self.local_store_ptr(*index);
                }

                Opcode::Athrow => {
                    // ...
                }

                Opcode::Baload => {
                    self.array_load::<jbyte>();
                }

                Opcode::Bastore => {
                    self.array_store::<jbyte>();
                }

                Opcode::Bipush(byte) => {
                    self._stack.push(cal_slots::<jint>(), *byte as jint);
                }

                Opcode::Caload => {
                    self.array_load::<jchar>();
                }

                Opcode::Castore => {
                    self.array_store::<jchar>();
                }

                Opcode::Checkcast(t) => {
                    // ...
                }

                Opcode::D2f => {
                    self.primitive_cast::<jdouble, jfloat>();
                }

                Opcode::D2i => {
                    self.primitive_cast::<jdouble, jint>();
                }

                Opcode::D2l => {
                    self.primitive_cast::<jdouble, jlong>();
                }

                Opcode::Dadd => {
                    self.add::<jdouble>();
                }

                Opcode::Daload => {
                    self.array_load::<jdouble>();
                }

                Opcode::Dastore => {
                    self.array_store::<jdouble>();
                }

                Opcode::Dcmpg => {
                    self.cmpg::<jdouble>();
                }

                Opcode::Dcmpl => {
                    self.cmpl::<jdouble>();
                }

                Opcode::Dconst0 => {
                    self._stack.push(cal_slots::<jdouble>(), 0.0f64);
                }

                Opcode::Dconst1 => {
                    self._stack.push(cal_slots::<jdouble>(), 1.0f64);
                }

                Opcode::Ddiv => {
                    self.div::<jdouble>();
                }

                Opcode::Dload(index) => {
                    self.local_load::<jdouble>(*index);
                }

                Opcode::Dmul => {
                    self.mul::<jdouble>();
                }

                Opcode::Dneg => {
                    self.neg::<jdouble>();
                }

                Opcode::Drem => {
                    self.rem::<jdouble>();
                }

                Opcode::Dreturn => {
                    if self.return_with_value::<jdouble>(cal_slots::<jdouble>()) {
                        code = self.code();
                        continue;
                    }
                }

                Opcode::Dstore(index) => {
                    self.local_store::<jdouble>(*index);
                }

                Opcode::Dsub => {
                    self.sub::<jdouble>();
                }

                Opcode::Dup => {
                    self.dup();
                }

                Opcode::DupX1 => {
                    self.dup_x1();
                }

                Opcode::DupX2 => {
                    self.dup_x2();
                }

                Opcode::Dup2 => {
                    self.dup2();
                }

                Opcode::Dup2X1 => {
                    self.dup2_x1();
                }

                Opcode::Dup2X2 => {
                    self.dup2_x2();
                }

                Opcode::F2d => {
                    self.primitive_cast::<jfloat, jdouble>();
                }

                Opcode::F2i => {
                    self.primitive_cast::<jfloat, jint>();
                }

                Opcode::F2l => {
                    self.primitive_cast::<jfloat, jlong>();
                }

                Opcode::Fadd => {
                    self.add::<jfloat>();
                }

                Opcode::Faload => {
                    self.array_load::<jfloat>();
                }

                Opcode::Fastore => {
                    self.array_store::<jfloat>();
                }

                Opcode::Fcmpg => {
                    self.cmpg::<jfloat>();
                }

                Opcode::Fcmpl => {
                    self.cmpl::<jfloat>();
                }

                Opcode::Fconst0 => {
                    self._stack.push(cal_slots::<jfloat>(), 0.0f32);
                }

                Opcode::Fconst1 => {
                    self._stack.push(cal_slots::<jfloat>(), 1.0f32);
                }

                Opcode::Fconst2 => {
                    self._stack.push(cal_slots::<jfloat>(), 2.0f32);
                }

                Opcode::Fdiv => {
                    self.div::<jfloat>();
                }

                Opcode::Fload(index) => {
                    self.local_load::<jfloat>(*index);
                }

                Opcode::Fmul => {
                    self.mul::<jfloat>();
                }

                Opcode::Fneg => {
                    self.neg::<jfloat>();
                }

                Opcode::Frem => {
                    self.rem::<jfloat>();
                }

                Opcode::Freturn => {
                    if self.return_with_value::<jfloat>(cal_slots::<jfloat>()) {
                        code = self.code();
                        continue;
                    }
                }

                Opcode::Fstore(index) => {
                    self.local_store::<jfloat>(*index);
                }

                Opcode::Fsub => {
                    self.sub::<jfloat>();
                }

                Opcode::Getfield(field_ref) => {
                    // ...
                }

                Opcode::Getstatic(field_ref) => {
                    // ...
                }

                Opcode::Goto(offset) => {
                    // ...
                }

                Opcode::I2b => {
                    self.primitive_cast::<jint, jbyte>();
                }

                Opcode::I2c => {
                    self.primitive_cast::<jint, jchar>();
                }

                Opcode::I2d => {
                    self.primitive_cast::<jint, jdouble>();
                }

                Opcode::I2f => {
                    self.primitive_cast::<jint, jfloat>();
                }

                Opcode::I2l => {
                    self.primitive_cast::<jint, jlong>();
                }

                Opcode::I2s => {
                    self.primitive_cast::<jint, i16>();
                }

                Opcode::Iadd => {
                    self.add::<jint>();
                }

                Opcode::Iaload => {
                    self.array_load::<jint>();
                }

                Opcode::Iand => {
                    self.and::<jint>();
                }

                Opcode::Iastore => {
                    self.array_store::<jint>();
                }

                Opcode::Iconst0 => {
                    self._stack.push(cal_slots::<jint>(), 0 as jint);
                }

                Opcode::Iconst1 => {
                    self._stack.push(cal_slots::<jint>(), 1 as jint);
                }

                Opcode::Iconst2 => {
                    self._stack.push(cal_slots::<jint>(), 2 as jint);
                }

                Opcode::Iconst3 => {
                    self._stack.push(cal_slots::<jint>(), 3 as jint);
                }

                Opcode::Iconst4 => {
                    self._stack.push(cal_slots::<jint>(), 4 as jint);
                }

                Opcode::Iconst5 => {
                    self._stack.push(cal_slots::<jint>(), 5 as jint);
                }

                Opcode::IconstM1 => {
                    self._stack.push(cal_slots::<jint>(), -1 as jint);
                }

                Opcode::Idiv => {
                    self.div::<jint>();
                }

                Opcode::IfAcmpeq(offs) => {
                    // ...
                }

                Opcode::IfAcmpne(offs) => {
                    // ...
                }

                Opcode::IfIcmpeq(offs) => {
                    // ...
                }

                Opcode::IfIcmpne(offs) => {
                    // ...
                }

                Opcode::IfIcmplt(offs) => {
                    // ...
                }

                Opcode::IfIcmpge(offs) => {
                    // ...
                }

                Opcode::IfIcmpgt(offs) => {
                    // ...
                }

                Opcode::IfIcmple(offs) => {
                    // ...
                }

                Opcode::Ifeq(offs) => {
                    // ...
                }

                Opcode::Ifne(offs) => {
                    // ...
                }

                Opcode::Iflt(offs) => {
                    // ...
                }

                Opcode::Ifge(offs) => {
                    // ...
                }

                Opcode::Ifgt(offs) => {
                    // ...
                }

                Opcode::Ifle(offs) => {
                    // ...
                }

                Opcode::Ifnonnull(offs) => {
                    // ...
                }

                Opcode::Ifnull(offs) => {
                    // ...
                }

                Opcode::Iinc(index, con) => {
                    // ...
                }

                Opcode::Iload(index) => {
                    self.local_load::<jint>(*index);
                }

                Opcode::Imul => {
                    self.mul::<jint>();
                }

                Opcode::Ineg => {
                    self.neg::<jint>();
                }

                Opcode::Instanceof(t) => {
                    // ...
                }

                Opcode::Invokedynamic(t) => {
                    // ...
                }

                Opcode::Invokeinterface(r, count) => {
                    // ...
                }

                Opcode::Invokespecial(r) => {
                    // ...
                }

                Opcode::Invokestatic(r) => {
                    // ...
                }

                Opcode::Invokevirtual(r) => {
                    // ...
                }

                Opcode::Ior => {
                    self.or::<jint>();
                }

                Opcode::Irem => {
                    self.rem::<jint>();
                }

                Opcode::Ireturn => {
                    if self.return_with_value::<jint>(cal_slots::<jint>()) {
                        code = self.code();
                        continue;
                    }
                }

                Opcode::Ishl => {
                    self.shl::<jint>();
                }

                Opcode::Ishr => {
                    self.shr::<jint>();
                }

                Opcode::Istore(index) => {
                    self.local_store::<jint>(*index);
                }

                Opcode::Isub => {
                    self.sub::<jint>();
                }

                Opcode::Iushr => {
                    self.ushr::<u32>();
                }

                Opcode::Ixor => {
                    self.xor::<jint>();
                }

                Opcode::Jsr(offs) => {
                    // ...
                }

                Opcode::L2d => {
                    self.primitive_cast::<jlong, jdouble>();
                }

                Opcode::L2f => {
                    self.primitive_cast::<jlong, jfloat>();
                }

                Opcode::L2i => {
                    self.primitive_cast::<jlong, jint>();
                }

                Opcode::Ladd => {
                    self.add::<jlong>();
                }

                Opcode::Laload => {
                    self.array_load::<jlong>();
                }

                Opcode::Land => {
                    self.and::<jlong>();
                }

                Opcode::Lastore => {
                    self.array_store::<jlong>();
                }

                Opcode::Lcmp => {

                }

                Opcode::Lconst0 => {
                    self._stack.push(cal_slots::<jlong>(), 0 as jlong);
                }

                Opcode::Lconst1 => {
                    self._stack.push(cal_slots::<jlong>(), 1 as jlong);
                }

                Opcode::Ldc(i) => {
                    // ...
                }

                Opcode::LdcW(i) => {
                    // ...
                }

                Opcode::Ldc2W(i) => {
                    // ...
                }

                Opcode::Ldiv => {
                    self.div::<jlong>();
                }

                Opcode::Lload(index) => {
                    self.local_load::<jlong>(*index);
                }

                Opcode::Lmul => {
                    self.mul::<jlong>();
                }

                Opcode::Lneg => {
                    self.neg::<jlong>();
                }

                Opcode::Lookupswitch(t) => {
                    // ...
                }

                Opcode::Lor => {
                    self.or::<jlong>();
                }

                Opcode::Lrem => {
                    self.rem::<jlong>();
                }

                Opcode::Lreturn => {
                    if self.return_with_value::<jlong>(cal_slots::<jlong>()) {
                        code = self.code();
                        continue;
                    }
                }

                Opcode::Lshl => {
                    self.shl::<jlong>();
                }

                Opcode::Lshr => {
                    self.shr::<jlong>();
                }

                Opcode::Lstore(index) => {
                    self.local_store::<jlong>(*index);
                }

                Opcode::Lsub => {
                    self.sub::<jlong>();
                }

                Opcode::Lushr => {
                    self.ushr::<u64>();
                }

                Opcode::Lxor => {
                    self.xor::<jlong>();
                }

                Opcode::Monitorenter => {
                    // ...
                }

                Opcode::Monitorexit => {
                    // ...
                }

                Opcode::Multianewarray(t, dims) => {
                    // ...
                }

                Opcode::New(t) => {
                    // ...
                }

                Opcode::Newarray(t) => {
                    // ...
                }

                Opcode::Nop => {
                    // Do nothing
                }

                Opcode::Pop => {
                    self._stack.pop::<slot_t>(1);
                }

                Opcode::Pop2 => {
                    self._stack.pop::<slot_t>(2);
                }

                Opcode::Putfield(field_ref) => {
                    // ...
                }

                Opcode::Putstatic(field_ref) => {
                    // ...
                }

                Opcode::Ret(index) => {
                    // ...
                }

                Opcode::Return => {
                    if self.return_void() {
                        code = self.code();
                        continue;
                    }
                }

                Opcode::Saload => {
                    self.array_load::<i16>();
                }

                Opcode::Sastore => {
                    self.array_store::<i16>();
                }

                Opcode::Sipush(short) => {
                    self._stack.push(cal_slots::<jint>(), *short as jint);
                }

                Opcode::Swap => {
                    let value2 = self._stack.pop::<slot_t>(1);
                    let value1 = self._stack.pop::<slot_t>(1);
                    self._stack.push(1, value2);
                    self._stack.push(1, value1);
                }

                Opcode::Tableswitch(rt) => {
                    // ...
                }

                // reserved codes

                Opcode::Breakpoint => {
                    // ...
                }

                Opcode::Impdep1 => {
                    // ...
                }

                Opcode::Impdep2 => {
                    // ...
                }

 */

                _ => {}
            }
        }

        Ok(())
    }
}
