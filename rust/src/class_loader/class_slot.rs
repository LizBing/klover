use crate::oops::klass::Klass;

pub struct ClassSlot<'cld> {
    klass: &'cld Klass<'cld>,
}

impl<'cld> ClassSlot<'cld> {
    pub fn klass(&self) -> &'cld Klass<'cld> {
        self.klass
    }

    pub fn new(klass: &'cld Klass<'cld>) -> Self {
        Self {
            klass,
        }
    }
}
