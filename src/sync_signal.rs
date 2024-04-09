use crate::core::UseRwSignal;
use default_struct_builder::DefaultBuilder;
use leptos::*;
use std::fmt::Debug;
use std::rc::Rc;

/// Two-way Signals synchronization.
///
/// > Note: Please consider first if you can achieve your goals with the
/// > ["Good Options" described in the Leptos book](https://book.leptos.dev/reactivity/working_with_signals.html#making-signals-depend-on-each-other)
/// > firstly. Only if you really have to, use this function. This is in effect the
/// > ["If you really must..."](https://book.leptos.dev/reactivity/working_with_signals.html#if-you-really-must).
///
/// ## Demo
///
/// [Link to Demo](https://github.com/Synphonyte/leptos-use/tree/main/examples/sync_signal)
///
/// ## Usage
///
/// ```
/// # use leptos::*;
/// # use leptos_use::sync_signal;
/// #
/// # #[component]
/// # fn Demo() -> impl IntoView {
/// let (a, set_a) = create_signal(1);
/// let (b, set_b) = create_signal(2);
///
/// let stop = sync_signal((a, set_a), (b, set_b));
///
/// logging::log!("a: {}, b: {}", a.get(), b.get()); // a: 1, b: 1
///
/// set_b.set(3);
///
/// logging::log!("a: {}, b: {}", a.get(), b.get()); // a: 3, b: 3
///
/// set_a.set(4);
///
/// logging::log!("a: {}, b: {}", a.get(), b.get()); // a: 4, b: 4
/// #
/// # view! { }
/// # }
/// ```
///
/// ### `RwSignal`
///
/// You can mix and match `RwSignal`s and `Signal`-`WriteSignal` pairs.
///
/// ```
/// # use leptos::*;
/// # use leptos_use::sync_signal;
/// #
/// # #[component]
/// # fn Demo() -> impl IntoView {
/// let (a, set_a) = create_signal(1);
/// let (b, set_b) = create_signal(2);
/// let c_rw = create_rw_signal(3);
/// let d_rw = create_rw_signal(4);
///
/// sync_signal((a, set_a), c_rw);
/// sync_signal(d_rw, (b, set_b));
/// sync_signal(c_rw, d_rw);
///
/// #
/// # view! { }
/// # }
/// ```
///
/// ### One directional
///
/// You can synchronize a signal only from left to right or right to left.
///
/// ```
/// # use leptos::*;
/// # use leptos_use::{sync_signal_with_options, SyncSignalOptions, SyncDirection};
/// #
/// # #[component]
/// # fn Demo() -> impl IntoView {
/// let (a, set_a) = create_signal(1);
/// let (b, set_b) = create_signal(2);
///
/// let stop = sync_signal_with_options(
///     (a, set_a),
///     (b, set_b),
///     SyncSignalOptions::default().direction(SyncDirection::LeftToRight)
/// );
///
/// set_b.set(3); // doesn't sync
///
/// logging::log!("a: {}, b: {}", a.get(), b.get()); // a: 1, b: 3
///
/// set_a.set(4);
///
/// logging::log!("a: {}, b: {}", a.get(), b.get()); // a: 4, b: 4
/// #
/// # view! { }
/// # }
/// ```
///
/// ### Custom Transform
///
/// You can optionally provide custom transforms between the two signals.
/// ```
/// # use leptos::*;
/// # use leptos_use::{sync_signal_with_options, SyncSignalOptions};
/// #
/// # #[component]
/// # fn Demo() -> impl IntoView {
/// let (a, set_a) = create_signal(10);
/// let (b, set_b) = create_signal(2);
///
/// let stop = sync_signal_with_options(
///     (a, set_a),
///     (b, set_b),
///     SyncSignalOptions::default()
///         .transform_ltr(|left| *left * 2)
///         .transform_rtl(|right| *right / 2)
/// );
///
/// logging::log!("a: {}, b: {}", a.get(), b.get()); // a: 10, b: 20
///
/// set_b.set(30);
///
/// logging::log!("a: {}, b: {}", a.get(), b.get()); // a: 15, b: 30
/// #
/// # view! { }
/// # }
/// ```
pub fn sync_signal<T>(
    left: impl Into<UseRwSignal<T>>,
    right: impl Into<UseRwSignal<T>>,
) -> impl Fn() + Clone
where
    T: Clone + PartialEq + 'static,
{
    sync_signal_with_options(left, right, SyncSignalOptions::default())
}

