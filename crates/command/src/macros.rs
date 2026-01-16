#[macro_export]
macro_rules! command {
    ($registry:expr, $name:literal, { $($body:tt)* }) => {
        $registry.register($name, |node| {
            $crate::command_body!(node, $($body)*);
        });
    };
}

#[macro_export]
macro_rules! command_body {
    // --- LITERALS ---

    // literal "name" { ... }
    ($node:expr, literal $name:literal { $($children:tt)* } $($rest:tt)*) => {
        $node.literal($name, |sub_node| {
            $crate::command_body!(sub_node, $($children)*);
        });
        $crate::command_body!($node, $($rest)*);
    };

    // literal "name" executes |ctx| ...
    ($node:expr, literal $name:literal executes $closure:expr $(, $($rest:tt)*)?) => {
        $node.literal($name, |sub_node| {
            sub_node.executes($closure);
        });
        $crate::command_body!($node, $($($rest)*)?);
    };

    // --- ARGUMENTS (TYPE INFERENCE) ---
    // This matches: argument "age" (i32) ...
    // It automatically creates <i32 as Parsable>::Parser::default()

    // argument "name" (Type) { ... }
    ($node:expr, argument $name:literal ($t:ty) { $($children:tt)* } $($rest:tt)*) => {
        $node.argument($name, Box::new(<$t as $crate::args::Parsable>::Parser::default()), |sub_node| {
            $crate::command_body!(sub_node, $($children)*);
        });
        $crate::command_body!($node, $($rest)*);
    };

    // argument "name" (Type) executes ...
    ($node:expr, argument $name:literal ($t:ty) executes $closure:expr $(, $($rest:tt)*)?) => {
        $node.argument($name, Box::new(<$t as $crate::args::Parsable>::Parser::default()), |sub_node| {
            sub_node.executes($closure);
        });
        $crate::command_body!($node, $($($rest)*)?);
    };

    // --- EXECUTES ---
    ($node:expr, executes $closure:expr) => {
        $node.executes($closure);
    };

    ($node:expr, ) => {};
}