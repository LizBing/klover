use crate::{class_parser::FieldInfo, runtime::ms_api::MsAllocator};

pub struct Fields {}

impl Fields {
    pub fn build(_infos: &[FieldInfo], _msa: &MsAllocator) -> Self {
        unimplemented!()
    }
}
