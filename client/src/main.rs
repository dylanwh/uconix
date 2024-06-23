use std::{
    io::{Read, Write},
    os::fd::AsRawFd,
    process::ExitCode,
};

use passfd::FdPassingExt;
use shared::{this_program, Program, SOCKET};

fn main() -> eyre::Result<ExitCode> {
    let p = this_program()?;
    let mut stream = std::os::unix::net::UnixStream::connect(SOCKET)?;
    stream.write_all(b"P")?;
    let p = rmp_serde::to_vec(&p)?;
    let len = p.len().to_ne_bytes();
    stream.write_all(&len)?;
    stream.write_all(&p)?;
    let stdin = std::io::stdin().as_raw_fd();
    let stdout = std::io::stdout().as_raw_fd();
    let stderr = std::io::stderr().as_raw_fd();
    stream.send_fd(stdin)?;
    stream.send_fd(stdout)?;
    stream.send_fd(stderr)?;

    let mut exit_code = [0; 1];
    stream.read_exact(&mut exit_code)?;
    let exit_code = ExitCode::from(u8::from_ne_bytes(exit_code));

    Ok(exit_code)
}
