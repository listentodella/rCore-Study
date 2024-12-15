# RUST study for RISCV  
## env  
`myos` 目录下是基础的`riscv` + `rust`环境, QEMU版本需要使用`4.2.1`(主要是因为高版本的QEMU需要SBI实现得要完整,否则无法正常启动)。  
对于`MacOS M`系列,即便手动编译`QEMU4.2.1`,也会无法正常启动,因此提供了Docker,供灵活使用。  

## 使用方法  
`myos` 下提供了`xtask`, 本质上`xtask`是一个独立的app, `myos/.cargo/config.toml`里提供了对应的快捷指令:  
### make  
```bash
# 默认编译 os
$ cargo make
# 选择编译
$ cargo make --bin os
$ cargo make --bin mysbi

```

### asm  
如果去掉`verbose` 选项, 则只会打印`section`信息 
```bash
# 默认编译并反汇编查看 os 的信息
$ cargo asm --verbose
# 选择特定目标
$ cargo asm --bin os --verbose
$ cargo asm --bin mysbi --verbose

```

### qemu  
```bash
# 通过qemu加载mysbi与os
$ cargo qemu

```
