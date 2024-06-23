use std::os::fd::FromRawFd;

use listener::Stdio;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::AsyncReadExt;
use tokio::net::UnixStream;

mod listener;

#[tokio::main]
async fn main() -> eyre::Result<()> {

    let mut recv = listener::run()?;
    while let Some((stdio, program, stream)) = recv.recv().await {
        tokio::spawn(async move {
            let Stdio { stdin, stdout, stderr } = stdio;
            // wrap in tokio file
            let stdin = unsafe { tokio::fs::File::from_raw_fd(stdin)};
            let mut stdout = unsafe { tokio::fs::File::from_raw_fd(stdout)};
            let mut stderr = unsafe { tokio::fs::File::from_raw_fd(stderr)};
            stdout.write_all(b"Hello, world!\n").await.unwrap();
            let mut stdin = tokio::io::BufReader::new(stdin);
            let mut line = String::new();
            stdin.read_line(&mut line).await.unwrap();
            stderr.write_all(b"No errors, yet.\n").await.unwrap();
            println!("Program: {:?}", program);
            println!("Received: {}", line);
            let mut stream = UnixStream::from_std(stream).unwrap();
            // now write a 16bit exit code (0)
            stream.write_all(&[0, 0]).await.unwrap();
        });
    }

    Ok(())
}
