use bonsai_contracts::FILE_DESCRIPTOR_SET;
use std::io::{self, Write};

fn main() -> io::Result<()> {
    io::stdout().write_all(FILE_DESCRIPTOR_SET)
}
