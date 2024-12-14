use clap::Parser;
use log::{debug, error, info, warn};
use os_xtask_utils::{CommandExt, Qemu};

//对该复合类型使用clap::Parser派生宏
#[derive(Debug, Parser)]
//使用command宏,解释该命令,如果不主动赋值,会从项目的Cargo.toml里取值
#[command(name = "mycli", version, author, about, long_about = None)]
struct Opts {
    //使用subcommand标记该成员为子命令
    #[command(subcommand)]
    cmd: SubCommand,
}

#[derive(Debug, Parser)]
enum SubCommand {
    //对子命令成员解释,如果不显式起名,则默认为该成员名的小写
    #[command(name = "qemu", about = "qemu args about")]
    Qemu(QemuOpts),
}

#[derive(Debug, Parser)]
struct QemuOpts {
    #[arg(long, default_value = "riscv64")]
    arch: String,
    #[arg(long, default_value = "1234")]
    gdb: Option<u16>,
}

fn main() {
    use SubCommand::*;
    pretty_env_logger::init();
    match Opts::parse().cmd {
        Qemu(qemu_opts) => qemu_opts.run(),
    }
}

impl QemuOpts {
    //qemu-system-riscv64 -nographic -machine virt -bios mysbi.bin
    //-device loader,file=os.bin,addr=0x80200000
    fn run(&self) {
        info!("qemu opt args {:?}", self);
        let mut binding = Qemu::system("riscv64");
        let qemu = binding
            .args(["-machine", "virt"])
            .arg("-nographic")
            .arg("-bios")
            .arg("mysbi.bin")
            .args(["-device", "loader,file=os.bin,addr=0x80200000"]);
        info!("QEMU CMD: {:?}", qemu.info());
        qemu.invoke();
    }
}
