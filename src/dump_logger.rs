use crate::ArgVerbosity;
use std::os::raw::{c_char, c_void};

static DUMP_CALLBACK: std::sync::OnceLock<Option<DumpCallback>> = std::sync::OnceLock::new();

/// # Safety
///
/// set dump log info callback.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dns2socks_set_log_callback(
    callback: Option<unsafe extern "C" fn(ArgVerbosity, *const c_char, *mut c_void)>,
    ctx: *mut c_void,
) {
    if let Some(_cb) = DUMP_CALLBACK.get_or_init(|| Some(DumpCallback(callback, ctx))) {
        log::info!("dump log callback set success");
    } else {
        log::warn!("dump log callback already set");
    }
}

#[derive(Clone, Debug)]
struct DumpCallback(Option<unsafe extern "C" fn(ArgVerbosity, *const c_char, *mut c_void)>, *mut c_void);

impl DumpCallback {
    unsafe fn call(self, dump_level: ArgVerbosity, info: *const c_char) {
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
        if let Some(Some(cb)) = DUMP_CALLBACK.get() {
            unsafe { cb.clone().call(record.level().into(), ptr) };
        }
    }
}
