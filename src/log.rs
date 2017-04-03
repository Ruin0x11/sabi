use std::fmt::{self, Display};
use std::panic;
use std::thread;

use std::io;
use std::fs::File;

use chrono::Local;
use slog::{self, Logger, DrainExt};
use slog::ser::Result as SerResult;
use slog_stream;

const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

const DISABLE: bool = true;

macro_rules! now {
    () => ( Local::now().format("%m-%d %H:%M:%S%.3f") )
}

/// Create a new root log and drain for a game system.
/// Intended to be used once per major game system. For more specific logs
/// within systems, use log.new(o!(...)) instead.
pub fn make_logger(system_name: &str) -> Result<Logger, ()> {
    let path = format!("/tmp/sabi-{}.log", system_name);
    let root_logfile = File::create(path)
        .expect("Couldn't open log file");

    let logger = if DISABLE {
        Logger::root(slog::Discard, o!())
    } else {
        let version = VERSION.unwrap_or("unknown");
        let drain = slog_stream::stream(root_logfile, SabiLogFormat).fuse();
        Logger::root(drain, o!("system" => system_name.to_string()))
    };

    info!(logger, "Log for {} initialized.", system_name);

    Ok(logger)
}

pub fn init_panic_hook() {
    panic::set_hook(Box::new(|info| {
        let logger = make_logger("error").unwrap();

        let thread = thread::current();
        let thread = thread.name().unwrap_or("unnamed");

        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => &**s,
                None => "Box<Any>",
            }
        };

        match info.location() {
            Some(location) => {
                error!(logger, "thread '{}' panicked at '{}': {}:{}",
                       thread,
                       msg,
                       location.file(),
                       location.line());
            }
            None => error!(logger, "thread '{}' panicked at '{}'", thread, msg),
        }
    }));
}

struct SabiLogFormat;

impl slog_stream::Format for SabiLogFormat {
    fn format(&self,
              io: &mut io::Write,
              rinfo: &slog::Record,
              logger_values: &slog::OwnedKeyValueList)
              -> io::Result<()> {
        let mut serializer = Serializer::new();
        for (k, v) in logger_values.iter() {
            try!(v.serialize(rinfo, k, &mut serializer));
        }
        let pairs = serializer.fields.join(", ");
        let msg = format!("{} {} - {} ({})\n",
                          now!(), rinfo.level(), rinfo.msg(), pairs);
        io.write_all(msg.as_bytes())?;

        Ok(())
    }
}

struct Serializer {
    fields: Vec<String>,
}

impl Serializer {
    fn new() -> Serializer {
        Serializer { fields: Vec::new() }
    }
    /// Add field without sanitizing the key
    ///
    /// Note: if the key isn't a valid journald key name, it will be ignored.
    fn add_field(&mut self, field: String) {
        self.fields.push(field);
    }
    fn emit<T: Display>(&mut self, key: &str, val: T) -> SerResult {
        self.add_field(format!("{}={}", key, val));
        Ok(())
    }
}

impl slog::Serializer for Serializer {
    fn emit_bool(&mut self, key: &str, val: bool) -> SerResult {
        self.emit(key, val)
    }
    fn emit_unit(&mut self, key: &str) -> SerResult {
        self.emit(key, "")
    }
    fn emit_none(&mut self, key: &str) -> SerResult {
        self.emit(key, "None")
    }
    fn emit_char(&mut self, key: &str, val: char) -> SerResult {
        self.emit(key, val)
    }
    fn emit_u8(&mut self, key: &str, val: u8) -> SerResult {
        self.emit(key, val)
    }
    fn emit_i8(&mut self, key: &str, val: i8) -> SerResult {
        self.emit(key, val)
    }
    fn emit_u16(&mut self, key: &str, val: u16) -> SerResult {
        self.emit(key, val)
    }
    fn emit_i16(&mut self, key: &str, val: i16) -> SerResult {
        self.emit(key, val)
    }
    fn emit_u32(&mut self, key: &str, val: u32) -> SerResult {
        self.emit(key, val)
    }
    fn emit_i32(&mut self, key: &str, val: i32) -> SerResult {
        self.emit(key, val)
    }
    fn emit_u64(&mut self, key: &str, val: u64) -> SerResult {
        self.emit(key, val)
    }
    fn emit_i64(&mut self, key: &str, val: i64) -> SerResult {
        self.emit(key, val)
    }
    fn emit_f32(&mut self, key: &str, val: f32) -> SerResult {
        self.emit(key, val)
    }
    fn emit_f64(&mut self, key: &str, val: f64) -> SerResult {
        self.emit(key, val)
    }
    fn emit_usize(&mut self, key: &str, val: usize) -> SerResult {
        self.emit(key, val)
    }
    fn emit_isize(&mut self, key: &str, val: isize) -> SerResult {
        self.emit(key, val)
    }
    fn emit_str(&mut self, key: &str, val: &str) -> SerResult {
        self.emit(key, val)
    }
    fn emit_arguments(&mut self, key: &str, val: &fmt::Arguments) -> SerResult {
        self.emit(key, val)
    }
}
