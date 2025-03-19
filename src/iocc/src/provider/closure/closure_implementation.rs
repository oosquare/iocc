use std::error::Error;

use crate::container::injector::Injector;
use crate::container::Managed;
use crate::key;
use crate::prelude::{InjectorError, TypedInjector};
use crate::provider::closure::Closure;

impl<F, T, E> Closure<()> for F
where
    F: Fn() -> Result<T, E> + Send + Sync + 'static,
    T: Managed,
    E: Into<Box<dyn Error + Send + Sync>>,
{
    type Constructed = T;

    type Error = E;

    fn run(
        &self,
        _injector: &dyn Injector,
    ) -> Result<Result<Self::Constructed, Self::Error>, InjectorError> {
        Ok(self())
    }
}

macro_rules! for_all_tuples {
    ($implementation:ident) => {
        $implementation!(D1);
        $implementation!(D1, D2);
        $implementation!(D1, D2, D3);
        $implementation!(D1, D2, D3, D4);
        $implementation!(D1, D2, D3, D4, D5);
        $implementation!(D1, D2, D3, D4, D5, D6);
        $implementation!(D1, D2, D3, D4, D5, D6, D7);
        $implementation!(D1, D2, D3, D4, D5, D6, D7, D8);
        $implementation!(D1, D2, D3, D4, D5, D6, D7, D8, D9);
        $implementation!(D1, D2, D3, D4, D5, D6, D7, D8, D9, D10);
        $implementation!(D1, D2, D3, D4, D5, D6, D7, D8, D9, D10, D11);
        $implementation!(D1, D2, D3, D4, D5, D6, D7, D8, D9, D10, D11, D12);
        $implementation!(D1, D2, D3, D4, D5, D6, D7, D8, D9, D10, D11, D12, D13);
        $implementation!(D1, D2, D3, D4, D5, D6, D7, D8, D9, D10, D11, D12, D13, D14);
        $implementation!(D1, D2, D3, D4, D5, D6, D7, D8, D9, D10, D11, D12, D13, D14, D15);
        $implementation!(D1, D2, D3, D4, D5, D6, D7, D8, D9, D10, D11, D12, D13, D14, D15, D16);
    };
}

macro_rules! impl_closure {
    ($($dep:ident),*) => {
        #[allow(non_snake_case)]
        impl<F, T, E, $($dep,)*> Closure<($($dep,)*)> for F
        where
            F: Fn($($dep,)*) -> Result<T, E> + Send + Sync + 'static,
            T: Managed,
            E: Into<Box<dyn Error + Send + Sync>>,
            $($dep: Managed,)*
        {
            type Constructed = T;

            type Error = E;

            fn run(
                &self,
                injector: &dyn Injector,
            ) -> Result<Result<Self::Constructed, Self::Error>, InjectorError> {
                $(
                    let $dep = injector.get(key::of())?;
                )*
                Ok(self($($dep,)*))
            }
        }
    };
}

for_all_tuples!(impl_closure);

#[cfg(test)]
mod tests {
    use std::convert::Infallible;

    use crate::container::injector::MockInjector;

    use super::*;

    #[test]
    fn closure_implementation_compilation_succeeds() {
        #[allow(dead_code)]
        fn closure_with_four_parameters() {
            let injector = MockInjector::new();
            let closure = |_: i32, _: i32, _: i32, _: i32| Ok::<_, Infallible>("str");
            let _ = closure.run(&injector);
        }

        #[allow(dead_code)]
        fn closure_with_no_parameter() {
            let injector = MockInjector::new();
            let closure = || Ok::<_, Infallible>("str");
            let _ = closure.run(&injector);
        }
    }
}
