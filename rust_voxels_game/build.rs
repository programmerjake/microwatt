use std::{env, io, path::Path, process::Command};

const CFLAGS: &[&str] = &[
    "-Os",
    "-g",
    "-Wall",
    "-std=c99",
    "-msoft-float",
    "-mno-string",
    "-mno-multiple",
    "-mno-vsx",
    "-mno-altivec",
    "-mlittle-endian",
    "-fno-stack-protector",
    "-mstrict-align",
    "-ffreestanding",
    "-fdata-sections",
    "-ffunction-sections",
    "-I../include",
];

fn prefix() -> &'static str {
    if env::var("HOST").unwrap() != "powerpc64le-linux-gnu" {
        "powerpc64le-linux-gnu-"
    } else {
        ""
    }
}

fn uart_bauds() -> u32 {
    let s = env::var_os("UART_BAUDS").unwrap_or_else(|| "115200".into());
    s.to_str().unwrap().parse().unwrap()
}

fn gcc(source: impl AsRef<Path>) -> io::Result<()> {
    let source = source.as_ref();
    println!("cargo:rerun-if-changed={}", source.display());
    let target = source.with_extension("o");
    let target = Path::new(target.file_name().unwrap());
    println!("cargo:rustc-link-arg={}", target.display());
    if !Command::new(format!("{}gcc", prefix()))
        .args(CFLAGS)
        .arg(format!("-DUART_BAUDS={}", uart_bauds()))
        .arg("-c")
        .arg("-o")
        .arg(&target)
        .arg(source)
        .status()?
        .success()
    {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("failed to compile: {}", source.display()),
        ))
    } else {
        Ok(())
    }
}

fn embedded() -> io::Result<()> {
    gcc("head.S")?;
    gcc("../lib/console.c")?;
    println!("cargo:rustc-link-arg=-T");
    println!("cargo:rustc-link-arg=powerpc.lds");
    println!("cargo:rerun-if-changed=powerpc.lds");
    println!("cargo:rustc-link-arg=-nostartfiles");
    println!("cargo:rustc-link-arg=-static");
    Ok(())
}

#[cfg(feature = "embedded")]
fn main() -> io::Result<()> {
    embedded()
}

#[cfg(feature = "hosted")]
fn main() {
    let _ = embedded;
}
