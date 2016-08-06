/// Coroutines, similar to iterators but with additional arguments to `send`/`next`.
pub trait Coroutine<Args> {
    type Item;

    fn send(&mut self, args: Args) -> Option<Self::Item>;
}


/// For-in loop for coroutines, passing additional arguments on each iteration.
#[macro_export]
macro_rules! co_for {
    ($x:pat in ($xs:expr) ($($args:expr),*) $b:block) => {{
        let mut gen = $xs;
        while let Some(item) = $crate::coroutine::Coroutine::send(&mut gen, ($($args,)*)) {
            let $x = item;
            $b;
        }
    }};
}
