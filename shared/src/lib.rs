use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use typed_builder::TypedBuilder;

pub static SOCKET: &str = "/tmp/uconix";

#[derive(Debug, Serialize, Deserialize, TypedBuilder)]
pub struct Program {
    pub env: HashMap<String, String>,
    pub args: Vec<String>,
    pub cwd: PathBuf,
}

pub fn this_program() -> eyre::Result<Program> {
    let p = Program::builder()
        .env(std::env::vars().collect())
        .args(std::env::args().map(Into::into).collect())
        .cwd(std::env::current_dir()?)  
        .build();

    Ok(p)
}