use nix::sys::signal;
use regex::Regex;
use std::env;
use std::io::prelude::*;
use std::io::{self, Write};
use std::io::{stdin, BufRead};
use std::process::{exit, Command, Stdio};

static mut PRE: String = String::new();

fn get_host_name() -> String {
    let replace_point = Regex::new(r"\..*").unwrap();

    let process = match Command::new("hostname")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
    {
        Err(why) => panic!("couldn't spawn process: {}", why),
        Ok(process) => process,
    };

    let mut devicename = String::new();

    match process.stdout.unwrap().read_to_string(&mut devicename) {
        Err(why) => panic!("couldn't read stdout: {}", why),
        Ok(_) => {}
    }

    devicename = devicename.replace("\n", "");

    devicename = replace_point.replace_all(&devicename, "").to_string();

    return devicename;
}

extern "C" fn handle_sigint(_num: i32) {
    unsafe{
        print!("\n{}", PRE);
        io::stdout().flush().unwrap();
    }
}

fn main() -> ! {
    let sigint_action = signal::SigAction::new(
        signal::SigHandler::Handler(handle_sigint),
        signal::SaFlags::empty(),
        signal::SigSet::empty(),
    );

    unsafe {
        let _sigint = signal::sigaction(signal::SIGINT, &sigint_action);
    }

    let re_find_curr_dir = Regex::new(r".+/").unwrap();
    let re_replace_to_home = Regex::new(r"(?P<y>\s{0,1})~").unwrap();

    let user_name;
    match env::var("USER") {
        Ok(val) => user_name = val,
        Err(_e) => user_name = String::from(""),
    }

    let host_name = get_host_name();

    let home;
    match env::var("HOME") {
        Ok(val) => home = val,
        Err(e) => {
            println!("Warning! get home dir failed!: {}", e);
            home = String::from("")
        }
    }
    loop {
        let current_dir = env::current_dir().expect("Get current dir failed!");
        let curr_dir_name =
            re_find_curr_dir.replace(current_dir.to_str().expect("to_str() failed!"), "").into_owned();
        unsafe{
            PRE = format!("[{}@{}] {} $ ", user_name, host_name, curr_dir_name);
        }
        print!("[{}@{}] {} $ ", user_name, host_name, curr_dir_name);
        io::stdout().flush().unwrap();

        let mut readin_flag = 0;
        let mut cmd = String::new();
        for line_res in stdin().lock().lines() {
            let line = line_res.expect("Read a line from stdin failed");
            readin_flag = 1;
            cmd = line;
            break;
        }

        if readin_flag == 0 {
            exit(0);
        }

        let mut replace_to_home = String::new();
        replace_to_home.push_str("$y");
        replace_to_home.push_str(&home);
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
                                let set_dir = re.replace(input_dir, &home).into_owned();
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
                    "echo" => {
                        let mut env_name;
                        match args.next() {
                            Option::Some(name) => env_name = String::from(name),
                            Option::None => env_name = String::from(""),
                        }

                        env_name = env_name.replace("$", "");

                        let value;
                        match env::var(env_name) {
                            Ok(val) => value = val,
                            Err(_e) => value = String::from(""),
                        }

                        println!("{}", value);
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
