use std::io;
use std::process::ExitCode;

use rs_emhdr2asn1least::stdin2hdrs2der2stdout;

fn sub() -> Result<(), io::Error> {
    let lmt: u64 = 1048576;
    let mut buf: Vec<u8> = Vec::with_capacity(lmt as usize);

    stdin2hdrs2der2stdout(&mut buf, lmt)?;
    Ok(())
}

fn main() -> ExitCode {
    sub().map(|_| ExitCode::SUCCESS).unwrap_or_else(|e| {
        eprintln!("{e}");
        ExitCode::FAILURE
    })
}
