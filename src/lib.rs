//! A highly efficient logging framework that targets resource-constrained devices, like
//! microcontrollers.
//!
//! Check out the defmt book at <https://defmt.ferrous-systems.com> for more information about how
//! to use it.
//!
//! # Compatibility
//!
//! The `defmt` wire format might change between major versions. Attempting to read a defmt stream
//! with an incompatible version will result in an error. This means that you have to update both
//! the host and target side if a breaking change in defmt is released.

#![cfg_attr(not(feature = "unstable-test"), no_std)]
// NOTE if you change this URL you'll also need to update all other crates in this repo
#![doc(html_logo_url = "https://knurling.ferrous-systems.com/knurling_logo_light_text.svg")]
#![warn(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod adapter;
#[doc(hidden)]
pub mod export;
mod formatter;
mod impls;
#[cfg(all(test, feature = "unstable-test"))]
mod tests;
mod traits;

pub use crate::{
    adapter::{Debug2Format, Display2Format},
    formatter::{Formatter, InternalFormatter, Str},
    traits::{Format, Logger},
};

#[cfg(all(test, not(feature = "unstable-test")))]
compile_error!(
    "to run unit tests enable the `unstable-test` feature, e.g. `cargo t --features unstable-test`"
);

/// Just like the [`core::assert!`] macro but `defmt` is used to log the panic message
///
/// [`core::assert!`]: https://doc.rust-lang.org/core/macro.assert.html
///
/// If used, the format string must follow the defmt syntax (documented in [the manual])
///
/// [the manual]: https://defmt.ferrous-systems.com/macros.html
pub use defmt_macros::assert_ as assert;

/// Just like the [`core::assert_eq!`] macro but `defmt` is used to log the panic message
///
/// [`core::assert_eq!`]: https://doc.rust-lang.org/core/macro.assert_eq.html
///
/// If used, the format string must follow the defmt syntax (documented in [the manual])
///
/// [the manual]: https://defmt.ferrous-systems.com/macros.html
pub use defmt_macros::assert_eq_ as assert_eq;

/// Just like the [`core::assert_ne!`] macro but `defmt` is used to log the panic message
///
/// [`core::assert_ne!`]: https://doc.rust-lang.org/core/macro.assert_ne.html
///
/// If used, the format string must follow the defmt syntax (documented in [the manual])
///
/// [the manual]: https://defmt.ferrous-systems.com/macros.html
pub use defmt_macros::assert_ne_ as assert_ne;

/// Just like the [`core::debug_assert!`] macro but `defmt` is used to log the panic message
///
/// [`core::debug_assert!`]: https://doc.rust-lang.org/core/macro.debug_assert.html
///
/// If used, the format string must follow the defmt syntax (documented in [the manual])
///
/// [the manual]: https://defmt.ferrous-systems.com/macros.html
pub use defmt_macros::debug_assert_ as debug_assert;

/// Just like the [`core::debug_assert_eq!`] macro but `defmt` is used to log the panic message
///
/// [`core::debug_assert_eq!`]: https://doc.rust-lang.org/core/macro.debug_assert_eq.html
///
/// If used, the format string must follow the defmt syntax (documented in [the manual])
///
/// [the manual]: https://defmt.ferrous-systems.com/macros.html
pub use defmt_macros::debug_assert_eq_ as debug_assert_eq;

/// Just like the [`core::debug_assert_ne!`] macro but `defmt` is used to log the panic message
///
/// [`core::debug_assert_ne!`]: https://doc.rust-lang.org/core/macro.debug_assert_ne.html
///
/// If used, the format string must follow the defmt syntax (documented in [the manual])
///
/// [the manual]: https://defmt.ferrous-systems.com/macros.html
pub use defmt_macros::debug_assert_ne_ as debug_assert_ne;

/// Just like the [`core::unreachable!`] macro but `defmt` is used to log the panic message
///
/// [`core::unreachable!`]: https://doc.rust-lang.org/core/macro.unreachable.html
///
/// If used, the format string must follow the defmt syntax (documented in [the manual])
///
/// [the manual]: https://defmt.ferrous-systems.com/macros.html
pub use defmt_macros::unreachable_ as unreachable;

/// Just like the [`core::todo!`] macro but `defmt` is used to log the panic message
///
/// [`core::todo!`]: https://doc.rust-lang.org/core/macro.todo.html
///
/// If used, the format string must follow the defmt syntax (documented in [the manual])
///
/// [the manual]: https://defmt.ferrous-systems.com/macros.html
pub use defmt_macros::todo_ as todo;

/// Just like the [`core::unimplemented!`] macro but `defmt` is used to log the panic message
///
/// [`core::unimplemented!`]: https://doc.rust-lang.org/core/macro.unimplemented.html
///
/// If used, the format string must follow the defmt syntax (documented in [the manual])
///
/// [the manual]: https://defmt.ferrous-systems.com/macros.html
pub use defmt_macros::todo_ as unimplemented;

/// Just like the [`core::panic!`] macro but `defmt` is used to log the panic message
///
/// [`core::panic!`]: https://doc.rust-lang.org/core/macro.panic.html
///
/// If used, the format string must follow the defmt syntax (documented in [the manual])
///
/// [the manual]: https://defmt.ferrous-systems.com/macros.html
pub use defmt_macros::panic_ as panic;

