#[repr(C)]
pub struct ObjPtrImpl;

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct ObjPtr(*mut ObjPtrImpl);

impl ObjPtr {
    pub fn is_null(self) -> bool {
        let Self(raw) = self;
        raw.is_null()
    }

    pub fn as_usize(self) -> usize {
        let Self(raw) = self;
        raw as _
    }

    pub fn encode(self) -> NObjPtr {
        unimplemented!()
    }
}

#[repr(transparent)]
pub struct NObjPtr(u32);

impl NObjPtr {
    pub fn is_null(self) -> bool {
        self.as_u32() == 0
    }

    pub fn as_u32(self) -> u32 {
        let Self(raw) = self;
        raw
    }

    pub fn decode(self) -> ObjPtr {
        unimplemented!()
    }
}
