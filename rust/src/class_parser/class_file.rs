use crate::class_parser::attr_info::{AttrInfo, read_attrs};
use crate::class_parser::field_info::read_fields;
use crate::class_parser::method_info::read_methods;
use crate::class_parser::parse_error::ClassFileErrorKind;
use crate::class_parser::{
    class_reader::ClassReader,
    cp_info::ConstantPoolInfo,
    field_info::FieldInfo,
    method_info::MethodInfo,
    parse_error::{ClassFileError, ClassFileResult},
};

const VALID_MAGIC: u32 = 0xCAFEBABE;

pub struct ClassFile {
    pub(super) this_class: u16,
    pub(super) super_class: u16,
    pub(super) acc_flags: u16,

    /// Raw constant pool, indexed from 1 per JVM spec.
    /// `cp[0]` is always `Unusable` as a placeholder;
    /// `cp[n]` directly corresponds to JVM CP entry `n`.
    pub(super) constant_pool: Vec<ConstantPoolInfo>,

    pub(super) interfaces: Vec<u16>,
    pub(super) fields: Vec<FieldInfo>,
    pub(super) methods: Vec<MethodInfo>,

    pub(super) attrs: Vec<AttrInfo>,
}

fn is_version_valid(minor: u16, major: u16) -> bool {
    major >= 45 && major <= 52 && minor == 0
}

fn read_cp(rd: &mut ClassReader) -> ClassFileResult<Vec<ConstantPoolInfo>> {
    // -- constant pool --
    // JVM 4.4.5: LongInfo / DoubleInfo occupy two slots (n, n+1).
    // Slot n+1 is unusable.  Slot 0 is also unusable (JVM CP is 1-based),
    // so we push a placeholder to keep cp[idx] == CP[idx].
    let cp_count = rd.read_u16()?;
    let mut cp = Vec::with_capacity(cp_count as usize);
    cp.push(ConstantPoolInfo::Unusable); // slot 0 placeholder
    
    let mut slot: u16 = 1;
    while slot < cp_count {
        let entry = ConstantPoolInfo::read(rd, slot)?;
        let wide = matches!(
            entry,
            ConstantPoolInfo::Long(..) | ConstantPoolInfo::Double(..)
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

fn read_interfaces(rd: &mut ClassReader) -> ClassFileResult<Vec<u16>> {
    let iface_count = rd.read_u16()?;
    let mut interfaces = Vec::with_capacity(iface_count as usize);
    for _ in 0..iface_count {
        interfaces.push(rd.read_u16()?);
    }

    Ok(interfaces)
}

impl ClassFile {
    pub fn from(stream: &[u8]) -> ClassFileResult<Self> {
        let mut rd = ClassReader::new(stream);

        // -- header --
        let magic = rd.read_u32()?;
        if magic != VALID_MAGIC {
            return Err(ClassFileError {
                offset: rd.position(),
                kind: ClassFileErrorKind::InvalidMagic(magic),
            });
        }

        let minor = rd.read_u16()?;
        let major = rd.read_u16()?;
        if !is_version_valid(minor, major) {
            return Err(ClassFileError {
                offset: rd.position(),
                kind: ClassFileErrorKind::UnsupportedVersion { major, minor }
            });
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

        if !rd.is_empty() {
            return Err(ClassFileError {
                offset: rd.position(),
                kind: ClassFileErrorKind::TrailingBytes { remaining: rd.remaining() }
            });
        }

        Ok(Self {
            this_class,
            super_class: super_index,
            acc_flags,
            constant_pool: cp,
            interfaces,
            fields,
            methods,
            attrs,
        })
    }
}
