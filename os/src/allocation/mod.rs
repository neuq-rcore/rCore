use core::ops::Deref;

#[derive(Clone, Copy)]
pub enum RefOrValue<'a, T> {
    Ref(&'a T),
    Value(T),
}

impl<'a, T> RefOrValue<'a, T> {
    pub fn from_ref(s: &'a T) -> Self {
        RefOrValue::Ref(s)
    }

    pub fn from_value(s: T) -> Self {
        RefOrValue::Value(s)
    }
}

impl<'a, T> RefOrValue<'a, T>
{
    pub fn as_ref(&self) -> &T {
        match self {
            RefOrValue::Ref(r) => r,
            RefOrValue::Value(v) => &v,
        }
    }
}

impl<T> Deref for RefOrValue<'_, T>
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
