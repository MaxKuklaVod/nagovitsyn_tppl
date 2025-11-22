#![allow(unused_imports)]

use std::env;
use std::fs;
use std::io::{self, BufRead, Write, Cursor};

#[derive(Debug, PartialEq, Clone)]
#[allow(non_camel_case_types)]
enum Command {
    moo, mOo, moO, mOO, Moo, MOo, MoO, MOO, OOO, MMM, OOM, oom,
}

#[cfg(not(test))]
fn main() {
    let args: Vec<String> = env::args().collect();
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut writer = io::stdout();

    let exit_code = run_app(args, &mut reader, &mut writer, |path| {
        fs::read_to_string(path).map_err(|e| e.to_string())
    });

    std::process::exit(exit_code);
}

fn run_app<F>(
    args: Vec<String>, 
    input: &mut dyn BufRead, 
    output: &mut dyn Write,
    file_reader: F
) -> i32 
where F: Fn(&str) -> Result<String, String>
{
    if args.len() < 2 {
        let _ = writeln!(output, "Error: No filename provided.");
        let _ = writeln!(output, "Usage: cargo run -- <filename.cow>");
        return 1;
    }

    let filename = &args[1];
    let code = match file_reader(filename) {
        Ok(c) => c,
        Err(e) => {
            let _ = writeln!(output, "Failed to read file '{}': {}", filename, e);
            return 1;
        }
    };

    if let Err(e) = run_cow(&code, input, output) {
        let _ = writeln!(output, "Runtime Error: {}", e);
        return 1;
    }

    0
}

fn run_cow(source_code: &str, input: &mut dyn BufRead, output: &mut dyn Write) -> Result<(), String> {
    let instructions = parse_code(source_code);
    
    let mut memory: Vec<u8> = vec![0; 30000];
    let mut pointer: usize = 0;
    let mut register: Option<u8> = None;
    let mut pc: usize = 0;

    while pc < instructions.len() {
        let new_pc = execute_command(
            &instructions[pc],
            &instructions,
            pc,
            &mut memory,
            &mut pointer,
            &mut register,
            input,
            output
        )?;
        pc = new_pc;
    }
    Ok(())
}

fn parse_code(code: &str) -> Vec<Command> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = code.chars().collect();
    let mut i = 0;

    while i + 2 < chars.len() {
        let chunk = format!("{}{}{}", chars[i], chars[i+1], chars[i+2]);
        
        let cmd = match chunk.as_str() {
            "moo" => Some(Command::moo),
            "mOo" => Some(Command::mOo),
            "moO" => Some(Command::moO),
            "mOO" => Some(Command::mOO),
            "Moo" => Some(Command::Moo),
            "MOo" => Some(Command::MOo),
            "MoO" => Some(Command::MoO),
            "MOO" => Some(Command::MOO),
            "OOO" => Some(Command::OOO),
            "MMM" => Some(Command::MMM),
            "OOM" => Some(Command::OOM),
            "oom" => Some(Command::oom),
            _ => None,
        };

        if let Some(c) = cmd {
            tokens.push(c);
            i += 3; 
        } else {
            i += 1; 
        }
    }
    tokens
}

