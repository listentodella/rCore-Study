use clap::Parser;

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

// 对于结构体,必须将每一个想要对外暴露的成员使用pub修饰
#[derive(Debug, Parser)]
struct QemuOpts {
    #[arg(long)]
    gdb: Option<u16>,
}

fn main() {
    use SubCommand::*;
    match Opts::parse().cmd {
        Qemu(qemu_opts) => qemu_opts.run(),
    }
}

impl QemuOpts {
    fn run(&self) {
        println!("qemu gdb port {:?}", self.gdb);
    }
}
