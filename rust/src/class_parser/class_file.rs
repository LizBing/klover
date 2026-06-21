use crate::class_loader::symbol_handle::SymbolHandle;
use crate::class_loader::symbol_table::SymbolTable;
use crate::class_parser::attr_info::AttrInfo;
use crate::class_parser::field_type::{FieldType, NonArrayFieldType};
use crate::class_parser::{
    acc_flags::AccFlags,
    class_reader::ClassReader,
    cp_info::ConstantPoolInfo,
    field_info::FieldInfo,
    method_info::MethodInfo,
    parse_error::{ParseError, ParseResult},
};
use crate::oops::oop_handle::NarrowOOP;

const VALID_MAGIC: u32 = 0xCAFEBABE;

/// A preliminarily parsed class file, ready for Klass construction.
///
/// Constructed directly from raw `.class` bytes via [`ClassFile::from`].
/// All symbolic references (class names, field/method names and descriptors,
/// attribute names) are resolved to [`SymbolHandle`] via the global symbol
/// table during parsing.  The raw constant pool is preserved for runtime use.
pub struct ClassFile {
    pub minor_version: u16,
    pub major_version: u16,
    pub this_class: SymbolHandle,
    pub super_class: Option<SymbolHandle>,
    pub acc_flags: AccFlags,

    /// Raw constant pool, indexed from 1 per JVM spec.
    /// `cp[0]` is always `Unusable` as a placeholder;
    /// `cp[n]` directly corresponds to JVM CP entry `n`.
    pub constant_pool: Vec<ConstantPoolInfo>,

    /// Resolved interface names.
    pub interfaces: Vec<SymbolHandle>,

    /// Parsed field metadata.
    pub instance_size: usize,
    pub ptr_fields_count: i32,
    pub fields: Vec<FieldInfo>,
    
    /// Parsed method metadata (including Code, if present).
    pub methods: Vec<MethodInfo>,

    /// Resolved class-level attributes.
    pub attrs: Vec<AttrInfo>,
}

pub(super) fn resolve_symbol(index: u16, cp: &[ConstantPoolInfo]) -> ParseResult<SymbolHandle> {
    match &cp[index as usize] {
        ConstantPoolInfo::Utf8Info { handle } => Ok(handle.clone()),
        _ => Err(ParseError::InvalidCPType)
    }
}

fn resolve_class_symbol(index: u16, cp: &[ConstantPoolInfo]) -> ParseResult<SymbolHandle> {
    match cp[index as usize] {
        ConstantPoolInfo::ClassInfo { name_index } => resolve_symbol(name_index, cp),
        _ => Err(ParseError::InvalidCPType)
    }
}

fn verify_version(minor: u16, major: u16) -> bool {
    (major < 45 || major > 69) || (major >= 56 && (minor != 0 && minor != 65535))
}

