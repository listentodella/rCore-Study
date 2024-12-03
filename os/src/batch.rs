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

impl AppManager {
    pub fn print_app_info(&self) {
        info!("[kernel] there are {} apps found", self.num_app);
        for i in 0..self.num_app {
            debug!(
                "[kernel] app[{i}] start_addr[{:#x}, {:#x})",
                self.app_start[i],
                self.app_start[i + 1]
            );
        }
    }

    pub fn get_current_app(&self) -> usize {
        self.curr_app
    }

    pub fn move_to_next_app(&mut self) {
        self.curr_app += 1;
    }
}

pub fn print_app_info() {
    APP_MANAGER.exclusive_access().print_app_info();
}

pub fn init() {
    print_app_info();
}
