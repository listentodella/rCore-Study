use crate::sbi::shutdown;
use crate::sync::UPSafeCell;
use crate::trap::TrapContext;
use core::arch::asm;
use lazy_static::*;
use log::*;

const MAX_APP_NUM: usize = 16;
// 这是kernel与app约定好的地址
const APP_BASE_ADDRESS: usize = 0x80400000;
const APP_SIZE_LIMIT: usize = 0x20000;

// 8KB stack size
const USER_STACK_SIZE: usize = 4096 * 2;
const KERNEL_STACK_SIZE: usize = 4096 * 2;

#[repr(align(4096))]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

#[repr(align(4096))]
struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

static KERNEL_STACK: KernelStack = KernelStack {
    data: [0; KERNEL_STACK_SIZE],
};
static USER_STACK: UserStack = UserStack {
    data: [0; USER_STACK_SIZE],
};

impl KernelStack {
    // 由于在 RISC-V 中栈是向下增长
    // 我们只需返回包裹的数组的结尾地址, 即是栈顶
    // 于是换栈, 只需将 sp 寄存器的值修改为 get_sp 的返回值即可
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }

    pub fn push_context(&self, cx: TrapContext) -> &'static mut TrapContext {
        let cx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe {
            *cx_ptr = cx;
        }
        unsafe { cx_ptr.as_mut().unwrap() }
    }
}

impl UserStack {
    // 由于在 RISC-V 中栈是向下增长
    // 我们只需返回包裹的数组的结尾地址, 即是栈顶
    // 于是换栈, 只需将 sp 寄存器的值修改为 get_sp 的返回值即可
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

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
            info!("All apps completed!");
            shutdown(false);
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

pub fn run_next_app() -> ! {
    let mut app_manager = APP_MANAGER.exclusive_access();
    let curr_app_id = app_manager.get_current_app();
    unsafe {
        app_manager.load_app(curr_app_id);
    }
    app_manager.move_to_next_app();
    // we need manually drop to release, or use {...} to release
    drop(app_manager);

    // before this we have to drop local variables related to resources manually
    // and release the resources
    extern "C" {
        fn __restore(cx_addr: usize);
    }
    unsafe {
        __restore(KERNEL_STACK.push_context(TrapContext::app_init_context(
            APP_BASE_ADDRESS,
            USER_STACK.get_sp(),
        )) as *const _ as usize);
    }

    panic!("Unreachable in batch::run_current_app!");
}
