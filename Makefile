arch ?= x86
kernel := build/kernel.bin
iso := build/os.iso
target ?= zero_os
rust_os := target/$(target)/debug/libzero_os.a	

linker_script := boot/linker.ld
grub_cfg := boot/grub.cfg
assembly_source_files := $(wildcard boot/*.asm)
assembly_object_files := $(patsubst boot/%.asm, \
	build/arch/$(arch)/%.o, $(assembly_source_files))

.PHONY: all clean run iso kernel

all: $(kernel)

clean:
	@rm -r build

run: $(iso)
	@qemu-system-x86_64 -serial stdio -cdrom $(iso)

iso: $(iso)

$(iso): $(kernel) $(grub_cfg)
	@mkdir -p build/isofiles/boot/grub
	@cp $(kernel) build/isofiles/boot/kernel.bin
	@cp $(grub_cfg) build/isofiles/boot/grub
	@grub-mkrescue -o $(iso) build/isofiles 2> /dev/null
	@rm -r build/isofiles

$(kernel): kernel $(rust_os) $(assembly_object_files) $(linker_script)
	@ld -n -T $(linker_script) -o $(kernel) \
		$(assembly_object_files) $(rust_os)

kernel:
	cargo build

# compile assembly files
build/arch/$(arch)/%.o: boot/%.asm
	@mkdir -p $(shell dirname $@)
	@nasm -felf64 $< -o $@