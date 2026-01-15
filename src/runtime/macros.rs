#[macro_export]
macro_rules! join {
    ( $future:expr $(,)? ) => {{ $future.await }};

    ( $f1:expr, $f2:expr $(,)? ) => {{
        use std::future;
        use std::future::Future;
        use std::pin::Pin;
        use std::task::{Context, Poll};

        let mut __f1 = (Box::pin($f1), None, false);
        let mut __f2 = (Box::pin($f2), None, false);

        future::poll_fn(move |cx: &mut Context<'_>| {
            if !__f1.2 {
                if let Poll::Ready(val) = __f1.0.as_mut().poll(cx) {
                    __f1.1 = Some(val);
                    __f1.2 = true;
                }
            }
            if !__f2.2 {
                if let Poll::Ready(val) = __f2.0.as_mut().poll(cx) {
                    __f2.1 = Some(val);
                    __f2.2 = true;
                }
            }
            if __f1.2 && __f2.2 {
                Poll::Ready((__f1.1.take().unwrap(), __f2.1.take().unwrap()))
            } else {
                Poll::Pending
            }
        })
        .await
    }};

    ( $f1:expr, $f2:expr, $f3:expr $(,)? ) => {{
        use std::future;
        use std::future::Future;
        use std::pin::Pin;
        use std::task::{Context, Poll};

        let mut __f1 = (Box::pin($f1), None, false);
        let mut __f2 = (Box::pin($f2), None, false);
        let mut __f3 = (Box::pin($f3), None, false);

        future::poll_fn(move |cx: &mut Context<'_>| {
            if !__f1.2 {
                if let Poll::Ready(val) = __f1.0.as_mut().poll(cx) {
                    __f1.1 = Some(val);
                    __f1.2 = true;
                }
            }
            if !__f2.2 {
                if let Poll::Ready(val) = __f2.0.as_mut().poll(cx) {
                    __f2.1 = Some(val);
                    __f2.2 = true;
                }
            }
            if !__f3.2 {
                if let Poll::Ready(val) = __f3.0.as_mut().poll(cx) {
                    __f3.1 = Some(val);
                    __f3.2 = true;
                }
            }
            if __f1.2 && __f2.2 && __f3.2 {
                Poll::Ready((
                    __f1.1.take().unwrap(),
                    __f2.1.take().unwrap(),
                    __f3.1.take().unwrap(),
                ))
            } else {
                Poll::Pending
            }
        })
        .await
    }};

    ( $f1:expr, $f2:expr, $f3:expr, $f4:expr $(,)? ) => {{
        use std::future;
        use std::future::Future;
        use std::pin::Pin;
        use std::task::{Context, Poll};

        let mut __f1 = (Box::pin($f1), None, false);
        let mut __f2 = (Box::pin($f2), None, false);
        let mut __f3 = (Box::pin($f3), None, false);
        let mut __f4 = (Box::pin($f4), None, false);

        future::poll_fn(move |cx: &mut Context<'_>| {
            if !__f1.2 {
                if let Poll::Ready(val) = __f1.0.as_mut().poll(cx) {
                    __f1.1 = Some(val);
                    __f1.2 = true;
                }
            }
            if !__f2.2 {
                if let Poll::Ready(val) = __f2.0.as_mut().poll(cx) {
                    __f2.1 = Some(val);
                    __f2.2 = true;
                }
            }
            if !__f3.2 {
                if let Poll::Ready(val) = __f3.0.as_mut().poll(cx) {
                    __f3.1 = Some(val);
                    __f3.2 = true;
                }
            }
            if !__f4.2 {
                if let Poll::Ready(val) = __f4.0.as_mut().poll(cx) {
                    __f4.1 = Some(val);
                    __f4.2 = true;
                }
            }
            if __f1.2 && __f2.2 && __f3.2 && __f4.2 {
                Poll::Ready((
                    __f1.1.take().unwrap(),
                    __f2.1.take().unwrap(),
                    __f3.1.take().unwrap(),
                    __f4.1.take().unwrap(),
                ))
            } else {
                Poll::Pending
            }
        })
        .await
    }};
}
