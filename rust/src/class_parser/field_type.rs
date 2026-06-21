use crate::{class_loader::{symbol_handle::SymbolHandle, symbol_table::SymbolTable}, class_parser::parse_error::{ParseError, ParseResult}, oops::oop_handle::NarrowOOP};

#[derive(Debug)]
pub enum NonArrayFieldType {
    Boolean,
    Byte,
    Char,
    Double,
    Float,
    Int,
    Long,
    Short,
    Class { name: SymbolHandle },
}

impl NonArrayFieldType {
    fn resolve(raw: String) -> ParseResult<Self> {
        match raw.as_str() {
            "B" => return Ok(Self::Boolean),
            "C" => return Ok(Self::Char),
            "D" => return Ok(Self::Double),
            "F" => return Ok(Self::Float),
            "I" => return Ok(Self::Int),
            "J" => return Ok(Self::Long),
            "S" => return Ok(Self::Short),
            "Z" => return Ok(Self::Boolean),

            _ => {
                if raw.chars().nth(0).unwrap_or('\0') == 'L' {
                    let name_raw = raw
                        .chars()
                        .filter(|x| *x != 'L' && *x != ';')
                        .collect();

                    return Ok(Self::Class { name: SymbolTable::intern(name_raw) })
                } else {
                    return Err(ParseError::InvalidDescriptor { raw })
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum FieldType {
    NonArray { ft: NonArrayFieldType },
    Array {
        elem: NonArrayFieldType,
        dimemsions: usize
    }
}

impl FieldType {
    pub fn size_of(&self) -> usize {
        match self {
            Self::Array { elem, dimemsions } => size_of::<NarrowOOP>(),
            Self::NonArray { ft } => match ft {
                NonArrayFieldType::Boolean => size_of::<i8>(),
                NonArrayFieldType::Byte => size_of::<i8>(),
                NonArrayFieldType::Char => size_of::<u16>(),
                NonArrayFieldType::Class { name } => size_of::<NarrowOOP>(),
                NonArrayFieldType::Double => size_of::<f64>(),
                NonArrayFieldType::Float => size_of::<f32>(),
                NonArrayFieldType::Int => size_of::<i32>(),
                NonArrayFieldType::Long => size_of::<i64>(),
                NonArrayFieldType::Short => size_of::<i16>()
            }
        }
    }

    pub fn resolve(mut raw: String) -> ParseResult<Self> {
        let mut dimensions = 0;
        for n in raw.chars() {
            if n == '[' { dimensions += 1 }
            else { break }
        }
        let elem_raw = raw.split_off(dimensions);
        let elem = NonArrayFieldType::resolve(elem_raw)?;

        if dimensions == 0 {
            Ok(Self::NonArray { ft: elem })
        } else {
            Ok(Self::Array { elem, dimemsions: dimensions })
        }
    }
}
