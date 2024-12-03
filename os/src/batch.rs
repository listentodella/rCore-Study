use crate::sync::UPSafeCell;
use lazy_static::*;
use log::*;

const MAX_APP_NUM: usize = 16;

struct AppManager {
    num_app: usize,
    curr_app: usize,
    app_start: [usize; MAX_APP_NUM + 1],
}

lazy_static! {
    static ref APP_MANAGER: UPSafeCell<AppManager> = unsafe {
        UPSafeCell::new({
            extern "C" {
                // from link_app.S
                // each element is an app start addr
                fn _num_app();
            }
            let num_app_ptr = _num_app as *mut usize;
            let num_app = num_app_ptr.read_volatile();

            // define an array to store apps start addrs
            let mut app_start: [usize; MAX_APP_NUM + 1] = [0; MAX_APP_NUM + 1];
            // the elem 0 is the num_app instead of any app addr
            // so we should skip it
            // the last elem is ?
            let app_start_raw: &[usize] =
                core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1);
            // fill the array
            app_start[..=num_app].copy_from_slice(app_start_raw);

            AppManager { num_app, curr_app: 0, app_start}
        })
    };
}
