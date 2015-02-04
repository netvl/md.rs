#[derive(Copy)]
pub struct MarkdownConfig {
    pub trim_newlines: bool
}

impl MarkdownConfig {
    #[inline]
    pub fn default() -> MarkdownConfig {
        MarkdownConfig {
            trim_newlines: true
        }
    }
}

macro_rules! impl_setters {
    ($target:ident; $($name:ident : $t:ty),+) => ($(
        impl $target {
            pub fn $name(mut self, value: $t) -> $target {
                self.$name = value;
                self
            }
        }
    )+)
}

impl_setters! { MarkdownConfig;
    trim_newlines: bool
}