/// Version of [`sync_signal`] that takes a `SyncSignalOptions`. See [`sync_signal`] for how to use.
pub fn sync_signal_with_options<L, R>(
    left: impl Into<UseRwSignal<L>>,
    right: impl Into<UseRwSignal<R>>,
    options: SyncSignalOptions<L, R>,
) -> impl Fn() + Clone
where
    L: Clone + PartialEq + 'static,
    R: Clone + PartialEq + 'static,
{
    let SyncSignalOptions {
        immediate,
        direction,
        transform_ltr,
        transform_rtl,
    } = options;

    let left = left.into();
    let right = right.into();

    let mut stop_watch_left = None;
    let mut stop_watch_right = None;

    if matches!(direction, SyncDirection::Both | SyncDirection::LeftToRight) {
        stop_watch_left = Some(watch(
            move || left.get(),
            move |new_value, _, _| {
                let new_value = (*transform_ltr)(new_value);

                if right.with_untracked(|right| right != &new_value) {
                    right.update(|right| *right = new_value);
                }
            },
            immediate,
        ));
    }

    if matches!(direction, SyncDirection::Both | SyncDirection::RightToLeft) {
        stop_watch_right = Some(watch(
            move || right.get(),
            move |new_value, _, _| {
                let new_value = (*transform_rtl)(new_value);

                if left.with_untracked(|left| left != &new_value) {
                    left.update(|left| *left = new_value);
                }
            },
            immediate,
        ));
    }

    move || {
        if let Some(stop_watch_left) = &stop_watch_left {
            stop_watch_left();
        }
        if let Some(stop_watch_right) = &stop_watch_right {
            stop_watch_right();
        }
    }
}

/// Direction of syncing.
pub enum SyncDirection {
    LeftToRight,
    RightToLeft,
    Both,
}

/// Options for [`sync_signal_with_options`].
#[derive(DefaultBuilder)]
pub struct SyncSignalOptions<L, R> {
    /// If `true`, the signals will be immediately synced when this function is called.
    /// If `false`, a signal is only updated when the other signal's value changes.
    /// Defaults to `true`.
    immediate: bool,

    /// Direction of syncing. Defaults to `SyncDirection::Both`.
    direction: SyncDirection,

    /// Transforms the left signal into the right signal.
    /// Defaults to identity.
    #[builder(skip)]
    transform_ltr: Rc<dyn Fn(&L) -> R>,

    /// Transforms the right signal into the left signal.
    /// Defaults to identity.
    #[builder(skip)]
    transform_rtl: Rc<dyn Fn(&R) -> L>,
}

impl<L, R> SyncSignalOptions<L, R> {
    /// Transforms the left signal into the right signal.
    /// Defaults to identity.
    pub fn transform_ltr(self, transform_ltr: impl Fn(&L) -> R + 'static) -> Self {
        Self {
            transform_ltr: Rc::new(transform_ltr),
            ..self
        }
    }

    /// Transforms the right signal into the left signal.
    /// Defaults to identity.
    pub fn transform_rtl(self, transform_rtl: impl Fn(&R) -> L + 'static) -> Self {
        Self {
            transform_rtl: Rc::new(transform_rtl),
            ..self
        }
    }
}

impl<T: Clone> Default for SyncSignalOptions<T, T> {
    fn default() -> Self {
        Self {
            immediate: true,
            direction: SyncDirection::Both,
            transform_ltr: Rc::new(|x| x.clone()),
            transform_rtl: Rc::new(|x| x.clone()),
        }
    }
}
