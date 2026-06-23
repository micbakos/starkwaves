use strum::VariantArray;

pub trait MenuIterable: VariantArray + PartialEq + Copy {
    fn first() -> Self {
        Self::VARIANTS[0]
    }

    fn next(self) -> Self {
        let index = Self::VARIANTS.iter().position(|v| *v == self).unwrap();

        let next = (index + 1) % Self::VARIANTS.len();
        Self::VARIANTS[next]
    }

    fn prev(self) -> Self {
        let count = Self::VARIANTS.len();
        let index = Self::VARIANTS.iter().position(|v| *v == self).unwrap();
        let prev = (index + count - 1) % count;

        Self::VARIANTS[prev]
    }
}

/// Blanket impl for all `VariantArray`s
impl<T> MenuIterable for T where T: VariantArray + PartialEq + Copy {}
