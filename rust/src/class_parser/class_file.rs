use crate::class_parser::{acc_flags::AccFlags, attr_info::AttrInfo, class_reader::ClassReader, cp_info::ConstantPoolInfo, field_info::FieldInfo, interface_info::InterfaceInfo, method_info::MethodInfo, parse_error::{ParseError, ParseResult}};

const VALID_MAGIC: u32 = 0xCAFEBABE;

pub struct ClassFile {
    this_class: String,
    super_class: Option<String>,
    acc_flags: AccFlags,
    
    constant_pool: Vec<ConstantPoolInfo>,
    interfaces: Vec<InterfaceInfo>,
    fields: Vec<FieldInfo>,
    methods: Vec<MethodInfo>,
    attributes: Vec<AttrInfo>
}

#[derive(Debug)]
pub struct ClassFileBuilder {
    this_class: u16,
    super_class: u16,
    acc_flags: AccFlags,
    
    constant_pool: Vec<ConstantPoolInfo>,
    interfaces: Vec<u16>,
    fields: Vec<FieldInfo>,
    methods: Vec<MethodInfo>,
    attributes: Vec<AttrInfo>
}

impl ClassFileBuilder {
    pub fn from(stream: &[u8]) -> ParseResult<Self> {
        let mut rd = ClassReader::new(stream);
        
        let magic = rd.read_u32()?;
        if magic != VALID_MAGIC {
            return Err(ParseError::InvalidMagic(magic))
        }
    
        let minor = rd.read_u16()?;
        let major = rd.read_u16()?;
        if (major < 45 || major > 69) || (major >= 56 && (minor != 0 && minor != 65535)) {
            return Err(ParseError::InvalidVersion { minor: minor, major: major })
        }
        
        let cp_count = rd.read_u16()?;
        let mut cp = Vec::new();
        for _ in 0..cp_count-1 {
            cp.push(ConstantPoolInfo::read(&mut rd)?);
        }
    
        let raw_acc_flags = rd.read_u16()?;
        let acc_flags = match AccFlags::from_bits(raw_acc_flags) {
            Some(x) => x,
            None => return Err(ParseError::InvalidAccFlags(raw_acc_flags))
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
}