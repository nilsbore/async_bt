
use std::future::Future;
//use std::task::{LocalWaker, Waker};
//use std::future::{self, FutureExt};

pub fn run_bt<F: Future<Output=bool>>(mut tree : F)
{
    use core::{
        pin::Pin,
        ptr::NonNull,
        task::{LocalWaker, Poll, UnsafeWake, Waker},
    };

    struct NoWake;

    impl NoWake {
        fn local_waker() -> LocalWaker {
            // Safety: all references to NoWake are never
            // dereferenced
            unsafe { LocalWaker::new(NonNull::<NoWake>::dangling()) }
        }
    }

    unsafe impl UnsafeWake for NoWake {
        unsafe fn clone_raw(&self) -> Waker {
            NoWake::local_waker().into_waker()
        }
        unsafe fn drop_raw(&self) {}
        unsafe fn wake(&self) {}
    }

    //let tree: Box<dyn Future<Output=bool>> = Box::new(fallback_f1());

    let lw = NoWake::local_waker();
    let value = loop {
        // Safety: `future` is a local variable which is
        // only ever used in this pinned reference
        match unsafe { Pin::new_unchecked(&mut tree) }.poll(&lw) {
            Poll::Ready(value) => {
                println!("Got value: {}", value);
                break value;
            },
            Poll::Pending => {
                println!("Pending");
                continue;
            },
        }
    };
    
    
    //return value;
    //let mut pinned = fallback_f1.read_to_end();
    //loop {
        //while tree.poll(lw: &LocalWaker);
        //tree.
    //}
    //block_on(tree);

}