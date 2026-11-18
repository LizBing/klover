use std::sync::atomic::{
    AtomicI8, AtomicI16, AtomicI32, AtomicI64, AtomicU8, AtomicU16, AtomicU32, AtomicU64, Ordering,
};

use crate::gc_bindings::oop_hierarchy::NObjPtr;

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct JBoolean(pub u8);
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct JByte(pub i8);
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct JShort(pub i16);
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct JChar(pub u16);
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct JInt(pub i32);
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct JLong(pub i64);
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct JFloat(pub f32);
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct JDouble(pub f64);

#[derive(Debug, Clone, Copy)]
pub enum JValue {
    Boolean(JBoolean),
    Byte(JByte),
    Short(JShort),
    Char(JChar),
    Int(JInt),
    Long(JLong),
    Float(JFloat),
    Double(JDouble),
    Ref(NObjPtr),
}

/// Atomic access to a Java primitive in memory. `VOLATILE` selects sequentially
/// consistent ordering. Ordinary field access uses relaxed ordering.
pub trait JPrimitive {
    /// # Safety
    /// `addr` must point to initialized, readable and writable storage aligned
    /// for the corresponding atomic type. Conflicting unsynchronized accesses
    /// must be atomic and cover exactly the same bytes.
    unsafe fn load<const VOLATILE: bool>(addr: *const Self) -> Self;

    /// # Safety
    /// The same storage and concurrency requirements as `load` apply.
    unsafe fn store<const VOLATILE: bool>(addr: *mut Self, value: Self);
}

macro_rules! impl_jprimitive {
    ($java:ident, $atomic:ident) => {
        impl_jprimitive!($java, $atomic, |value| value, |value| value);
    };
    ($java:ident, $atomic:ident, $from_bits:expr, $to_bits:expr) => {
        impl JPrimitive for $java {
            unsafe fn load<const VOLATILE: bool>(addr: *const Self) -> Self {
                let ordering = if VOLATILE {
                    Ordering::SeqCst
                } else {
                    Ordering::Relaxed
                };
                // SAFETY: The wrapper and atomic have the same size and accept
                // the same bit patterns. The caller guarantees valid access.
                let atomic = unsafe { $atomic::from_ptr(addr.cast_mut().cast()) };
                Self(($from_bits)(atomic.load(ordering)))
            }

            unsafe fn store<const VOLATILE: bool>(addr: *mut Self, value: Self) {
                let ordering = if VOLATILE {
                    Ordering::SeqCst
                } else {
                    Ordering::Relaxed
                };
                // SAFETY: As in load, the layout matches and the caller
                // guarantees valid access.
                let atomic = unsafe { $atomic::from_ptr(addr.cast()) };
                atomic.store(($to_bits)(value.0), ordering);
            }
        }
    };
}

impl_jprimitive!(JBoolean, AtomicU8);
impl_jprimitive!(JByte, AtomicI8);
impl_jprimitive!(JShort, AtomicI16);
impl_jprimitive!(JChar, AtomicU16);
impl_jprimitive!(JInt, AtomicI32);
impl_jprimitive!(JLong, AtomicI64);
impl_jprimitive!(JFloat, AtomicU32, f32::from_bits, f32::to_bits);
impl_jprimitive!(JDouble, AtomicU64, f64::from_bits, f64::to_bits);

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use super::*;

    fn assert_access<T: JPrimitive + Copy, R: Debug + PartialEq>(
        initial: T,
        replacement: T,
        repr: impl Fn(T) -> R,
    ) {
        let mut slot = initial;
        let addr = &raw mut slot;
        // SAFETY: These initialized wrapper types have their atomic counterpart's
        // alignment on supported hosts. The slot is accessed by this thread only.
        unsafe {
            assert_eq!(repr(T::load::<false>(addr)), repr(initial));
            assert_eq!(repr(T::load::<true>(addr)), repr(initial));
            T::store::<false>(addr, replacement);
            assert_eq!(repr(addr.read()), repr(replacement));
            T::store::<true>(addr, initial);
            assert_eq!(repr(addr.read()), repr(initial));
        }
    }

    #[test]
    fn primitive_loads_and_stores_preserve_bits() {
        assert_access(JBoolean(0x80), JBoolean(1), |JBoolean(v)| v);
        assert_access(JByte(i8::MIN), JByte(i8::MAX), |JByte(v)| v);
        assert_access(JShort(i16::MIN), JShort(i16::MAX), |JShort(v)| v);
        assert_access(JChar(0xd800), JChar(u16::MAX), |JChar(v)| v);
        assert_access(JInt(i32::MIN), JInt(i32::MAX), |JInt(v)| v);
        assert_access(JLong(i64::MIN), JLong(i64::MAX), |JLong(v)| v);
        assert_access(
            JFloat(f32::from_bits(0x7fc0_1234)),
            JFloat(-0.0),
            |JFloat(v)| v.to_bits(),
        );
        assert_access(
            JDouble(-0.0),
            JDouble(f64::from_bits(0x7ff8_0000_1234_5678)),
            |JDouble(v)| v.to_bits(),
        );
    }
}
