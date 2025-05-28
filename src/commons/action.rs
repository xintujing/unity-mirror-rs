use std::marker::PhantomData;

pub trait Action<T> {
    fn invoke(&self, arguments: T);
}

pub trait Arguments {
    fn from_arguments() -> Self
    where
        Self: Sized;
}

pub trait Function<T>: 'static {
    fn call(&self, params: T);
}

#[rustfmt::skip]
mod _impl_function {
    use super::*;

    impl<F> Function<()> for F
    where
        F: Fn() -> () + 'static,
    {
        fn call(&self, params: ()) {
            (self)()
        }
    }

    macro_rules! f {
        (($($Ts:ident),*), ($($Ns:tt),*)) => {
            impl<F, $($Ts,)*> Function<( $($Ts, )* )> for F
            where
                F: Fn( $($Ts,)* ) -> () + 'static
            {
                fn call(&self, params: ( $($Ts,)* )) {
                    (self)(
                        $(params.$Ns, )*
                    )
                }
            }
        };
    }
    f!((T1), (0));
    f!((T1, T2), (0, 1));
    f!((T1, T2, T3), (0, 1, 2));
    f!((T1, T2, T3, T4), (0, 1, 2, 3));
    f!((T1, T2, T3, T4, T5), (0, 1, 2, 3, 4));
    f!((T1, T2, T3, T4, T5, T6), (0, 1, 2, 3, 4, 5));
    f!((T1, T2, T3, T4, T5, T6, T7), (0, 1, 2, 3, 4, 5, 6));
    f!((T1, T2, T3, T4, T5, T6, T7, T8), (0, 1, 2, 3, 4, 5, 6, 7));
    f!(
        (T1, T2, T3, T4, T5, T6, T7, T8, T9),
        (0, 1, 2, 3, 4, 5, 6, 7, 8)
    );
    f!(
        (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10),
        (0, 1, 2, 3, 4, 5, 6, 7, 8, 9)
    );
}

pub struct ActionWrapper<F, T> {
    f: F,
    _t: PhantomData<T>,
}
impl<F, T> ActionWrapper<F, T>
where
    F: Function<T>,
    T: Arguments,
{
    pub fn new(f: F) -> Box<Self> {
        Box::new(Self { f, _t: PhantomData })
    }
}
impl<F, T> Action<T> for ActionWrapper<F, T>
where
    F: Function<T>,
    T: Arguments,
{
    fn invoke(&self, arguments: T) {
        self.f.call(arguments);
    }
}

#[rustfmt::skip]
mod _impl_arguments {
    use super::*;

    impl Arguments for () {
        fn from_arguments() -> Self
        where
            Self: Sized,
        {
            ()
        }
    }
    macro_rules! f {
        ($($Ts:tt),*) => {
            impl< $($Ts,)* > Arguments for ( $($Ts,)* )
            where
                $(
                    $Ts: Arguments,
                )*
            {
                fn from_arguments() -> Self {
                    (
                        $(
                            $Ts::from_arguments(),
                        )*
                    )
                }
            }
        };
    }
    f!(T1);
    f!(T1, T2);
    f!(T1, T2, T3);
    f!(T1, T2, T3, T4);
    f!(T1, T2, T3, T4, T5);
    f!(T1, T2, T3, T4, T5, T6);
    f!(T1, T2, T3, T4, T5, T6, T7);
    f!(T1, T2, T3, T4, T5, T6, T7, T8);
    f!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
    f!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
}
