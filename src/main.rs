#[macro_use]
extern crate clap;
extern crate env_logger;
extern crate fuse;
#[macro_use]
extern crate log;
extern crate chan_signal;

use std::collections::HashMap;
use std::ffi::OsString;
use std::io::Error;

use chan_signal::Signal;
use clap::{App, Arg};

mod flags;
mod catfs;

#[derive(Default)]
struct FlagStorage {
    cat_from: OsString,
    cat_to: OsString,
    mount_point: OsString,
    mount_options: HashMap<String, String>,
    foreground: bool,
}

fn main() {
    env_logger::init().unwrap();

    let mut flags: FlagStorage = Default::default();

    let app = App::new("catfs")
        .about("Cache Anything FileSystem")
        .version(crate_version!());

    {
        let mut args = [
            flags::Flag {
                arg: Arg::with_name("foreground").short("f").help(
                    "Run catfs in foreground",
                ),
                value: &mut flags.foreground,
            },
            flags::Flag {
                arg: Arg::with_name("option").short("o").takes_value(true).help(
                    "Additional system-specific mount options. Be careful!",
                ),
                value: &mut flags.mount_options,
            },
            flags::Flag {
                arg: Arg::with_name("from").index(1).required(true).help(
                    "Cache files from this directory",
                ),
                value: &mut flags.cat_from,
            },
            flags::Flag {
                arg: Arg::with_name("to").index(2).required(true).help(
                    "Cache files to this directory",
                ),
                value: &mut flags.cat_to,
            },
            flags::Flag {
                arg: Arg::with_name("mountpoint").index(3).required(true).help(
                    "Expose the mount point at this directory",
                ),
                value: &mut flags.mount_point,
            },
        ];


        flags::parse_options(app, &mut args);
    }

    if flags.foreground {
        println!("foreground on");
    }

    let signal = chan_signal::notify(&[Signal::INT, Signal::TERM]);
    match catfs::CatFS::new(flags.cat_from.as_os_str(), flags.cat_to.as_os_str()) {
        Ok(catfs) => {
            unsafe {
                let res = fuse::spawn_mount(catfs, &flags.mount_point, &[]);
                match res {
                    Ok(session) => {
                        // unmount after we get signaled because session will go out of scope
                        signal.recv().unwrap();
                    }
                    Err(e) => error!("Cannot mount {:?}: {}", flags.mount_point, e),
                }
            }
        }
        Err(e) => error!("Cannot mount {:?}: {}", flags.mount_point, e),
    }
}