/// Unwraps an `Option` or `Result`, panicking if it is `None` or `Err`.
///
/// This macro is roughly equivalent to `{Option,Result}::{expect,unwrap}` but invocation looks
/// a bit different because this is a macro and not a method. The other difference is that
/// `unwrap!`-ing a `Result<T, E>` value requires that the error type `E` implements the `Format`
/// trait
///
/// The following snippet shows the differences between core's unwrap method and defmt's unwrap
/// macro:
///
/// ```
/// use defmt::unwrap;
///
/// # let option = Some(());
/// let x = option.unwrap();
/// let x = unwrap!(option);
///
/// # let result = Ok::<(), ()>(());
/// let x = result.unwrap();
/// let x = unwrap!(result);
///
/// # let value = result;
/// let x = value.expect("text");
/// let x = unwrap!(value, "text");
///
/// # let arg = ();
/// let x = value.expect(&format!("text {:?}", arg));
/// let x = unwrap!(value, "text {:?}", arg); // arg must be implement `Format`
/// ```
///
/// If used, the format string must follow the defmt syntax (documented in [the manual])
///
/// [the manual]: https://defmt.ferrous-systems.com/macros.html
pub use defmt_macros::unwrap;

/// Overrides the panicking behavior of `defmt::panic!`
///
/// By default, `defmt::panic!` calls `core::panic!` after logging the panic message using `defmt`.
/// This can result in the panic message being printed twice in some cases. To avoid that issue use
/// this macro. See [the manual] for details.
///
/// [the manual]: https://defmt.ferrous-systems.com/panic.html
///
/// # Inter-operation with built-in attributes
///
/// This attribute cannot be used together with the `export_name` or `no_mangle` attributes
pub use defmt_macros::panic_handler;

/// Creates an interned string ([`Str`]) from a string literal.
///
/// This must be called on a string literal, and will allocate the literal in the object file. At
/// runtime, only a small string index is required to refer to the string, represented as the
/// [`Str`] type.
///
/// # Example
///
/// ```
/// let interned = defmt::intern!("long string literal taking up little space");
/// ```
///
/// [`Str`]: struct.Str.html
pub use defmt_macros::intern;

/// Logs data at *debug* level.
///
/// Please refer to [the manual] for documentation on the syntax.
///
/// [the manual]: https://defmt.ferrous-systems.com/macros.html
pub use defmt_macros::debug;
/// Logs data at *error* level.
///
/// Please refer to [the manual] for documentation on the syntax.
///
/// [the manual]: https://defmt.ferrous-systems.com/macros.html
pub use defmt_macros::error;
/// Logs data at *info* level.
///
/// Please refer to [the manual] for documentation on the syntax.
///
/// [the manual]: https://defmt.ferrous-systems.com/macros.html
pub use defmt_macros::info;
/// Logs data at *trace* level.
///
/// Please refer to [the manual] for documentation on the syntax.
///
/// [the manual]: https://defmt.ferrous-systems.com/macros.html
pub use defmt_macros::trace;
/// Logs data at *warn* level.
///
/// Please refer to [the manual] for documentation on the syntax.
///
/// [the manual]: https://defmt.ferrous-systems.com/macros.html
pub use defmt_macros::warn;

/// Just like the [`std::dbg!`] macro but `defmt` is used to log the message at `TRACE` level.
///
/// [`std::dbg!`]: https://doc.rust-lang.org/std/macro.dbg.html
pub use defmt_macros::dbg;

/// Writes formatted data to a [`Formatter`].
///
/// [`Formatter`]: struct.Formatter.html
pub use defmt_macros::write;

/// Defines the global defmt logger.
///
/// `#[global_logger]` needs to be put on a unit struct type declaration. This struct has to
/// implement the [`Logger`] trait.
///
/// # Example
///
/// ```
/// use defmt::{Logger, global_logger};
///
/// #[global_logger]
/// struct MyLogger;
///
/// unsafe impl Logger for MyLogger {
///     fn acquire() {
/// # todo!()
///         // ...
///     }
///     unsafe fn release() {
/// # todo!()
///         // ...
///     }
///     unsafe fn write(bytes: &[u8]) {
/// # todo!()
///         // ...
///     }
/// }
/// ```
///
/// [`Logger`]: trait.Logger.html
pub use defmt_macros::global_logger;

/// Defines the global timestamp provider for defmt.
///
/// This macro can be used to attach a timestamp or other data to every defmt message. Its syntax
/// works exactly like the logging macros, except that no local variables can be accessed and the
/// macro should be placed in a module instead of a function.
///
/// `timestamp!` must only be used once across the crate graph.
///
/// If no crate defines a timestamp, no timestamp will be included in the logged messages.
///
/// # Examples
///
/// ```
/// # use core::sync::atomic::{AtomicU32, Ordering};
///
/// static COUNT: AtomicU32 = AtomicU32::new(0);
/// defmt::timestamp!("{=u32:µs}", COUNT.fetch_add(1, Ordering::Relaxed));
/// ```
pub use defmt_macros::timestamp;

#[doc(hidden)] // documented as the `Format` trait instead
pub use defmt_macros::Format;

#[export_name = "__defmt_default_timestamp"]
fn default_timestamp(_f: Formatter<'_>) {
    // By default, no timestamp is used.
}

// There is no default timestamp format. Instead, the decoder looks for a matching ELF symbol. If
// absent, timestamps are turned off.

#[export_name = "__defmt_default_panic"]
fn default_panic() -> ! {
    core::panic!()
}
