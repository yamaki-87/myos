use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let kernel = PathBuf::from(
        env::var_os("CARGO_BIN_FILE_KERNEL_kernel").expect("CARGO_BIN_FILE_KERNEL_kernel not set"),
    );

    let bios_path = out_dir.join("os-bios.img");
    let uefi_path = out_dir.join("os-uefi.img");

    bootloader::BiosBoot::new(&kernel)
        .create_disk_image(&bios_path)
        .expect("failed to create BIOS image");

    bootloader::UefiBoot::new(&kernel)
        .create_disk_image(&uefi_path)
        .expect("failed to create UEFI image");

    println!("cargo:warning=BIOS image: {}", bios_path.display());
    println!("cargo:warning=UEFI image: {}", uefi_path.display());
}
