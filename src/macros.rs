/// A macro to send a message to the game message log. Gets around borrowing by
/// automatically binding the provided arguments ahead of time. The syntax is
/// similar to format!, but with an ident and '=' before the expression, like:
///
/// ```no_run
/// mes!(world, "{}: {}", a=world.some_immut_fn(), b=world.some_mut_fn());
/// ```
///
/// It would be better if some temporary names could be dynamically generated
/// instead of having to provide them each time.
macro_rules! mes {
    ($w:expr) => {
        $w.next_message();
    };
    ($w:expr, $e:expr) => {
        $w.message($e);
    };
    ($w:expr, $e:expr, $( $x:ident=$y:expr ),+) => {
        use util::grammar;
        $(
            let $x = $y;
        )*;

        $w.message(&grammar::capitalize(&format!($e, $($x),+)));
    };
}

/// A macro to send a message to the game message log with formatting for the given entity. Certain
/// format specifiers can be used in the format string.
///
/// Available format specifiers:
/// - `%u`: Pronoun, "I/it/they"
/// - `%U`: Full name, "the putit"
/// - `%r`: Possessive, "my/its/their"
/// - `%R`: Named possessive, "the putit's"
/// - `%A`: Accusative, "me/it/them"
/// - `%%`: Literal '%'
/// - `<<`: Literal '<'
///
/// You can also conjugate verbs by surrounding the infinitive with angle brackets ("<>").
///
/// ```no_run
// format_mes!(world, entity, "%u <kill> {}! ({})", a = other.name(world), b = damage);
/// ```
macro_rules! format_mes {
    ($world:expr, $entity:expr, $format:expr) => {
        use util::format;
        let formatted = format::format_message($format, $entity, $world);
        $world.message(&formatted);
    };
    ($world:expr, $entity:expr, $format:expr, $( $x:ident=$y:expr ),+) => {
        use util::format;
        $(
            let $x = $y;
        )*;

        let raw = format!($format, $($x),+);

        let formatted = format::format_message(&raw, $entity, $world);
        $world.message(&formatted);
    };
}

/// A macro to open a UI window with a set of string choices. Called like:
///
///```no_run
/// menu!(context,
///       "foo" => do_foo(),
///       "bar" => do_bar(),
///       "bar" => do_baz(),
/// )
///```
macro_rules! menu {
    ($context:ident, $( $x:expr => $y:expr ),*) => {
        {
            let mut temp_vec = Vec::new();

            $(
                temp_vec.push($x.to_string());
            )*;

            match menu_choice($context, temp_vec) {
                Some(idx) => {
                    let mut i: usize = 0;

                    $(
                        if idx == i {
                            return $y;
                        }
                        i += 1;
                    )*

                        Err(CommandError::Cancel)
                },
                None => Err(CommandError::Cancel)
            }

        }
    }
}

/// A macro to pass variable assignments to Lua maps, to be run after the init() portion of map
/// generation.
///
/// ```no_run
/// map_args! { width: 80, height: 40 }
/// ```
macro_rules! prefab_args {
    {
        $($var:ident: $value:expr,)+
    } => {
        {

            use std::collections::HashMap;
            let mut res = HashMap::new();

            $(
                res.insert(stringify!($var).to_string(), stringify!($value).to_string());
            )*;

            res
        }
    }
}

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
