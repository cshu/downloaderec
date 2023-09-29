#![allow(clippy::print_literal)]
#![allow(clippy::needless_return)]
#![allow(dropping_references)]
#![allow(clippy::assertions_on_constants)]

use crabrs::*;
use crabwebrs::*;
use log::*;
use std::path::PathBuf;
use std::process::*;
use std::*;

#[macro_use(defer)]
extern crate scopeguard;

fn main() -> ExitCode {
    env::set_var("RUST_BACKTRACE", "1"); //? not 100% sure this has 0 impact on performance? Maybe setting via command line instead of hardcoding is better?
                                         //env::set_var("RUST_LIB_BACKTRACE", "1");//? this line is useless?
                                         ////
    env::set_var("RUST_LOG", "trace"); //note this line must be above logger init.
    env_logger::init();

    let args: Vec<String> = env::args().collect(); //Note that std::env::args will panic if any argument contains invalid Unicode.
    fn the_end() {
        if std::thread::panicking() {
            info!("{}", "PANICKING");
        }
        info!("{}", "FINISHED");
    }
    defer! {
        the_end();
    }
    if main_inner(args).is_err() {
        return ExitCode::from(1);
    }
    ExitCode::from(0)
}

fn main_inner(args: Vec<String>) -> CustRes<()> {
    let cwd = env::current_dir()?;
    if env::var("DOWNLOADEREC_WITHOUT_CHK_EMPTY_FOLDER") != Ok("true".to_owned()) {
        if fs::read_dir(cwd)?.next().is_some() {
            return dummy_err("Folder is not empty. Aborted.");
        }
    }
    let mut off = match env::var("DOWNLOADEREC_OFFSET") {
        Ok(vstr) => vstr.parse::<u64>()?,
        Err(_) => 1,
    };
    let dur_millis = match env::var("DOWNLOADEREC_SLEEP_INTERVAL") {
        Ok(vstr) => vstr.parse::<u64>()?,
        Err(_) => 0,
    };
    let mut stdin_w = StdinWrapper::default();
    let mut urls: Vec<String> = vec![];
    let mut lst = vec![];
    loop {
        let iline = match stdin_w.lines.next() {
            None => {
                coutln!("Input ended.");
                break;
            }
            Some(Err(err)) => {
                let l_err: std::io::Error = err;
                return Err(l_err.into());
            }
            Some(Ok(linestr)) => linestr,
        };
        //let iline = iline.trim();
        //if !iline.is_empty() {
        //	urls.push(iline.into());
        //}
        urls.push(iline);
    }
    for iline in &urls {
        let ustr = iline.trim();
        if !ustr.is_empty() {
            lst.push(ustr);
        }
    }
    println!("{}{}", "NUMBER OF URL: ", lst.len());
    let rwc = reqwest::blocking::Client::new();
    let time_dur = time::Duration::from_millis(dur_millis);
    for ustr in lst {
        let filenm = format!("{:0>9}", off) + ".htm";
        off += 1;
        coutln!(ustr);
        let bytes = easy_http_bytes(rwc.get(ustr))?;
        coutln!(filenm);
        thread::sleep(time_dur);
        fs::write(filenm, bytes)?;
    }
    Ok(())
}
