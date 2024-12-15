use clap::Parser;
use log::{debug, error, info, warn};
use os_xtask_utils::{BinUtil, Cargo, CommandExt, Qemu};
use std::path::{Path, PathBuf};

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
    #[command(name = "asm", about = "asm args about")]
    Asm(AsmOpts),
    #[command(name = "qemu", about = "qemu args about")]
    Qemu(QemuOpts),
    #[command(name = "make", about = "build args about")]
    Make(BuildOpts),
}

#[derive(Debug, Parser)]
struct AsmOpts {
    #[clap(flatten)]
    make: BuildOpts,
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

#[derive(Debug, Parser)]
struct QemuOpts {
    #[clap(flatten)]
    make: BuildOpts,
    #[arg(long, default_value = "riscv64")]
    arch: String,
    #[arg(long, default_value = "1234")]
    gdb: Option<u16>,
    #[arg(short, long, default_value_t = false)]
    run: bool,
}

#[derive(Debug, Parser)]
struct BuildOpts {
    /// Chapter number
    #[arg(short, long, default_value = "os")]
    bin: String,
    /// Log level
    #[arg(long, default_value = "trace")]
    log: Option<String>,
    /// Builds in release mode
    #[arg(long, default_value_t = false)]
    release: bool,
    #[arg(short, long, default_value = "riscv64gc-unknown-none-elf")]
    target: String,
}

fn main() {
    use SubCommand::*;
    pretty_env_logger::init();
    match Opts::parse().cmd {
        Qemu(qemu_opts) => qemu_opts.run(),
        Make(build_opts) => {
            let _ = build_opts.run();
        }
        Asm(asm_opts) => asm_opts.run(),
    }
}

impl QemuOpts {
    //qemu-system-riscv64 -nographic -machine virt -bios mysbi.bin
    //-device loader,file=os.bin,addr=0x80200000
    fn run(&self) {
        info!("qemu opt args {:?}", self);
        // let mysbi = BuildOpts {
        //     bin: "mysbi".to_string(),
        //     log: None,
        //     release: false,
        //     target: "riscv64gc-unknown-none-elf".to_string(),
        // };
        // mysbi.run();

        // let os = BuildOpts {
        //     bin: "os".to_string(),
        //     log: None,
        //     release: false,
        //     target: "riscv64gc-unknown-none-elf".to_string(),
        // };
        // os.run();

        let mut path = PathBuf::new();
        path.push("target");
        path.push(self.make.target.as_str());
        path.push(if self.make.release {
            "release"
        } else {
            "debug"
        });
        let mut sbi = path.clone();
        sbi.push("mysbi.bin");
        let mut os = path.clone();
        os.push("os.bin");

        let mut binding = Qemu::system("riscv64");
        let qemu = binding
            .args(["-machine", "virt"])
            .args(["-m", "128M"])
            .arg("-nographic")
            .arg("-bios")
            .arg(format!("{}", sbi.display()).as_str())
            .arg("-device")
            .arg(format!("loader,file={},addr=0x80200000", os.display()).as_str())
            .optional(&self.gdb, |qemu, gdb| {
                if !self.run {
                    qemu.args(["-S", "-gdb", format!("tcp::{}", gdb).as_str()]);
                }
            });
        debug!("QEMU CMD: {:?}", qemu.info());
        qemu.invoke();
    }
}

impl AsmOpts {
    fn run(&self) {
        let elf = self.make.run();
        let mut binding = BinUtil::objdump();
        let bin_cmd = binding
            .arg(elf)
            .arg(if self.verbose { "-d" } else { "-h" })
            .output()
            .stdout;
        let output = String::from_utf8(bin_cmd).unwrap();
        println!("{}", output);
    }
}

impl BuildOpts {
    fn run(&self) -> PathBuf {
        info!("build opt args {:?}", self);
        let mut path = PathBuf::new();
        let mut binding = Cargo::new("-C");
        let cargo_cmd = binding
            .arg(self.bin.as_str())
            .args(["-Z", "unstable-options"])
            .arg("build")
            .package(self.bin.as_str())
            .target(self.target.as_str());
        info!("{:?}", cargo_cmd.info());
        cargo_cmd.invoke();
        path.push("target");
        path.push(self.target.as_str());
        path.push(if self.release { "release" } else { "debug" });
        path.push(self.bin.as_str());
        info!("build success for {:?}", path);
        objcopy(&path, true);
        path
    }
}

fn objcopy(elf: impl AsRef<Path>, is_binary: bool) -> PathBuf {
    let elf = elf.as_ref();
    let bin = elf.with_extension("bin");
    let mut binding = BinUtil::objcopy();
    let bin_cmd = binding
        .arg(elf)
        .arg("--strip-all")
        .conditional(is_binary, |binutil| {
            binutil.args(["-O", "binary"]);
        })
        .arg(&bin);
    info!("objcopy CMD: {:?}", bin_cmd.info());
    bin_cmd.invoke();

    bin
}
