#[macro_export]
macro_rules! select {
    (
        $f1:expr => |$v1:pat_param| $r1:expr $(,)?
    ) => {{
        let $v1 = $f1.await;
        $r1
    }};

    (
        $f1:expr => |$v1:pat_param| $r1:expr,
        $f2:expr => |$v2:pat_param| $r2:expr $(,)?
    ) => {{
        use std::future::poll_fn;
        use std::task::Poll;

        enum __SelectResult<A, B> {
            F1(A),
            F2(B),
        }

        let mut f1 = Box::pin($f1);
        let mut f2 = Box::pin($f2);

        let res = poll_fn(move |cx| {
            if let Poll::Ready(val) = f1.as_mut().poll(cx) {
                return Poll::Ready(__SelectResult::F1(val));
            }
            if let Poll::Ready(val) = f2.as_mut().poll(cx) {
                return Poll::Ready(__SelectResult::F2(val));
            }
            Poll::Pending
        })
        .await;

        match res {
            __SelectResult::F1($v1) => $r1,
            __SelectResult::F2($v2) => $r2,
        }
    }};

    (
        $f1:expr => |$v1:pat_param| $r1:expr,
        $f2:expr => |$v2:pat_param| $r2:expr,
        $f3:expr => |$v3:pat_param| $r3:expr $(,)?
    ) => {{
        use std::future::poll_fn;
        use std::task::Poll;

        enum __SelectResult<A, B, C> {
            F1(A),
            F2(B),
            F3(C),
        }

        let mut f1 = Box::pin($f1);
        let mut f2 = Box::pin($f2);
        let mut f3 = Box::pin($f3);

        let res = poll_fn(move |cx| {
            if let Poll::Ready(val) = f1.as_mut().poll(cx) {
                return Poll::Ready(__SelectResult::F1(val));
            }
            if let Poll::Ready(val) = f2.as_mut().poll(cx) {
                return Poll::Ready(__SelectResult::F2(val));
            }
            if let Poll::Ready(val) = f3.as_mut().poll(cx) {
                return Poll::Ready(__SelectResult::F3(val));
            }
            Poll::Pending
        })
        .await;

        match res {
            __SelectResult::F1($v1) => $r1,
            __SelectResult::F2($v2) => $r2,
            __SelectResult::F3($v3) => $r3,
        }
    }};

    (
        $f1:expr => |$v1:pat_param| $r1:expr,
        $f2:expr => |$v2:pat_param| $r2:expr,
        $f3:expr => |$v3:pat_param| $r3:expr,
        $f4:expr => |$v4:pat_param| $r4:expr $(,)?
    ) => {{
        use std::future::poll_fn;
        use std::task::Poll;

        enum __SelectResult<A, B, C, D> {
            F1(A),
            F2(B),
            F3(C),
            F4(D),
        }

        let mut f1 = Box::pin($f1);
        let mut f2 = Box::pin($f2);
        let mut f3 = Box::pin($f3);
        let mut f4 = Box::pin($f4);

        let res = poll_fn(move |cx| {
            if let Poll::Ready(val) = f1.as_mut().poll(cx) {
                return Poll::Ready(__SelectResult::F1(val));
            }
            if let Poll::Ready(val) = f2.as_mut().poll(cx) {
                return Poll::Ready(__SelectResult::F2(val));
            }
            if let Poll::Ready(val) = f3.as_mut().poll(cx) {
                return Poll::Ready(__SelectResult::F3(val));
            }
            if let Poll::Ready(val) = f4.as_mut().poll(cx) {
                return Poll::Ready(__SelectResult::F4(val));
            }
            Poll::Pending
        })
        .await;

        match res {
            __SelectResult::F1($v1) => $r1,
            __SelectResult::F2($v2) => $r2,
            __SelectResult::F3($v3) => $r3,
            __SelectResult::F4($v4) => $r4,
        }
    }};
}
