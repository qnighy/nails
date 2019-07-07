#[macro_export]
macro_rules! assert_ts_eq {
    ($lhs:expr, $rhs:expr) => {{
        let lhs: TokenStream = $lhs;
        let rhs: TokenStream = $rhs;
        if lhs.to_string() != rhs.to_string() {
            panic!(
                r#"assertion failed: `(left == right)`
left:
```
{}
```

right: ```
{}
```
"#,
                synstructure::unpretty_print(&lhs),
                synstructure::unpretty_print(&rhs),
            );
        }
    }};
    ($lhs:expr, $rhs:expr,) => {
        assert_ts_eq!($lhs, $rhs)
    };
}
