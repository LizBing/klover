//! Field and method descriptor parsing (JVMS 4.3).

use std::fmt;

use super::error::{ParseError, ParseErrorKind, ParseResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FieldType {
    Boolean,
    Byte,
    Char,
    Short,
    Int,
    Long,
    Float,
    Double,
    Object(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FieldDescriptor {
    raw: String,
    dimensions: u8,
    element: FieldType,
}

impl FieldDescriptor {
    pub(crate) fn parse(raw: &str) -> ParseResult<Self> {
        let (descriptor, consumed) = parse_field(raw.as_bytes(), 0)?;
        if consumed != raw.len() {
            return Err(invalid("trailing characters in field descriptor"));
        }
        Ok(descriptor)
    }

    pub(crate) fn raw(&self) -> &str {
        &self.raw
    }

    pub(crate) fn dimensions(&self) -> u8 {
        self.dimensions
    }

    pub(crate) fn element(&self) -> &FieldType {
        &self.element
    }

    pub(crate) fn slots(&self) -> u16 {
        if self.dimensions > 0 {
            return 1;
        }
        match self.element {
            FieldType::Long | FieldType::Double => 2,
            _ => 1,
        }
    }
}

impl fmt::Display for FieldDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.raw)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MethodDescriptor {
    raw: String,
    parameters: Vec<FieldDescriptor>,
    return_type: Option<FieldDescriptor>,
}

impl MethodDescriptor {
    pub(crate) fn parse(raw: &str) -> ParseResult<Self> {
        let bytes = raw.as_bytes();
        if bytes.first() != Some(&b'(') {
            return Err(invalid("method descriptor must start with '('"));
        }

        let mut parameters = Vec::new();
        let mut index = 1;
        while bytes.get(index) != Some(&b')') {
            let (parameter, consumed) = parse_field(bytes, index)?;
            parameters.push(parameter);
            index = consumed;
        }
        index += 1; // ')'

        let return_type = match bytes.get(index) {
            Some(b'V') => {
                index += 1;
                None
            }
            Some(_) => {
                let (descriptor, consumed) = parse_field(bytes, index)?;
                index = consumed;
                Some(descriptor)
            }
            None => return Err(invalid("missing method return descriptor")),
        };

        if index != bytes.len() {
            return Err(invalid("trailing characters in method descriptor"));
        }

        Ok(Self {
            raw: raw.to_owned(),
            parameters,
            return_type,
        })
    }

    pub(crate) fn raw(&self) -> &str {
        &self.raw
    }

    pub(crate) fn parameters(&self) -> &[FieldDescriptor] {
        &self.parameters
    }

    pub(crate) fn return_type(&self) -> Option<&FieldDescriptor> {
        self.return_type.as_ref()
    }

    pub(crate) fn parameter_slots(&self) -> u16 {
        self.parameters.iter().map(FieldDescriptor::slots).sum()
    }
}

impl fmt::Display for MethodDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.raw)
    }
}

fn parse_field(bytes: &[u8], start: usize) -> ParseResult<(FieldDescriptor, usize)> {
    let mut index = start;
    let mut dimensions: u8 = 0;
    while bytes.get(index) == Some(&b'[') {
        dimensions = dimensions
            .checked_add(1)
            .ok_or_else(|| invalid("too many array dimensions"))?;
        index += 1;
    }

    let element = match bytes.get(index).copied() {
        Some(b'B') => {
            index += 1;
            FieldType::Byte
        }
        Some(b'C') => {
            index += 1;
            FieldType::Char
        }
        Some(b'D') => {
            index += 1;
            FieldType::Double
        }
        Some(b'F') => {
            index += 1;
            FieldType::Float
        }
        Some(b'I') => {
            index += 1;
            FieldType::Int
        }
        Some(b'J') => {
            index += 1;
            FieldType::Long
        }
        Some(b'S') => {
            index += 1;
            FieldType::Short
        }
        Some(b'Z') => {
            index += 1;
            FieldType::Boolean
        }
        Some(b'L') => {
            let name_start = index + 1;
            let semicolon = bytes[name_start..]
                .iter()
                .position(|&b| b == b';')
                .map(|offset| name_start + offset)
                .ok_or_else(|| invalid("unterminated object descriptor"))?;
            if semicolon == name_start {
                return Err(invalid("empty class name in descriptor"));
            }
            let name = std::str::from_utf8(&bytes[name_start..semicolon])
                .map_err(|_| invalid("class name in descriptor is not UTF-8"))?;
            if name.contains(['.', '[', ';', '(']) {
                return Err(invalid("invalid class name in descriptor"));
            }
            index = semicolon + 1;
            FieldType::Object(name.to_owned())
        }
        Some(_) => return Err(invalid("invalid field descriptor")),
        None => return Err(invalid("truncated field descriptor")),
    };

    let raw = std::str::from_utf8(&bytes[start..index])
        .map_err(|_| invalid("descriptor is not UTF-8"))?
        .to_owned();

    Ok((
        FieldDescriptor {
            raw,
            dimensions,
            element,
        },
        index,
    ))
}

fn invalid(message: &'static str) -> ParseError {
    ParseError::new(0, ParseErrorKind::InvalidDescriptor(message))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_field_descriptors() {
        assert_eq!(
            FieldDescriptor::parse("I").unwrap().element(),
            &FieldType::Int
        );
        let array = FieldDescriptor::parse("[[D").unwrap();
        assert_eq!(array.dimensions(), 2);
        assert_eq!(array.element(), &FieldType::Double);
        let object = FieldDescriptor::parse("Ljava/lang/String;").unwrap();
        assert_eq!(
            object.element(),
            &FieldType::Object("java/lang/String".to_owned())
        );
        assert_eq!(FieldDescriptor::parse("[I").unwrap().slots(), 1);
        assert_eq!(FieldDescriptor::parse("J").unwrap().slots(), 2);
    }

    #[test]
    fn rejects_invalid_field_descriptors() {
        for descriptor in ["", "V", "L;", "Lno/semicolon", "[", "II", "Q", "[V"] {
            assert!(FieldDescriptor::parse(descriptor).is_err(), "{descriptor}");
        }
    }

    #[test]
    fn parses_method_descriptors() {
        let none = MethodDescriptor::parse("()V").unwrap();
        assert!(none.parameters().is_empty());
        assert!(none.return_type().is_none());

        let mixed = MethodDescriptor::parse("(IJLjava/lang/String;)[D").unwrap();
        assert_eq!(mixed.parameters().len(), 3);
        assert_eq!(mixed.parameter_slots(), 4);
        assert_eq!(mixed.return_type().unwrap().dimensions(), 1);
    }

    #[test]
    fn rejects_invalid_method_descriptors() {
        for descriptor in ["", "V", "(V)V", "()", "(I", "()VV", "()Vx"] {
            assert!(MethodDescriptor::parse(descriptor).is_err(), "{descriptor}");
        }
    }
}
