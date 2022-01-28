#[macro_export]
macro_rules! error_printer {
    ($e:expr) => {{
        println!("=== ERROR ===");
        dbg!($e);
    }};
}

pub fn pretty_large_int(x: impl Into<u128>) -> String {
    let mut num: u128 = x.into();
    let mut s = String::new();
    while num / 1000 != 0 {
        s = format!(",{:03}{}", num % 1000, s);
        num /= 1000;
    }
    format!("{}{}", num % 1000, s)
}

pub fn pretty_bytesize(x: impl Into<u128>) -> String {
    const SIZES: [&str; 9] = ["B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
    let mut bytes: u128 = x.into();
    if bytes < 1024 {
        return format!("{bytes}B");
    }

    let mut index: usize = 1;
    while index < 9 && bytes / 1024 >= 1024 {
        bytes /= 1024;
        index += 1;
    }

    if index < 9 {
        format!(
            "{}.{:02}{}",
            pretty_large_int(bytes / 1024),
            (bytes % 1024 + 5) / 10,
            SIZES[index]
        )
    } else {
        format!("{}YB", pretty_large_int(bytes))
    }
}
