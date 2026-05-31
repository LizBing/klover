use crate::class_parser::{
    acc_flags::AccFlags,
    attr_info::AttrInfo,
    class_reader::ClassReader,
    cp_info::ConstantPoolInfo,
    field_info::FieldInfo,
    interface_info::InterfaceInfo,
    method_info::MethodInfo,
    parse_error::{ParseError, ParseResult},
};

const VALID_MAGIC: u32 = 0xCAFEBABE;

pub struct ClassFile {
    pub minor_version: u16,
    pub major_version: u16,
    pub this_class: String,
    pub super_class: Option<String>,
    pub acc_flags: AccFlags,

    pub constant_pool: Vec<ConstantPoolInfo>,
    pub interfaces: Vec<InterfaceInfo>,
    pub fields: Vec<FieldInfo>,
    pub methods: Vec<MethodInfo>,
    pub attributes: Vec<AttrInfo>,
}

#[derive(Debug)]
pub struct ClassFileBuilder {
    pub minor_version: u16,
    pub major_version: u16,
    pub this_class: u16,
    pub super_class: u16,
    pub acc_flags: AccFlags,

    pub constant_pool: Vec<ConstantPoolInfo>,
    pub interfaces: Vec<u16>,
    pub fields: Vec<FieldInfo>,
    pub methods: Vec<MethodInfo>,
    pub attributes: Vec<AttrInfo>,
}

impl ClassFileBuilder {
    pub fn from(stream: &[u8]) -> ParseResult<Self> {
        let mut rd = ClassReader::new(stream);

        let magic = rd.read_u32()?;
        if magic != VALID_MAGIC {
            return Err(ParseError::InvalidMagic(magic));
        }

        let minor = rd.read_u16()?;
        let major = rd.read_u16()?;
        if (major < 45 || major > 69) || (major >= 56 && (minor != 0 && minor != 65535)) {
            return Err(ParseError::InvalidVersion {
                minor: minor,
                major: major,
            });
        }

        let cp_count = rd.read_u16()?;
        let mut cp = Vec::new();
        for _ in 0..cp_count - 1 {
            cp.push(ConstantPoolInfo::read(&mut rd)?);
        }

        let raw_acc_flags = rd.read_u16()?;
        let acc_flags = match AccFlags::from_bits(raw_acc_flags) {
            Some(x) => x,
            None => return Err(ParseError::InvalidAccFlags(raw_acc_flags)),
        };

        let this_index = rd.read_u16()?;
        let super_index = rd.read_u16()?;

        let interfaces_count = rd.read_u16()?;
        let mut interfaces = Vec::new();
        for _ in 0..interfaces_count {
            interfaces.push(rd.read_u16()?);
        }

        let fields_count = rd.read_u16()?;
        let mut fields = Vec::new();
        for _ in 0..fields_count {
            fields.push(FieldInfo::read(&mut rd)?);
        }

        let methods_count = rd.read_u16()?;
        let mut methods = Vec::new();
        for _ in 0..methods_count {
            methods.push(MethodInfo::read(&mut rd)?);
        }

        let attrs_count = rd.read_u16()?;
        let mut attrs = Vec::new();
        for _ in 0..attrs_count {
            attrs.push(AttrInfo::read(&mut rd)?);
        }

        let res = Self {
            minor_version: minor,
            major_version: major,
            this_class: this_index,
            super_class: super_index,
            acc_flags: acc_flags,

            constant_pool: cp,
            interfaces: interfaces,
            fields: fields,
            methods: methods,
            attributes: attrs,
        };

        Ok(res)
    }

    pub fn build(self) -> ClassFile {
        let this_class = self
            .resolve_class_name(self.this_class as usize)
            .unwrap_or_else(|| "<unknown>".to_string());

        let super_class = if self.super_class == 0 {
            None
        } else {
            self.resolve_class_name(self.super_class as usize)
        };

        ClassFile {
            minor_version: self.minor_version,
            major_version: self.major_version,
            this_class,
            super_class,
            acc_flags: self.acc_flags,
            constant_pool: self.constant_pool,
            interfaces: self
                .interfaces
                .into_iter()
                .map(|name_idx| InterfaceInfo {
                    name_index: name_idx,
                })
                .collect(),
            fields: self.fields,
            methods: self.methods,
            attributes: self.attributes,
        }
    }

    fn resolve_class_name(&self, idx: usize) -> Option<String> {
        match self.constant_pool.get(idx.wrapping_sub(1)) {
            Some(ConstantPoolInfo::ClassInfo { name_index }) => {
                self.resolve_utf8(*name_index as usize)
            }
            _ => None,
        }
    }

    fn resolve_utf8(&self, idx: usize) -> Option<String> {
        match self.constant_pool.get(idx.wrapping_sub(1)) {
            Some(ConstantPoolInfo::Utf8Info { utf8 }) => Some(utf8.clone()),
            _ => None,
        }
    }
}
