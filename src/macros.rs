#![macro_escape]

macro_rules! one_of(
    ($c:expr, $p:expr) => ($c == $p);
    ($c:expr, $p:expr, $($rest:expr),+) => (
        ($c == $p || one_of!($c, $($rest),+))
    )
)
