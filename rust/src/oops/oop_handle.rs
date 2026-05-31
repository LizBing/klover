// Ordinary Object Pointer

#[repr(C)]
struct ObjDesc {
    markword: u64,
    payload: [u8; 0]
}

type OOP = *mut ObjDesc;
type NarrowOOP = u32;