fn execute_command(
    cmd: &Command,
    instructions: &Vec<Command>,
    mut pc: usize,
    memory: &mut Vec<u8>,
    pointer: &mut usize,
    register: &mut Option<u8>,
    input: &mut dyn BufRead,
    output: &mut dyn Write
) -> Result<usize, String> {
    match cmd {
        Command::moo => {
            let mut depth = 1;
            loop {
                if pc == 0 { return Err("Loop error: unmatched 'moo'".to_string()); }
                pc -= 1;
                match instructions[pc] {
                    Command::moo => depth += 1,
                    Command::MOO => depth -= 1,
                    _ => {}
                }
                if depth == 0 { return Ok(pc); }
            }
        }
        Command::mOo => if *pointer > 0 { *pointer -= 1; },
        Command::moO => if *pointer < memory.len() - 1 { *pointer += 1; },
        Command::mOO => {
            let code = memory[*pointer];
            if code == 3 { return Err("Infinite Loop Error (code 3 in mOO)".to_string()); }
            
            let cmd_to_run = match code {
                0 => Some(Command::moo), 1 => Some(Command::mOo), 2 => Some(Command::moO),
                3 => Some(Command::mOO), 4 => Some(Command::Moo), 5 => Some(Command::MOo),
                6 => Some(Command::MoO), 7 => Some(Command::MOO), 8 => Some(Command::OOO),
                9 => Some(Command::MMM), 10 => Some(Command::OOM), 11 => Some(Command::oom),
                _ => None,
            };

            if let Some(c) = cmd_to_run {
                execute_command(&c, instructions, pc, memory, pointer, register, input, output)?;
            } else {
                return Err(format!("Unknown command code: {}", code));
            }
        }
        Command::Moo => {
            if memory[*pointer] == 0 {
                memory[*pointer] = read_char(input);
            } else {
                if let Some(c) = char::from_u32(memory[*pointer] as u32) {
                    write!(output, "{}", c).map_err(|e| e.to_string())?;
                } else {
                    write!(output, "{}", memory[*pointer]).map_err(|e| e.to_string())?;
                }
                output.flush().map_err(|e| e.to_string())?;
            }
        }
        Command::MOo => memory[*pointer] = memory[*pointer].wrapping_sub(1),
        Command::MoO => memory[*pointer] = memory[*pointer].wrapping_add(1),
        Command::MOO => {
            if memory[*pointer] == 0 {
                let mut depth = 1;
                while depth > 0 {
                    pc += 1;
                    if pc >= instructions.len() { return Err("Loop error: unmatched 'MOO'".to_string()); }
                    match instructions[pc] {
                        Command::MOO => depth += 1,
                        Command::moo => depth -= 1,
                        _ => {}
                    }
                }
            }
        }
        Command::OOO => memory[*pointer] = 0,
        Command::MMM => {
            if register.is_none() {
                *register = Some(memory[*pointer]);
            } else {
                memory[*pointer] = register.unwrap();
                *register = None;
            }
        }
        Command::OOM => {
            write!(output, "{}", memory[*pointer]).map_err(|e| e.to_string())?;
            output.flush().map_err(|e| e.to_string())?;
        }
        Command::oom => memory[*pointer] = read_integer(input),
    }
    Ok(pc + 1)
}

fn read_integer(reader: &mut dyn BufRead) -> u8 {
    let mut input = String::new();
    if reader.read_line(&mut input).is_ok() {
        return input.trim().parse().unwrap_or(0);
    }
    0
}

