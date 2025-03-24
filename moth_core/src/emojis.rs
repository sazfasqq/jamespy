use serenity::small_fixed_array::FixedString;

macro_rules! define_symbol_constants {
    ($($name:ident => $value:expr),*) => {
        $(
            pub struct $name(pub &'static str);

            impl $name {
                #[must_use] pub fn to_fixed_string() -> FixedString {
                    FixedString::from_str_trunc($value)
                }
            }
        )*
    };
}

define_symbol_constants! {
    Anger => "💢",
    Question => "❓",
    Checkmark => "✅",
    X => "❌"
}
