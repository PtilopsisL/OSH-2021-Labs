use nix::sys::signal;
use regex::Regex;
use std::env;
use std::fs;
use std::io::prelude::*;
use std::io::{self, Write};
use std::io::{stdin, BufRead};
use std::net::TcpStream;
use std::os::unix::io::IntoRawFd;
use std::process::{exit, Command, Stdio};
use std::vec::Vec;

static mut PRE: String = String::new();
static NOCHANGE: i32 = 0;
static CREATE: i32 = 1;
static APPEND: i32 = 2;
static INPUT: i32 = 3;
static HERE: i32 = 4; // HERE Document input mode
static TEXTIN: i32 = 5;

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

fn reset_fd(reset: &Vec<(i32, i32)>) {
    for elem in reset {
        let (old_fd, new_fd) = elem;
        nix::unistd::dup2(*old_fd, *new_fd).unwrap();
        nix::unistd::close(*old_fd).unwrap();
    }
}

extern "C" fn handle_sigint(_num: i32) {
    unsafe {
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
    let re_create = Regex::new(r"(?P<x>[^>]{1})([0-9]*)>[\s]*([/0-9a-zA-Z\._]+)").unwrap();
    let re_append = Regex::new(r"(?P<x>[^>]{1})([0-9]*)>>[\s]*([/0-9a-zA-Z\._]+)").unwrap();
    let re_input = Regex::new(r"(?P<x>[^<]{1}){1}([0-9]*)<[\s]*([/0-9a-zA-Z\._]+)").unwrap();
    let re_here = Regex::new(r"(?P<x>[^<]{1})<<[\s]*([0-9a-zA-Z\._]+)").unwrap();
    let re_textin = Regex::new(r"(?P<x>[^<]{1})<<<[\s]*([\S]+)").unwrap();
    let re_fd_in = Regex::new(r"[\s]+([0-9]+)<&[\s]*([0-9]+)").unwrap();
    let re_fd_out = Regex::new(r"[\s]+([0-9]+)>&[\s]*([0-9]+)").unwrap();

    let re_tcp = Regex::new(r"/dev/tcp/([\d]+\.[\d]+\.[\d]+\.[\d]+)/([\d]+)").unwrap();

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
            println!("Warning: get home dir failed!: {}", e);
            home = String::from("")
        }
    }
    loop {
        let mut fd_reset_list: Vec<(i32, i32)> = Vec::new();
        let current_dir = env::current_dir().expect("Get current dir failed!");
        let curr_dir_name = re_find_curr_dir
            .replace(current_dir.to_str().expect("to_str() failed!"), "")
            .into_owned();
        unsafe {
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
        let mut cmd = re_replace_to_home
            .replace_all(&cmd, replace_to_home)
            .into_owned();

        let mut in_out_file = String::new();
        let mut redirect_out_state = NOCHANGE;
        let mut redirect_in_state = NOCHANGE;

        // regex match

        // CREATE
        for caps in re_create.captures_iter(&cmd) {
            let raw_fd_result = String::from(&caps[2]).parse::<i32>();
            let raw_fd: i32;
            match raw_fd_result {
                Ok(num) => raw_fd = num,
                Err(_) => raw_fd = 1,
            };

            in_out_file = String::from(&caps[3]);

            let fd_backup = nix::unistd::dup(raw_fd);
            match fd_backup {
                Ok(new_fd) => fd_reset_list.push((new_fd, raw_fd)),
                Err(_) => {}
            }

            let mut file_fd = raw_fd;
            let mut tcp_flag = false;
            for tcp_cap in re_tcp.captures_iter(&in_out_file) {
                tcp_flag = true;

                let ip_addr = String::from(&tcp_cap[1]);
                let port = String::from(&tcp_cap[2]);

                let tcp_stream = TcpStream::connect(ip_addr + ":" + &port)
                    .expect("Couldn't connect to the server...");
                file_fd = tcp_stream.into_raw_fd();
            }

            if tcp_flag == false {
                let file = fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(&in_out_file)
                    .unwrap();
                file_fd = file.into_raw_fd();
            }

            if file_fd == raw_fd {
                panic!("Redirect failed!\n");
            }

            nix::unistd::dup2(file_fd, raw_fd).unwrap();
            nix::unistd::close(file_fd).unwrap();
            redirect_out_state = CREATE;
        }

        if redirect_out_state == CREATE {
            cmd = re_create.replace_all(&cmd, "$x").into_owned();
        }
        // End of CREATE

        // APPEND
        for caps in re_append.captures_iter(&cmd) {
            let raw_fd_result = String::from(&caps[2]).parse::<i32>();
            let raw_fd: i32;
            match raw_fd_result {
                Ok(num) => raw_fd = num,
                Err(_) => raw_fd = 1,
            };

            in_out_file = String::from(&caps[3]);

            let fd_backup = nix::unistd::dup(raw_fd);
            match fd_backup {
                Ok(new_fd) => fd_reset_list.push((new_fd, raw_fd)),
                Err(_) => {}
            }

            let file = fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(&in_out_file)
                .unwrap();
            let file_fd = file.into_raw_fd();
            nix::unistd::dup2(file_fd, raw_fd).unwrap();
            nix::unistd::close(file_fd).unwrap();
            redirect_out_state = APPEND;
        }

        if redirect_out_state == APPEND {
            cmd = re_append.replace_all(&cmd, "$x").into_owned();
        }
        // End of APPEND

        // INPUT
        for caps in re_input.captures_iter(&cmd) {
            let raw_fd_result = String::from(&caps[2]).parse::<i32>();
            let raw_fd: i32;
            match raw_fd_result {
                Ok(num) => raw_fd = num,
                Err(_) => raw_fd = 0,
            };

            in_out_file = String::from(&caps[3]);

            let fd_backup = nix::unistd::dup(raw_fd);
            match fd_backup {
                Ok(new_fd) => fd_reset_list.push((new_fd, raw_fd)),
                Err(_) => {}
            }

            let mut file_fd = raw_fd;
            let mut tcp_flag = false;
            for tcp_cap in re_tcp.captures_iter(&in_out_file) {
                tcp_flag = true;

                let ip_addr = String::from(&tcp_cap[1]);
                let port = String::from(&tcp_cap[2]);

                let tcp_stream = TcpStream::connect(ip_addr + ":" + &port)
                    .expect("Couldn't connect to the server...");
                file_fd = tcp_stream.into_raw_fd();
            }

            if tcp_flag == false {
                let file = fs::OpenOptions::new()
                .read(true)
                .open(&in_out_file)
                .unwrap();
                file_fd = file.into_raw_fd();
            }

            if file_fd == raw_fd {
                panic!("Redirect failed!\n");
            }

            nix::unistd::dup2(file_fd, raw_fd).unwrap();
            nix::unistd::close(file_fd).unwrap();
            redirect_in_state = INPUT;
        }

        if redirect_in_state == INPUT {
            cmd = re_input.replace_all(&cmd, "$x").into_owned();
        }
        // End of INPUT

        for caps in re_here.captures_iter(&cmd) {
            in_out_file = String::from(&caps[2]);
            redirect_in_state = HERE;
        }

        if redirect_in_state == HERE {
            cmd = re_here.replace_all(&cmd, "$x").into_owned();
        }

        for caps in re_textin.captures_iter(&cmd) {
            in_out_file = String::from(&caps[2]);
            redirect_in_state = TEXTIN;
        }

        if redirect_in_state == TEXTIN {
            cmd = re_textin.replace_all(&cmd, "$x").into_owned();
        }

        for caps in re_fd_in.captures_iter(&cmd) {
            let raw_fd1 = String::from(&caps[1]).parse::<i32>().unwrap();
            let raw_fd2 = String::from(&caps[2]).parse::<i32>().unwrap();

            let fd_backup = nix::unistd::dup(raw_fd1);
            match fd_backup {
                Ok(new_fd) => fd_reset_list.push((new_fd, raw_fd1)),
                Err(_) => {}
            }

            nix::unistd::dup2(raw_fd2, raw_fd1).unwrap();
        }

        cmd = re_fd_in.replace_all(&cmd, "").into_owned();

        for caps in re_fd_out.captures_iter(&cmd) {
            let raw_fd1 = String::from(&caps[1]).parse::<i32>().unwrap();
            let raw_fd2 = String::from(&caps[2]).parse::<i32>().unwrap();

            let fd_backup = nix::unistd::dup(raw_fd1);
            match fd_backup {
                Ok(new_fd) => fd_reset_list.push((new_fd, raw_fd1)),
                Err(_) => {}
            }

            nix::unistd::dup2(raw_fd2, raw_fd1).unwrap();
        }

        cmd = re_fd_out.replace_all(&cmd, "").into_owned();

        let pipes = cmd.split("|");
        let mut prog_out = String::new();

        if redirect_in_state == HERE {
            print!("> ");
            io::stdout().flush().unwrap();
            for line_res in stdin().lock().lines() {
                let get_line = line_res.expect("Read a line from stdin failed");
                if get_line == in_out_file {
                    break;
                } else {
                    prog_out = prog_out + &get_line + "\n";
                }
                print!("> ");
                io::stdout().flush().unwrap();
            }
        } else if redirect_in_state == TEXTIN {
            prog_out = in_out_file.clone() + "\n";
        }

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
                        prog_out =
                            format!("{}\n", env::current_dir().expect(err).to_str().expect(err));
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

                        prog_out = format!("{}\n", value);
                    }
                    _ => {
                        let process;
                        if redirect_in_state == HERE || redirect_in_state == TEXTIN {
                            process = match Command::new(prog)
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
                        } else {
                            process = match Command::new(prog)
                                .args(args)
                                .stdout(Stdio::piped())
                                .spawn()
                            {
                                Err(why) => panic!("couldn't spawn process: {}", why),
                                Ok(process) => process,
                            };
                        }

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

        reset_fd(&fd_reset_list);
    }
}
