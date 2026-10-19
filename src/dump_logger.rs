use std::os::raw::{c_char, c_void};

static DUMP_CALLBACK: std::sync::Mutex<Option<DumpCallback>> = std::sync::Mutex::new(None);

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum LogLevel {
    Off = 0,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<log::LevelFilter> for LogLevel {
    fn from(level: log::LevelFilter) -> Self {
        match level {
            log::LevelFilter::Off => LogLevel::Off,
            log::LevelFilter::Error => LogLevel::Error,
            log::LevelFilter::Warn => LogLevel::Warn,
            log::LevelFilter::Info => LogLevel::Info,
            log::LevelFilter::Debug => LogLevel::Debug,
            log::LevelFilter::Trace => LogLevel::Trace,
        }
    }
}

impl From<LogLevel> for log::LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Off => log::LevelFilter::Off,
            LogLevel::Error => log::LevelFilter::Error,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Trace => log::LevelFilter::Trace,
        }
    }
}

/// # Safety
///
/// set dump log info callback.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dns2socks_set_log_callback(
    callback: Option<unsafe extern "C" fn(LogLevel, *const c_char, *mut c_void)>,
    ctx: *mut c_void,
) {
    if let Ok(mut lock) = DUMP_CALLBACK.lock() {
        *lock = Some(DumpCallback(callback, ctx));
        log::info!("dump log callback set success");
    } else {
        log::warn!("dump log callback set failed");
    }
}

#[derive(Clone, Debug)]
struct DumpCallback(Option<unsafe extern "C" fn(LogLevel, *const c_char, *mut c_void)>, *mut c_void);

impl DumpCallback {
    unsafe fn call(self, dump_level: LogLevel, info: *const c_char) {
        if let Some(cb) = self.0 {
            unsafe { cb(dump_level, info, self.1) };
        }
    }
}

unsafe impl Send for DumpCallback {}
unsafe impl Sync for DumpCallback {}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct DumpLogger;

impl log::Log for DumpLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Trace
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let current_crate_name = env!("CARGO_CRATE_NAME");
            if record.module_path().unwrap_or("").starts_with(current_crate_name) {
                self.do_dump_log(record);
            }
        }
    }

    fn flush(&self) {}
}

impl DumpLogger {
    fn do_dump_log(&self, record: &log::Record) {
        let timestamp: chrono::DateTime<chrono::Local> = chrono::Local::now();
        let msg = format!(
            "[{} {:<5} {}] - {}",
            timestamp.format("%Y-%m-%d %H:%M:%S"),
            record.level(),
            record.module_path().unwrap_or(""),
            record.args()
        );
        let Ok(c_msg) = std::ffi::CString::new(msg) else {
            return;
        };
        let ptr = c_msg.as_ptr();
        let callback = DUMP_CALLBACK.lock().ok().and_then(|lock| lock.clone());
        if let Some(cb) = callback {
            unsafe { cb.call(record.level().to_level_filter().into(), ptr) };
        }
    }
}
