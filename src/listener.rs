use std::{io::Read, os::unix::net::{UnixListener, UnixStream}, thread};

use passfd::FdPassingExt;
use shared::{Program, SOCKET};

pub struct Stdio {
    pub stdin: i32,
    pub stdout: i32,
    pub stderr: i32,
}

type Run = (Stdio, Program, UnixStream);

type Sender = tokio::sync::mpsc::Sender<Run>;
type Receiver = tokio::sync::mpsc::Receiver<Run>;


pub fn run() -> eyre::Result<Receiver> {

    let (sender, receiver) = tokio::sync::mpsc::channel(32);
    thread::spawn(|| {
        listener(SOCKET, sender).unwrap()
    });

    Ok(receiver)
}

fn listener(socket: &str, sender: Sender) -> eyre::Result<()> {
    if std::fs::metadata(socket).is_ok() {
        std::fs::remove_file(socket)?;
    }
    let listener = UnixListener::bind(socket)?;
    println!("Listening on {}", socket);

    for stream in listener.incoming() {
        let mut stream = stream?;
        println!("Accepted connection from {:?}", stream.peer_addr()?);
        // read first byte to determine what to do
        let mut buf = [0; 1];
        stream.read_exact(&mut buf)?;
        match buf[0] as char {
            'P' => handle_program(&sender, stream)?,
            _ => {
                eprintln!("Unknown message type: {:?}", buf[0]);
            }
        }
    }

    Ok(())
}

fn handle_program(
    sender: &Sender,
    mut stream: std::os::unix::net::UnixStream,
) -> eyre::Result<()> {
    let mut program_len: [u8; 8] = [0; 8];
    stream.read_exact(&mut program_len)?;
    let program_len = usize::from_ne_bytes(program_len);
    let mut program = vec![0; program_len];
    stream.read_exact(&mut program)?;
    let program = rmp_serde::from_slice::<Program>(&program)?;
    let stdin = stream.recv_fd()?;
    let stdout = stream.recv_fd()?;
    let stderr = stream.recv_fd()?;
    let stdio = Stdio {
        stdin,
        stdout,
        stderr,
    };

    // put stream into non-blocking mode
    stream.set_nonblocking(true)?;
    sender.blocking_send((stdio, program, stream))?;

    Ok(())
}