pub(super) fn read_acc_flags(rd: &mut ClassReader) -> ParseResult<AccFlags> {
    let raw = rd.read_u16()?;

    match AccFlags::from_bits(raw) {
        Some(x) => Ok(x),
        None => Err(ParseError::InvalidAccFlags(raw)),
    }
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

fn read_interfaces(rd: &mut ClassReader, cp: &[ConstantPoolInfo]) -> ParseResult<Vec<SymbolHandle>> {
    // -- interfaces (resolve immediately) --
    let iface_count = rd.read_u16()?;
    let mut interfaces = Vec::with_capacity(iface_count as usize);
    for _ in 0..iface_count {
        interfaces.push(resolve_class_symbol(rd.read_u16()?, &cp)?);
    }

    Ok(interfaces)
}

pub(super) fn read_attrs(rd: &mut ClassReader, cp: &[ConstantPoolInfo]) -> ParseResult<Vec<AttrInfo>> {
    let attrs_count = rd.read_u16()?;
    let mut attrs = Vec::with_capacity(attrs_count as _);
    for _ in 0..attrs_count {
        let name = rd.read_u16()?;
        let len = rd.read_u32()? as _;
        let data = rd.read(len)?;
        attrs.push(AttrInfo::read(name, data, cp)?);
    }

    Ok(attrs)
}

fn read_fields(rd: &mut ClassReader, cp: &[ConstantPoolInfo]) -> ParseResult<(usize, i32, Vec<FieldInfo>)> {
    let fields_count = rd.read_u16()?;
    let mut fields = Vec::with_capacity(fields_count as _);

    let mut ptr_fields = Vec::new();
    let mut fields_of_8 = Vec::new();
    let mut fields_of_4 = Vec::new();
    let mut fields_of_2 = Vec::new();
    let mut fields_of_1 = Vec::new();
    
    for _ in 0..fields_count {
        let field = FieldInfo::read(rd, cp)?;
        match &field.desc {
            FieldType::Array { elem, dimemsions } => {
                ptr_fields.push(field);
            }

            FieldType::NonArray { ft } => match ft {
                NonArrayFieldType::Boolean => fields_of_1.push(field),
                NonArrayFieldType::Byte => fields_of_1.push(field),
                NonArrayFieldType::Char => fields_of_2.push(field),
                NonArrayFieldType::Class { name } => ptr_fields.push(field),
                NonArrayFieldType::Double => fields_of_8.push(field),
                NonArrayFieldType::Float => fields_of_4.push(field),
                NonArrayFieldType::Int => fields_of_4.push(field),
                NonArrayFieldType::Long => fields_of_8.push(field),
                NonArrayFieldType::Short => fields_of_2.push(field),
            }
        }
    }

    let ptr_fields_count = ptr_fields.len();
    let mut instance_size = 0;
    
    for n in &ptr_fields {
        n.offs.set(instance_size).unwrap();
        instance_size += size_of::<NarrowOOP>();
    }
    fields.append(&mut ptr_fields);

    if ptr_fields_count % 2 != 0 {
        instance_size += size_of::<NarrowOOP>()     // gap for alignment
    }

    for n in &fields_of_8 {
        n.offs.set(instance_size).unwrap();
        instance_size += 8;
    }
    fields.append(&mut fields_of_8);
    
    for n in &fields_of_4 {
        n.offs.set(instance_size).unwrap();
        instance_size += 4;
    }
    fields.append(&mut fields_of_4);
    
    for n in &fields_of_2 {
        n.offs.set(instance_size).unwrap();
        instance_size += 2;
    }
    fields.append(&mut fields_of_2);
    
    for n in &fields_of_1 {
        n.offs.set(instance_size).unwrap();
        instance_size += 1;
    }
    fields.append(&mut fields_of_1);

    instance_size = (instance_size + size_of::<usize>() - 1) & !(size_of::<usize>() - 1);

    Ok((instance_size, ptr_fields_count as _, fields))
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
    /// Parse a `.class` file from raw bytes.
    ///
    /// Validates magic number, version, and structural integrity,
    /// then resolves all symbolic references against the constant pool
    /// and interns names into the global symbol table.
    pub fn from(stream: &[u8]) -> ParseResult<Self> {
        let mut rd = ClassReader::new(stream);

        // -- header --
        let magic = rd.read_u32()?;
        if magic != VALID_MAGIC {
            return Err(ParseError::InvalidMagic(magic));
        }

        let minor = rd.read_u16()?;
        let major = rd.read_u16()?;
        if verify_version(minor, major) {
            return Err(ParseError::InvalidVersion { minor, major });
        }

        let cp = read_cp(&mut rd)?;

        // -- access flags --
        let acc_flags = read_acc_flags(&mut rd)?;

        // -- this class, super class (resolve immediately) --
        let this_class = resolve_class_symbol(rd.read_u16()?, &cp)?;
        let super_index = rd.read_u16()?;
        let super_class = if super_index == 0 {
            None
        } else {
            Some(resolve_class_symbol(super_index, &cp)?)
        };

        let interfaces = read_interfaces(&mut rd, &cp)?;

        let (instance_size, ptr_fields_count, fields) = read_fields(&mut rd, &cp)?;
        
        let methods = read_methods(&mut rd, &cp)?;
        
        // -- class-level attributes (read & resolve immediately) --
        let attrs = read_attrs(&mut rd, &cp)?;

        Ok(Self {
            minor_version: minor,
            major_version: major,
            this_class,
            super_class,
            acc_flags,
            constant_pool: cp,
            interfaces,
            instance_size,
            ptr_fields_count,
            fields,
            methods,
            attrs,
        })
    }
}
