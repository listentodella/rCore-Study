pub fn backup_and_restore_sp() {
    extern "C" {
        fn branch_test();
    }
    unsafe {
        branch_test();
    }
}
