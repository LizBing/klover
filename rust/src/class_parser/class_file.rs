use crate::class_parser::attr_info::AttrInfo;
use crate::class_parser::{
    class_reader::ClassReader,
    cp_info::ConstantPoolInfo,
    field_info::FieldInfo,
    method_info::MethodInfo,
    parse_error::{ParseError, ParseResult},
};

const VALID_MAGIC: u32 = 0xCAFEBABE;

pub struct ClassFile {
    pub minor_version: u16,
    pub major_version: u16,
    pub this_class: u16,
    pub super_index: u16,
    pub acc_flags: u16,

    /// Raw constant pool, indexed from 1 per JVM spec.
    /// `cp[0]` is always `Unusable` as a placeholder;
    /// `cp[n]` directly corresponds to JVM CP entry `n`.
    pub constant_pool: Vec<ConstantPoolInfo>,

    pub interfaces: Vec<u16>,
    pub fields: Vec<FieldInfo>,
    pub methods: Vec<MethodInfo>,

    pub attrs: Vec<AttrInfo>,
}

fn is_version_valid(minor: u16, major: u16) -> bool {
    major >= 45 && major <= 52 && minor == 0
}

fn read_cp(rd: &mut ClassReader) -> ParseResult<Vec<ConstantPoolInfo>> {
    // -- constant pool --
    // JVM 4.4.5: LongInfo / DoubleInfo occupy two slots (n, n+1).
    // Slot n+1 is unusable.  Slot 0 is also unusable (JVM CP is 1-based),
    // so we push a placeholder to keep cp[idx] == CP[idx].
    let cp_count = rd.read_u16()?;
    let mut cp = Vec::with_capacity(cp_count as usize);
    cp.push(ConstantPoolInfo::Unusable); // slot 0 placeholder
    let mut slot: u16 = 1;
    while slot < cp_count {
        let entry = ConstantPoolInfo::read(rd)?;
        let wide = matches!(
            entry,
            ConstantPoolInfo::LongInfo { .. } | ConstantPoolInfo::DoubleInfo { .. }
        );
        cp.push(entry);
        slot += 1;
        if wide && slot < cp_count {
            cp.push(ConstantPoolInfo::Unusable);
            slot += 1;
        }
    }

    Ok(cp)
}

fn read_interfaces(rd: &mut ClassReader) -> ParseResult<Vec<u16>> {
    let iface_count = rd.read_u16()?;
    let mut interfaces = Vec::with_capacity(iface_count as usize);
    for _ in 0..iface_count {
        interfaces.push(rd.read_u16()?);
    }

    Ok(interfaces)
}

pub(super) fn read_attrs(rd: &mut ClassReader, cp: &[ConstantPoolInfo]) -> ParseResult<Vec<AttrInfo>> {
    let attrs_count = rd.read_u16()?;
    let mut attrs = Vec::with_capacity(attrs_count as _);
    for _ in 0..attrs_count {
        if let Some(x) = AttrInfo::read(rd, cp)? {
            attrs.push(x);
        }
    }

    Ok(attrs)
}

fn read_fields(rd: &mut ClassReader, cp: &[ConstantPoolInfo]) -> ParseResult<Vec<FieldInfo>> {
    let fields_count = rd.read_u16()?;
    let mut fields = Vec::with_capacity(fields_count as _);

    for _ in 0..fields_count {
        fields.push(FieldInfo::read(rd, cp)?);
    }

    Ok(fields)
}

fn read_methods(rd: &mut ClassReader, cp: &[ConstantPoolInfo]) -> ParseResult<Vec<MethodInfo>> {
    let methods_count = rd.read_u16()?;
    let mut methods = Vec::with_capacity(methods_count as _);
    for _ in 0..methods_count {
        methods.push(MethodInfo::read(rd, cp)?);
    }

    Ok(methods)
}

impl ClassFile {
    pub fn from(stream: &[u8]) -> ParseResult<Self> {
        let mut rd = ClassReader::new(stream);

        // -- header --
        let magic = rd.read_u32()?;
        if magic != VALID_MAGIC {
            return Err(ParseError::InvalidMagic(magic));
        }

        let minor = rd.read_u16()?;
        let major = rd.read_u16()?;
        if !is_version_valid(minor, major) {
            return Err(ParseError::InvalidVersion { minor, major });
        }

        let cp = read_cp(&mut rd)?;

        // -- access flags --
        let acc_flags = rd.read_u16()?;

        let this_class = rd.read_u16()?;
        let super_index = rd.read_u16()?;

        let interfaces = read_interfaces(&mut rd)?;
        let fields = read_fields(&mut rd, &cp)?;
        let methods = read_methods(&mut rd, &cp)?;
        
        let attrs = read_attrs(&mut rd, &cp)?;

        Ok(Self {
            minor_version: minor,
            major_version: major,
            this_class,
            super_index,
            acc_flags,
            constant_pool: cp,
            interfaces,
            fields,
            methods,
            attrs,
        })
    }
}
