.PHONY: all run size dump emu clean

ARCH = $(shell uname -m)
ifneq ("$(ARCH)", "ppc64")
ifneq ("$(ARCH)", "ppc64le")
	CROSS_COMPILE ?= powerpc64le-linux-gnu-
endif
endif

CC = $(CROSS_COMPILE)gcc
LD = $(CROSS_COMPILE)ld
OBJCOPY = $(CROSS_COMPILE)objcopy

CFLAGS = -Os -g -Wall -std=c99 -msoft-float -mno-string -mno-multiple -mno-vsx -mno-altivec -mlittle-endian -fno-stack-protector -mstrict-align -ffreestanding -fdata-sections -ffunction-sections -I../include
ASFLAGS = $(CFLAGS)
LDFLAGS = -T powerpc.lds --gc-sections

RUST_BIN = $(abspath target/powerpc64le-unknown-linux-gnu/release/rust_voxels_game)
RUST_BIN_DEP = $(abspath target/powerpc64le-unknown-linux-gnu/release/rust_voxels_game.d)

all: rust_voxels_game.hex rust_voxels_game.bin

run: rust_voxels_game.bin
	-ln -sf rust_voxels_game.bin main_ram.bin
	../core_tb > /dev/null

$(RUST_BIN) $(RUST_BIN_DEP): Cargo.toml Cargo.lock Xargo.toml
	RUSTFLAGS="-C target-feature=-vsx,-altivec,-hard-float" UART_BAUDS=1000000 xargo build --bin rust_voxels_game --release --target=powerpc64le-unknown-linux-gnu --features=embedded --no-default-features && touch -c "$(RUST_BIN)"

include $(RUST_BIN_DEP)

size: rust_voxels_game.elf
	size rust_voxels_game.elf

dump: rust_voxels_game.elf
	powerpc64le-linux-gnu-objdump -S rust_voxels_game.elf | less

rust_voxels_game.elf: $(RUST_BIN)
	cp "$(RUST_BIN)" rust_voxels_game.elf

rust_voxels_game.bin: rust_voxels_game.elf
	$(OBJCOPY) -O binary $^ $@

rust_voxels_game.hex: rust_voxels_game.bin
	../scripts/bin2hex.py $^ > $@

emu:
	cargo run

clean:
	cargo clean
	@rm -f *.o rust_voxels_game.elf rust_voxels_game.bin rust_voxels_game.hex
