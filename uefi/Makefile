build:
	cargo +nightly build --release

run: build
	mkdir -p mnt/EFI/BOOT
	cp Shell.efi mnt/EFI/BOOT/BOOTX64.EFI
	cp ./target/x86_64-unknown-uefi/release/uefi.efi mnt/
	qemu-system-x86_64 -bios OVMF.fd -drive format=raw,file=fat:rw:mnt -serial mon:stdio

test: build
	qemu-system-x86_64 -bios OVMF.fd