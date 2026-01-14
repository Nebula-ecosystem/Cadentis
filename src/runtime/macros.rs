#[macro_export]
macro_rules! join {
    ( $( let $name:ident = $fut:expr ),* $(,)? ) => {{
        $(
            let $name = $crate::task::spawn(async move { $fut.await });
        )*

        (
            $(
                $name.await
            ),*
        )
    }};
}