fn read_char(reader: &mut dyn BufRead) -> u8 {
    let mut input = String::new();
    if reader.read_line(&mut input).is_ok() {
        if let Some(c) = input.chars().next() {
            return c as u8;
        }
    }
    0
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    fn run_cow_test(code: &str, input_data: &str) -> (Result<(), String>, String) {
        let mut input = Cursor::new(input_data.as_bytes()); 
        let mut output = Vec::new(); 
        let res = run_cow(code, &mut input, &mut output);
        let output_str = String::from_utf8(output).unwrap();
        (res, output_str)
    }

    fn run_app_test(args: Vec<String>, file_content: Result<String, String>) -> (i32, String) {
        let mut input = Cursor::new("");
        let mut output = Vec::new();
        
        let file_reader = |_path: &str| -> Result<String, String> {
            file_content.clone()
        };

        let code = run_app(args, &mut input, &mut output, file_reader);
        let output_str = String::from_utf8(output).unwrap();
        (code, output_str)
    }

    #[test]
    fn test_cli_no_args() {
        let args = vec!["program_name".to_string()];
        let (exit_code, out) = run_app_test(args, Ok("".to_string()));
        assert_eq!(exit_code, 1);
        assert!(out.contains("Error: No filename provided"));
    }

    #[test]
    fn test_cli_file_not_found() {
        let args = vec!["program".to_string(), "bad.cow".to_string()];
        let (exit_code, out) = run_app_test(args, Err("Not found".to_string()));
        assert_eq!(exit_code, 1);
        assert!(out.contains("Failed to read file 'bad.cow'"));
    }

    #[test]
    fn test_cli_runtime_error() {
        let code = "moo"; 
        let args = vec!["program".to_string(), "script.cow".to_string()];
        let (exit_code, out) = run_app_test(args, Ok(code.to_string()));
        assert_eq!(exit_code, 1);
        assert!(out.contains("Runtime Error"));
    }

    #[test]
    fn test_cli_success() {
        let code = "MoO OOM"; 
        let args = vec!["program".to_string(), "good.cow".to_string()];
        let (exit_code, _) = run_app_test(args, Ok(code.to_string()));
        assert_eq!(exit_code, 0);
    }

    #[test]
    fn test_parsing() {
        let code = "MoO   moO invalid text mOo";
        let tokens = parse_code(code);
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Command::MoO);
    }

    #[test]
    fn test_math() {
        let code = "MoO MOo MOo OOM";
        let (res, out) = run_cow_test(code, "");
        assert!(res.is_ok());
        assert_eq!(out, "255");
    }

    #[test]
    fn test_pointer_move() {
        let code = "mOo moO MoO mOo OOM"; 
        let (res, out) = run_cow_test(code, "");
        assert!(res.is_ok());
        assert_eq!(out, "0");
    }
    
    #[test]
    fn test_pointer_boundary_right() {
        let code = "moO";
        let (res, _) = run_cow_test(code, "");
        assert!(res.is_ok());
    }

    #[test]
    fn test_loop_skip() {
        let code = "MOO MoO MoO moo OOM"; 
        let (res, out) = run_cow_test(code, "");
        assert!(res.is_ok());
        assert_eq!(out, "0");
    }

    #[test]
    fn test_loop_enter_and_back() {
        let code = "MoO MOO MOo moo OOM";
        let (res, out) = run_cow_test(code, "");
        assert!(res.is_ok());
        assert_eq!(out, "0");
    }

    #[test]
    fn test_loop_nested() {
        let code = "MOO MOO MoO moo moo OOM";
        let (res, out) = run_cow_test(code, "");
        assert!(res.is_ok());
        assert_eq!(out, "0");
    }

    #[test]
    fn test_error_unmatched_moo() {
        let code = "moo";
        let (res, _) = run_cow_test(code, "");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "Loop error: unmatched 'moo'");
    }

    #[test]
    fn test_error_unmatched_MOO() {
        let code = "MOO";
        let (res, _) = run_cow_test(code, "");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "Loop error: unmatched 'MOO'");
    }

    #[test]
    fn test_register_mmm() {
        let code = "MoO MMM OOO MMM OOM";
        let (res, out) = run_cow_test(code, "");
        assert!(res.is_ok());
        assert_eq!(out, "1");
    }

    #[test]
    fn test_io_integer() {
        let code = "oom OOM";
        let (res, out) = run_cow_test(code, "123\n");
        assert!(res.is_ok());
        assert_eq!(out, "123");
    }

    #[test]
    fn test_io_char() {
        let code = "Moo MoO Moo"; 
        let (res, out) = run_cow_test(code, "A");
        assert!(res.is_ok());
        assert_eq!(out, "B");
    }

    #[test]
    fn test_mOO_execution() {
        let code = "MoO MoO MoO MoO MoO MoO mOO OOM";
        let (res, out) = run_cow_test(code, "");
        assert!(res.is_ok());
        assert_eq!(out, "7");
    }

    #[test]
    fn test_mOO_error_3() {
        let code = "MoO MoO MoO mOO";
        let (res, _) = run_cow_test(code, "");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "Infinite Loop Error (code 3 in mOO)");
    }

    #[test]
    fn test_mOO_error_unknown() {
        let code = "MOo mOO";
        let (res, _) = run_cow_test(code, "");
        assert!(res.is_err());
    }

    #[test]
    fn test_zero_memory() {
        let code = "MoO OOO OOM"; 
        let (res, out) = run_cow_test(code, "");
        assert!(res.is_ok());
        assert_eq!(out, "0");
    }

    #[test]
    fn test_read_eof() {
        let code = "oom OOM";
        let (res, out) = run_cow_test(code, ""); 
        assert!(res.is_ok());
        assert_eq!(out, "0");
    }
}