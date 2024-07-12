#[macro_export]
macro_rules! if_or {
    ($cond: expr, $if: expr) => {
        if $cond { $if } else {}
    };
    ($cond: expr, $if: expr, $else: expr) => {
        if $cond { $if } else { $else }
    };
}

/// Evaluate expressions in the order they are passed, and return the result of the last expression.
#[macro_export]
macro_rules! seq {
    () => { {} };
    ($expr: expr) => { $expr };
    ($expr: expr, $($rest: expr),+) => {
        {
            $expr;
            seq!($($rest),+)
        }
    };
}
