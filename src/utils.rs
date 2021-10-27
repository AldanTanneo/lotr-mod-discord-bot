#[macro_export]
macro_rules! error_printer {
    ($e:expr) => {{
        println!("=== ERROR ===");
        dbg!($e);
    }};
}
