#[macro_export]
macro_rules! register_errors {
    (
        $(
            $name:ident => $category:expr => $desc:expr
        ),* $(,)?
    ) => {
        $(
            $crate::registry::register_error(
                stringify!($name),
                $crate::registry::ErrorDescriptor {
                    name: stringify!($name),
                    category: $category,
                    description: $desc,
                }
            );
        )*
    };
}