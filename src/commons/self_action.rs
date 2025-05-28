use std::marker::PhantomData;

pub trait SelfAction<S, T> {
    fn invoke(&self, this: &S, arguments: T); // 传入 &self 和参数
}

pub trait Arguments {
    fn from_arguments() -> Self
    where
        Self: Sized;
}

pub trait MethodFunction<S, T>: 'static {
    fn call(&self, this: &S, params: T); // 显式传入 &self（this）和参数
}

#[rustfmt::skip]
mod _impl_method_function {
    use super::*;

    // 无参数情况
    impl<S, F> MethodFunction<S, ()> for F
    where
        S: ?Sized,
        F: Fn(&S) -> () + 'static,
    {
        fn call(&self, this: &S, params: ()) {
            (self)(this)
        }
    }

    // 带参数的宏定义（支持最多 10 个参数）
    macro_rules! f {
        (($S:ident, $($Ts:ident),*), ($($Ns:tt),*)) => {
            impl<F, $S, $($Ts,)*> MethodFunction<$S, ( $($Ts, )* )> for F
            where
                $S: ?Sized,
                F: Fn(&$S, $($Ts,)* ) -> () + 'static
            {
                fn call(&self, this: &$S, params: ( $($Ts,)* )) {
                    (self)(this, $(params.$Ns, )*)
                }
            }
        };
    }

    // 展开宏（注意第一个参数为 S，表示 &self 的类型）
    f!((S, T1), (0));
    f!((S, T1, T2), (0, 1));
    f!((S, T1, T2, T3), (0, 1, 2));
    // ... 继续展开到 10 个参数
}

pub struct SelfActionWrapper<S, F, T> {
    f: F,
    _this: PhantomData<&'static S>, // 标记 &self 的生命周期
    _t: PhantomData<T>,
}
impl<S, F, T> SelfActionWrapper<S, F, T>
where
    S: ?Sized,
    F: MethodFunction<S, T>,
    T: Arguments,
{
    pub fn new(f: F) -> Box<Self> {
        Box::new(Self {
            f,
            _this: PhantomData,
            _t: PhantomData,
        })
    }
}

impl<S, F, T> SelfAction<S, T> for SelfActionWrapper<S, F, T>
where
    S: ?Sized,
    F: MethodFunction<S, T>,
    T: Arguments,
{
    fn invoke(&self, this: &S, arguments: T) {
        self.f.call(this, arguments); // 传入 &self 和参数
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
