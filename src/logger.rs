use std::panic::set_hook;
use std::sync::Mutex;

use log::{self, Record, Level, Metadata};


static mut LOGGER: GlobalLogger = GlobalLogger {
    cur: None,
};

static mut INITED: bool = false;

#[cfg(target_arch="wasm32")]
unsafe impl Sync for GlobalLogger {}

struct GlobalLogger {
    cur: Option<Mutex<Logger>>,
}

struct Logger {
    buf: String,
    sublogger: String,
}

pub struct SchedulerLogger;
pub struct Sublogger(usize);

extern {
    fn log_panic(payload_ptr: *const u8, payload_len: usize,
                        file_ptr: *const u8, file_len: usize, line: u32);
}


impl SchedulerLogger {
    pub fn context() -> SchedulerLogger {
        if !unsafe { INITED } {
            init();
        }
        unsafe {
            if LOGGER.cur.is_some() {
                panic!("nested scheduler logging context")
            }
            LOGGER.cur = Some(Mutex::new(Logger {
                buf: String::with_capacity(8096),
                sublogger: String::with_capacity(16),
            }))
        }
        return SchedulerLogger;
    }
    pub fn into_inner(self) -> String {
        let buf = unsafe {
            LOGGER.cur.take().expect("scheduler is set and not nested")
        };
        drop(self);
        return buf.into_inner().expect("logger not poisoned").buf;
    }
}

impl Drop for SchedulerLogger {
    fn drop(&mut self) {
        unsafe {
            LOGGER.cur.take();
        }
    }
}

impl Sublogger {
    pub fn context(name: &str) -> Sublogger {
        unsafe {
            let mut lg = LOGGER.cur.as_mut()
                .expect("logger is set").lock().expect("logger not poisoned");
            let sub = Sublogger(lg.sublogger.len());
            if lg.sublogger.len() != 0 {
                lg.sublogger.push('.');
            }
            lg.sublogger.push_str(name);
            return sub;
        }
    }
}

impl Drop for Sublogger {
    fn drop(&mut self) {
        unsafe {
            let mut lg = LOGGER.cur.as_mut()
                .expect("logger is set").lock().expect("logger not poisoned");
            lg.sublogger.truncate(self.0);
        }
    }
}

impl log::Log for GlobalLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        use std::fmt::Write;
        if self.enabled(record.metadata()) {
            if self.cur.is_none() {
                panic!("logging when no schedule is active");
            }
            let ref mut log = *self.cur.as_ref()
                .expect("logger is set").lock().expect("logger not poisoned");
            writeln!(&mut log.buf,
                "{:>5}: {}[{}]: {}",
                    record.level(),
                    record.module_path().unwrap_or("<unknown>"),
                    &log.sublogger,
                    record.args())
                    .expect("can write to buffer");
        }
    }

    fn flush(&self) { }
}

pub fn init() {
    unsafe {
        INITED = true;
        log::set_logger(&LOGGER).expect("log init ok");
    }
    log::set_max_level(log::LevelFilter::Debug);
    set_hook(Box::new(|panic_info| {
        let payload = panic_info.payload();
        let (ptr, len) = if let Some(s) = payload.downcast_ref::<&str>() {
            (s.as_bytes().as_ptr(), s.len())
        } else if let Some(s) = payload.downcast_ref::<String>() {
            (s.as_bytes().as_ptr(), s.len())
        } else {
            (0 as *const u8, 0)
        };
        let (file_ptr, file_len, line) = match panic_info.location() {
            Some(loc) => {
                let file = loc.file().as_bytes();
                (file.as_ptr(), file.len(), loc.line())
            }
            None => (0 as *const u8, 0, 0),
        };
        unsafe {
            log_panic(ptr, len, file_ptr, file_len, line);
        }
    }));
}
