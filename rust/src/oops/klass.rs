use crate::oops::normal_klass::NormalKlass;

#[derive(Debug)]
pub enum Klass<'cld> {
    Normal(NormalKlass<'cld>),
}

impl<'cld> Klass<'cld> {
    pub fn as_normal_klass(&self) -> Option<&NormalKlass<'cld>> {
        match self {
            Self::Normal(klass) => Some(klass),
        }
    }
}
