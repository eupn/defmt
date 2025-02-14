use core::fmt;

#[cfg(feature = "unstable-test")]
use crate as defmt;
use crate::{Format, Formatter};

/// An "adapter" type to feed `Debug` values into defmt macros, which expect `defmt::Format` values.
///
/// This adapter disables compression and uses the `core::fmt` code on-device! You should prefer
/// `defmt::Format` over `Debug` whenever possible.
///
/// # Examples
///
/// ```rust
/// # #[derive(Debug)]
/// # struct ExpensiveThing();
/// # let expensive_thing = ExpensiveThing();
/// #
/// defmt::info!("{:?}", defmt::Debug2Format(&expensive_thing));
/// //                                        ˆˆˆˆˆˆˆˆˆˆˆˆˆˆˆ
/// //                                        must `#[derive(Debug)]`
/// ```
///
/// Note that any provided defmt display hints will be ignored
/// because this always uses `{:?}` to format the contained value.
pub struct Debug2Format<'a, T: fmt::Debug + ?Sized>(pub &'a T);

impl<T: fmt::Debug + ?Sized> Format for Debug2Format<'_, T> {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = defmt_macros::internp!("{=__internal_Debug}");
            fmt.inner.tag(&t);
        }
        fmt.inner.debug(&self.0);
    }
}

/// An "adapter" type to feed `Display` values into defmt macros, which expect `defmt::Format` values.
///
/// This adapter disables compression and uses the `core::fmt` code on-device! You should prefer
/// `defmt::Format` over `Display` whenever possible.
///
/// # Examples
///
/// ```rust
/// # struct ExpensiveThing();
/// #
/// # impl core::fmt::Display for ExpensiveThing {
/// #     fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
/// #         write!(f, "{}", "expensive")
/// #     }
/// #  }
/// # let expensive_thing = ExpensiveThing();
/// #
/// defmt::info!("{}", defmt::Display2Format(&expensive_thing));
/// //                                        ˆˆˆˆˆˆˆˆˆˆˆˆˆˆˆ
/// //                                        must implement `fmt::Display`
/// ```
///
/// Note that any provided defmt display hints will be ignored
/// because this always uses `{}` to format the contained value.
pub struct Display2Format<'a, T: fmt::Display + ?Sized>(pub &'a T);

impl<T: fmt::Display + ?Sized> Format for Display2Format<'_, T> {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = defmt_macros::internp!("{=__internal_Display}");
            fmt.inner.tag(&t);
        }
        fmt.inner.display(&self.0);
    }
}
