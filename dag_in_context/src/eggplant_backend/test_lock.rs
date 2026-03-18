#[cfg(all(test, feature = "eggplant"))]
use std::sync::{Mutex, MutexGuard, OnceLock};

#[cfg(all(test, feature = "eggplant"))]
fn native_test_mutex() -> &'static Mutex<()> {
    static NATIVE_TEST_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();
    NATIVE_TEST_MUTEX.get_or_init(|| Mutex::new(()))
}

#[cfg(all(test, feature = "eggplant"))]
pub(crate) fn lock() -> MutexGuard<'static, ()> {
    native_test_mutex()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
