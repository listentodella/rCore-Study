//! File and filesystem-related syscalls

const FD_STDOUT: usize = 1;

pub fn sys_write(fd: usize, buffer: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            //let str = unsafe { core::str::from_raw_parts(buffer, len) };
            let slice = unsafe { core::slice::from_raw_parts(buffer, len) };
            let str = core::str::from_utf8(slice).unwrap();
            println!("{}", str);
            len as isize
        }
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}
