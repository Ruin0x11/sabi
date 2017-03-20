use std::fmt;
use std::panic;
use std::thread;

use backtrace::Backtrace;

use std::panic::*;
use std::io;
use std::fs::File;

use chrono::Local;
use slog::{self, Logger, DrainExt};
use slog_stream;

const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

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

    let root_drain = slog_stream::stream(root_logfile, SabiLogFormat);
    let version = VERSION.unwrap_or("unknown");
    let logger = Logger::root(root_drain.fuse(), o!("version" => version, "system"
    => system_name.to_string()));

    info!(logger, "Log for {} initialized.", system_name);

    Ok(logger)
}

struct Shim(Backtrace);

impl fmt::Debug for Shim {
    #[cfg(feature = "with-backtrace")]
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "\n{:?}", self.0)
    }

    #[cfg(not(feature = "with-backtrace"))]
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

pub fn init_panic_hook() {
    panic::set_hook(Box::new(|info| {
        let logger = make_logger("error").unwrap();
        let backtrace = Backtrace::new();

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
                error!(logger, "thread '{}' panicked at '{}': {}:{}{:?}",
                       thread,
                       msg,
                       location.file(),
                       location.line(),
                       Shim(backtrace));
            }
            None => error!(logger, "thread '{}' panicked at '{}'{:?}", thread, msg, Shim(backtrace)),
        }
    }));
}

struct SabiLogFormat;

impl slog_stream::Format for SabiLogFormat {
    fn format(&self,
              io: &mut io::Write,
              rinfo: &slog::Record,
              _logger_values: &slog::OwnedKeyValueList) // IMPLEMENT
              -> io::Result<()> {
        let msg = format!("{} {} - {}\n", now!(), rinfo.level(), rinfo.msg());
        io.write_all(msg.as_bytes()).map(|_| ())
    }
}
