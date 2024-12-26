pub fn backup_ra() {
    extern "C" {
        fn branch_test();
    }
    unsafe {
        branch_test();
    }
}
