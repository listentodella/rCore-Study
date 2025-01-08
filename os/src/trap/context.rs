use riscv::register::sstatus::{self, Sstatus, SPP};

//Trap上下文（即数据结构 TrapContext ），
//类似函数调用上下文，即在 Trap 发生时需要保存的物理资源内容
#[repr(C)]
pub struct TrapContext {
    // 通用寄存器 x0 ~ x31
    // 不过x0其实硬编码为0,不可能有变化
    pub x: [usize; 32],

    // scause/stval ：它总是在 Trap 处理的第一时间就被使用或者是在其他地方保存下来了
    // 因此它没有被修改并造成不良影响的风险。
    // 而对于 sstatus/sepc 而言，它们会在 Trap 处理的全程有意义（在 Trap 控制流最后 sret 的时候还用到了它们），而且确实会出现 Trap 嵌套的情况使得它们的值被覆盖掉。
    // 所以我们需要将它们也一起保存下来，并在 sret 之前恢复原样
    // sstatus reg
    pub sstatus: Sstatus,
    // sepc reg
    pub sepc: usize,
}

impl TrapContext {
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }
    pub fn app_init_context(entry: usize, sp: usize) -> Self {
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User);
        let mut ctx = Self {
            x: [0; 32],
            sstatus,
            sepc: entry,
        };
        ctx.set_sp(sp);
        ctx
    }
}
