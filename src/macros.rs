/// A macro to send a message to the game message log. The syntax is
/// similar to format!.
///
/// ```no_run
/// mes!(world, "{}: {}", world.some_immut_fn(), world.some_mut_fn());
/// ```
macro_rules! mes {
    ($w:expr) => {
        $w.next_message();
    };
    ($w:expr, $e:expr) => {
        $w.message($e);
    };
    ($w:expr, $e:expr, $( $y:expr ),+) => {
        use util::grammar;

        $w.message(&grammar::capitalize(&format!($e, $($y),+)));
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
// format_mes!(world, entity, "%u <kill> {}! ({})", other.name(world), damage);
/// ```
macro_rules! format_mes {
    ($world:expr, $entity:expr, $format:expr) => {
        use util::format;
        let formatted = format::format_message($format, $entity, $world);
        $world.message(&formatted);
    };
    ($world:expr, $entity:expr, $format:expr, $( $y:expr ),+) => {
        use util::format;

        let raw = format!($format, $($y),+);

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
#[allow(unused_assignments)]
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
    ( $($var:ident: $value:expr,)+ $(,)*)=> {
        {

            use std::collections::HashMap;
            let mut res = HashMap::new();

            $(
                res.insert(stringify!($var).to_string(), $value.to_string());
            )*;

            res
        }
    }
}

macro_rules! make_global {
    ($name:ident, $global_ty:ty, $maker:expr) => {
        pub(super) mod instance {
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


use toml;
pub trait Getter<T: Default> {
    fn get_for(table: &toml::value::Table) -> Result<T, ()>;
}

pub struct TomlInstantiate;

/// A macro for allowing structs to be deserialized from TOML, like so:
///
///```toml
/// [Meta]
/// parent="mob"
///
/// [Components.Name]
/// name="putit"
/// proper_name="Dood"
///
/// [Components.Health]
/// max_hit_points=100
/// hit_points=100
///```
///
/// The only requirement is that the struct has to implement Default. Omitted fields use the
/// values supplied by Default.
///
/// # Usage
///
///```no_run
/// make_getter!(Name {
///     name: String,
///     proper_name: Option<String>,
///     gender: Gender,
/// });
///```
#[macro_export]
macro_rules! make_getter {
    ($s:ident { $( $x:ident: $y:ty ),+ $(,)* } ) => {
        #[derive(Clone, Debug, Serialize, Deserialize)]
        pub struct $s {
            $( pub $x: $y ),+
        }

        impl Getter<$s> for TomlInstantiate {
            fn get_for(table: &toml::value::Table) -> Result<$s, ()> {
                let mut component: $s = Default::default();
                $(
                    if table.contains_key(stringify!($x)) {
                        match table[stringify!($x)].clone().try_into::<$y>() {
                            Ok(val) => component.$x = val,
                            Err(_) => return Err(())
                        }
                    }
                )*

                    Ok(component)
            }
        }
    }
}

#[macro_export]
macro_rules! crit_ecs(
    ($w:ident, $e:expr, #$tag:expr, $($args:tt)+) => {
        if let Some(l) = $w.ecs().logs.get($e) {
            crit!(l.get(), $tag, $($args)+);
        }
    };
    ($w:ident, $e:expr, $($args:tt)+) => {
        if let Some(l) = $w.ecs().logs.get($e) {
            crit!(l.get(), $($args)+);
        }
    };
);

#[macro_export]
macro_rules! error_ecs(
    ($w:ident, $e:expr, #$tag:expr, $($args:tt)+) => {
        if let Some(l) = $w.ecs().logs.get($e) {
            error!(l.get(), $tag, $($args)+);
        }
    };
    ($w:ident, $e:expr, $($args:tt)+) => {
        if let Some(l) = $w.ecs().logs.get($e) {
            error!(l.get(), $($args)+);
        }
    };
);

#[macro_export]
macro_rules! warn_ecs(
    ($w:ident, $e:expr, #$tag:expr, $($args:tt)+) => {
        if let Some(l) = $w.ecs().logs.get($e) {
            warn!(l.get(), $tag, $($args)+);
        }
    };
    ($w:ident, $e:expr, $($args:tt)+) => {
        if let Some(l) = $w.ecs().logs.get($e) {
            warn!(l.get(), $($args)+);
        }
    };
);

#[macro_export]
macro_rules! info_ecs(
    ($w:ident, $e:expr, #$tag:expr, $($args:tt)+) => {
        if let Some(l) = $w.ecs().logs.get($e) {
            info!(l.get(), $tag, $($args)+);
        }
    };
    ($w:ident, $e:expr, $($args:tt)+) => {
        if let Some(l) = $w.ecs().logs.get($e) {
            info!(l.get(), $($args)+);
        }
    };
);

#[macro_export]
macro_rules! debug_ecs(
    ($w:ident, $e:expr, #$tag:expr, $($args:tt)+) => {
        if let Some(l) = $w.ecs().logs.get($e) {
            debug!(l.get(), $tag, $($args)+);
        }
    };
    ($w:ident, $e:expr, $($args:tt)+) => {
        if let Some(l) = $w.ecs().logs.get($e) {
            debug!(l.get(), $($args)+);
        }
    };
);

#[macro_export]
macro_rules! trace_ecs(
    ($w:ident, $e:expr, #$tag:expr, $($args:tt)+) => {
        if let Some(l) = $w.ecs().logs.get($e) {
            trace!(l.get(), $tag, $($args)+);
        }
    };
    ($w:ident, $e:expr, $($args:tt)+) => {
        if let Some(l) = $w.ecs().logs.get($e) {
            trace!(l.get(), $($args)+);
        }
    };
);
