use crate::sync::UPSafeCell;
use core::arch::asm;
use lazy_static::*;
use log::*;

const MAX_APP_NUM: usize = 16;
// 这是kernel与app约定好的地址
const APP_BASE_ADDRESS: usize = 0x80400000;
const APP_SIZE_LIMIT: usize = 0x20000;

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
    fn print_app_info(&self) {
        info!("[kernel] there are {} apps found", self.num_app);
        for i in 0..self.num_app {
            debug!(
                "[kernel] app[{i}] start_addr[{:#x}, {:#x})",
                self.app_start[i],
                self.app_start[i + 1]
            );
        }
    }

    fn get_current_app(&self) -> usize {
        self.curr_app
    }

    fn move_to_next_app(&mut self) {
        self.curr_app += 1;
    }

    unsafe fn load_app(&self, app_id: usize) {
        if app_id >= self.num_app {
            error!("All apps completed!");
        }
        info!("[kernel] Loading app_{}", app_id);

        //clear app area
        core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, APP_SIZE_LIMIT).fill(0);

        //load from app
        let app_src = core::slice::from_raw_parts(
            self.app_start[app_id] as *const u8,
            self.app_start[app_id + 1] - self.app_start[app_id],
        );

        // store to app area
        let app_dst = core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, app_src.len());
        app_dst.copy_from_slice(app_src);

        // Memory fence about fetching the instruction memory
        // See also: riscv non-priv spec chapter 3, 'Zifencei' extension.
        // 通常情况下， CPU 会认为程序的代码段不会发生变化，因此 i-cache 是一种只读缓存。
        // 但在这里，OS 将修改会被 CPU 取指的内存区域，这会使得 i-cache 中含有与内存中不一致的内容。
        // 因此， OS 在这里必须使用取指屏障指令 fence.i ，
        // 它的功能是保证 在它之后的取指过程必须能够看到在它之前的所有对于取指内存区域的修改 ，
        // 这样才能保证 CPU 访问的应用代码是最新的而不是 i-cache 中过时的内容。
        // 不过,在QEMU环境中, 即便不加该指令,也不一定出错
        asm!("fence.i");
    }
}

pub fn print_app_info() {
    APP_MANAGER.exclusive_access().print_app_info();
}

pub fn init() {
    print_app_info();
}

pub fn run_next_app() {
    let mut app_manager = APP_MANAGER.exclusive_access();
    let curr_app_id = app_manager.get_current_app();
    unsafe {
        app_manager.load_app(curr_app_id);
    }
    app_manager.move_to_next_app();
}
