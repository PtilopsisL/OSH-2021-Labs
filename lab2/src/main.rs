use std::env;
use std::io::prelude::*;
use std::io::{self, Write};
use std::io::{stdin, BufRead};
use std::process::{exit, Command, Stdio};
extern crate dirs;
extern crate regex;
extern crate whoami;
use regex::Regex;

fn main() -> ! {
    let re_find_curr_dir = Regex::new(r".+/").unwrap();
    let re_replace_to_home = Regex::new(r"(?P<y>\s{0,1})~").unwrap();
    let user_name = whoami::username();
    let device_name = whoami::devicename();
    let home = dirs::home_dir().expect("Get home dir failed!");
    loop {
        let current_dir = env::current_dir().expect("Get current dir failed!");
        let curr_dir_name =
            re_find_curr_dir.replace(current_dir.to_str().expect("to_str() failed!"), "");
        print!("[{}@{}] {} $ ", user_name, device_name, curr_dir_name);
        io::stdout().flush().unwrap();
        let mut cmd = String::new();
        for line_res in stdin().lock().lines() {
            let line = line_res.expect("Read a line from stdin failed");
            cmd = line;
            break;
        }
        let mut replace_to_home = String::new();
        replace_to_home.push_str("$y");
        replace_to_home.push_str(home.to_str().expect("to_str() failed!"));
        let cmd = re_replace_to_home.replace_all(&cmd, replace_to_home);

        let pipes = cmd.split("|");
        let mut prog_out = String::new();

        for progs in pipes {
            let mut args = progs.split_whitespace();
            let prog = args.next();

            

            match prog {
                None => continue,
                Some(prog) => match prog {
                    "cd" => {
                        let input_dir = args.next();
                        match input_dir {
                            None => {
                                env::set_current_dir(&home).expect("Changing current dir failed");
                            }
                            Some(input_dir) => {
                                let re = Regex::new("^~").unwrap();
                                let set_dir = re
                                    .replace(input_dir, home.to_str().expect("to_str() failed!"))
                                    .into_owned();
                                env::set_current_dir(set_dir).expect("Changing current dir failed");
                            }
                        }
                    }
                    "pwd" => {
                        let err = "Getting current dir failed";
                        println!("{}", env::current_dir().expect(err).to_str().expect(err));
                    }
                    "export" => {
                        for arg in args {
                            let mut assign = arg.split("=");
                            let name = assign.next().expect("No variable name");
                            let value = assign.next().expect("No variable value");
                            env::set_var(name, value);
                        }
                    }
                    "exit" => {
                        exit(0);
                    }
                    _ => {
                        let process = match Command::new(prog)
                            .args(args)
                            .stdin(Stdio::piped())
                            .stdout(Stdio::piped())
                            .spawn()
                        {
                            Err(why) => panic!("couldn't spawn process: {}", why),
                            Ok(process) => process,
                        };

                        match process.stdin.unwrap().write_all(prog_out.as_bytes()) {
                            Err(why) => panic!("couldn't write to process: {}", why),
                            Ok(_) => {}
                        }

                        prog_out.clear();

                        match process.stdout.unwrap().read_to_string(&mut prog_out) {
                            Err(why) => panic!("couldn't read stdout: {}", why),
                            Ok(_) => {}
                        }
                    }
                },
            }
        }

        if !prog_out.is_empty() {
            print!("{}", prog_out);
            io::stdout().flush().unwrap();
        }
    }
}
