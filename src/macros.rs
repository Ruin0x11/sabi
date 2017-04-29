macro_rules! make_global {
    ($name:ident, $global_ty:ty, $maker:expr) => {
        mod instance {
            use super::*;
            use std::cell::RefCell;
            thread_local!(static $name: RefCell<$global_ty> = RefCell::new($maker); );

            pub fn with<A, F>(f: F) -> A
                where F: FnOnce(&$global_ty) -> A {
                $name.with(|w| f(& *w.borrow()))
            }

            pub fn with_mut<A, F>(f: F) -> A
                where F: FnOnce(&mut $global_ty) -> A {
                $name.with(|w| f(&mut *w.borrow_mut()))
            }
        }
    }
}

#[macro_export]
macro_rules! crit_ecs(
    ($w:ident, $e:expr, #$tag:expr, $($args:tt)+) => {
        if let Some(l) = $w.ecs().logs.get($e) {
            crit!(l.logger, $tag, $($args)+);
        }
    };
    ($w:ident, $e:expr, $($args:tt)+) => {
        if let Some(l) = $w.ecs().logs.get($e) {
            crit!(l.logger, $($args)+);
        }
    };
);

#[macro_export]
macro_rules! error_ecs(
    ($w:ident, $e:expr, #$tag:expr, $($args:tt)+) => {
        if let Some(l) = $w.ecs().logs.get($e) {
            error!(l.logger, $tag, $($args)+);
        }
    };
    ($w:ident, $e:expr, $($args:tt)+) => {
        if let Some(l) = $w.ecs().logs.get($e) {
            error!(l.logger, $($args)+);
        }
    };
);

#[macro_export]
macro_rules! warn_ecs(
    ($w:ident, $e:expr, #$tag:expr, $($args:tt)+) => {
        if let Some(l) = $w.ecs().logs.get($e) {
            warn!(l.logger, $tag, $($args)+);
        }
    };
    ($w:ident, $e:expr, $($args:tt)+) => {
        if let Some(l) = $w.ecs().logs.get($e) {
            warn!(l.logger, $($args)+);
        }
    };
);

#[macro_export]
macro_rules! info_ecs(
    ($w:ident, $e:expr, #$tag:expr, $($args:tt)+) => {
        if let Some(l) = $w.ecs().logs.get($e) {
            info!(l.logger, $tag, $($args)+);
        }
    };
    ($w:ident, $e:expr, $($args:tt)+) => {
        if let Some(l) = $w.ecs().logs.get($e) {
            info!(l.logger, $($args)+);
        }
    };
);

#[macro_export]
macro_rules! debug_ecs(
    ($w:ident, $e:expr, #$tag:expr, $($args:tt)+) => {
        if let Some(l) = $w.ecs().logs.get($e) {
            debug!(l.logger, $tag, $($args)+);
        }
    };
    ($w:ident, $e:expr, $($args:tt)+) => {
        if let Some(l) = $w.ecs().logs.get($e) {
            debug!(l.logger, $($args)+);
        }
    };
);

#[macro_export]
macro_rules! trace_ecs(
    ($w:ident, $e:expr, #$tag:expr, $($args:tt)+) => {
        if let Some(l) = $w.ecs().logs.get($e) {
            trace!(l.logger, $tag, $($args)+);
        }
    };
    ($w:ident, $e:expr, $($args:tt)+) => {
        if let Some(l) = $w.ecs().logs.get($e) {
            trace!(l.logger, $($args)+);
        }
    };
);
