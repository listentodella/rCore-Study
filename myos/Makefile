DOCKER_TAG ?= rcore-study:latest
.PHONY: docker build_docker
	
docker:
	docker run --rm -it -p 3333:1234 -v ${PWD}:/mnt -w /mnt --name rcore-study ${DOCKER_TAG} bash

build_docker: 
	docker build -t ${DOCKER_TAG} --target build .

fmt:
	cd os ; cargo fmt;  cd ..

gdb:
	qemu-system-riscv64 -machine virt -m 128M -nographic -bios target/riscv64gc-unknown-none-elf/debug/mysbi.bin -device loader,file=target/riscv64gc-unknown-none-elf/debug/os.bin,addr=0x80200000 -S -gdb tcp::1234

run:
	qemu-system-riscv64 -machine virt -m 128M -nographic -bios target/riscv64gc-unknown-none-elf/debug/mysbi.bin -device loader,file=target/riscv64gc-unknown-none-elf/debug/os.bin,addr=0x80200000
