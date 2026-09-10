use std::env;
use std::fs;
use std::io::{self, IsTerminal, Read, Write};
use std::process::{Command, Stdio};
use terminal_size::terminal_size;

mod render;
mod syntax;
mod table;
mod terminal;

const VERSION: &str = "1.1.0";

struct CliArgs {
    file: Option<String>,
    no_pager: bool,
    width: Option<usize>,
}

fn print_help() {
    println!(
        "mdview {} — Vibrant High-Fidelity CLI Markdown Reader

USAGE:
    mdview [OPTIONS] [FILE]
    md [OPTIONS] [FILE]

ARGS:
    <FILE>    Markdown file to view (or - for standard input)

OPTIONS:
    -p, --no-pager               Do not pipe output into a pager
    -w, --width <COLS>           Override terminal display width
    -c, --completions <SHELL>    Generate shell completion script (zsh, fish, bash)
    -v, -V, --version            Print version information
    -h, --help                   Print help information
",
        VERSION
    );
}

fn print_completions(shell: &str) {
    match shell.to_lowercase().as_str() {
        "zsh" => print!("{}", include_str!("../completions/zsh/_mdview")),
        "fish" => print!("{}", include_str!("../completions/fish/mdview.fish")),
        "bash" => print!("{}", include_str!("../completions/bash/mdview.bash")),
        other => {
            eprintln!(
                "Error: Unsupported shell '{}'. Supported shells: zsh, fish, bash",
                other
            );
            std::process::exit(1);
        }
    }
}

fn parse_args() -> Result<CliArgs, String> {
    let mut args = env::args().skip(1);
    let mut file = None;
    let mut no_pager = false;
    let mut width = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            "-v" | "-V" | "--version" => {
                println!("mdview {}", VERSION);
                std::process::exit(0);
            }
            "-c" | "--completions" => {
                if let Some(shell) = args.next() {
                    print_completions(&shell);
                    std::process::exit(0);
                } else {
                    return Err(
                        "Option '--completions' requires a shell name (zsh, fish, bash)"
                            .to_string(),
                    );
                }
            }
            s if s.starts_with("--completions=") => {
                let shell = &s["--completions=".len()..];
                print_completions(shell);
                std::process::exit(0);
            }
            "-p" | "--no-pager" => {
                no_pager = true;
            }
            "-w" | "--width" => {
                if let Some(val) = args.next() {
                    match val.parse::<usize>() {
                        Ok(w) => width = Some(w),
                        Err(_) => return Err(format!("Invalid width value: '{}'", val)),
                    }
                } else {
                    return Err("Option '--width' requires a value".to_string());
                }
            }
            s if s.starts_with("--width=") => {
                let val = &s["--width=".len()..];
                match val.parse::<usize>() {
                    Ok(w) => width = Some(w),
                    Err(_) => return Err(format!("Invalid width value: '{}'", val)),
                }
            }
            s if s.starts_with('-') && s != "-" => {
                return Err(format!("Unknown option: '{}'", s));
            }
            other => {
                if file.is_none() {
                    file = Some(other.to_string());
                } else {
                    return Err(format!("Unexpected argument: '{}'", other));
                }
            }
        }
    }

    Ok(CliArgs {
        file,
        no_pager,
        width,
    })
}

fn get_terminal_width(custom_width: Option<usize>) -> usize {
    if let Some(w) = custom_width {
        return w.max(40);
    }
    if let Some((terminal_size::Width(w), _)) = terminal_size() {
        return (w as usize).clamp(40, 100);
    }
    if let Ok(cols) = env::var("COLUMNS")
        && let Ok(w) = cols.parse::<usize>()
    {
        return w.clamp(40, 100);
    }
    80
}

fn format_bat_header(path: &str, width: usize) -> String {
    let path_obj = std::path::Path::new(path);
    let display_name = if path_obj.is_absolute() {
        env::current_dir()
            .ok()
            .and_then(|cwd| path_obj.strip_prefix(&cwd).ok())
            .and_then(|rel| rel.to_str())
            .or_else(|| path_obj.file_name().and_then(|s| s.to_str()))
            .unwrap_or(path)
    } else {
        path
    };

    let border_color = "\x1b[38;2;90;90;125m";
    let reset = "\x1b[0m";
    let margin = "  ";
    let bar_len = width.saturating_sub(4);
    let bar = "─".repeat(bar_len);

    format!(
        "{margin}{border_color}{bar}{reset}\n\
         {margin}\x1b[38;2;130;130;160mFile: \x1b[1;38;2;84;160;255m{display_name}{reset}\n\
         {margin}{border_color}{bar}{reset}\n\n"
    )
}

fn format_bat_footer(width: usize) -> String {
    let border_color = "\x1b[38;2;90;90;125m";
    let reset = "\x1b[0m";
    let margin = "  ";
    let bar_len = width.saturating_sub(4);
    let bar = "─".repeat(bar_len);

    format!("\n\n{margin}{border_color}{bar}{reset}\n")
}

fn output_with_pager(rendered: &str, no_pager: bool) -> io::Result<()> {
    if no_pager || !io::stdout().is_terminal() {
        let mut stdout = io::stdout().lock();
        stdout.write_all(rendered.as_bytes())?;
        stdout.write_all(b"\n")?;
        return Ok(());
    }

    let pager_env = env::var("PAGER").unwrap_or_else(|_| "less".to_string());
    let mut parts = pager_env.split_whitespace();
    let pager_bin = parts.next().unwrap_or("less");
    let mut pager_args: Vec<&str> = parts.collect();

    // Default flags for less to pass ANSI colors and avoid clearing screen
    if (pager_bin == "less" || pager_bin.ends_with("/less"))
        && pager_args.is_empty()
        && env::var("LESS").is_err()
    {
        pager_args.push("-R"); // Raw control characters (ANSI colors)
        pager_args.push("-F"); // Quit if one screen
        pager_args.push("-X"); // Don't clear screen
    }

    match Command::new(pager_bin)
        .args(&pager_args)
        .env("LESSCHARSET", "utf-8")
        .env(
            "LESSUTFCHARDEF",
            "E000-F8FF:p,F0000-FFFFD:p,100000-10FFFD:p",
        )
        .stdin(Stdio::piped())
        .spawn()
    {
        Ok(mut child) => {
            if let Some(mut stdin) = child.stdin.take() {
                // Ignore BrokenPipe if user exits pager early with 'q'
                let _ = stdin.write_all(rendered.as_bytes());
                let _ = stdin.write_all(b"\n");
            }
            let _ = child.wait();
            Ok(())
        }
        Err(_) => {
            // Fallback to standard output if pager fails to execute
            let mut stdout = io::stdout().lock();
            stdout.write_all(rendered.as_bytes())?;
            stdout.write_all(b"\n")?;
            Ok(())
        }
    }
}

fn main() {
    let args = match parse_args() {
        Ok(a) => a,
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    };

    let content = match args.file.as_deref() {
        None | Some("-") => {
            if args.file.is_none() && io::stdin().is_terminal() {
                print_help();
                std::process::exit(1);
            }
            let mut buf = String::new();
            if let Err(e) = io::stdin().read_to_string(&mut buf) {
                eprintln!("Error reading stdin: {}", e);
                std::process::exit(1);
            }
            buf
        }
        Some(path) => match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error reading '{}': {}", path, e);
                std::process::exit(1);
            }
        },
    };

    let term_width = get_terminal_width(args.width);
    let renderer = render::MarkdownRenderer::new(term_width);
    let rendered = renderer.render(&content);

    let file_path = args.file.as_deref().filter(|p| *p != "-");
    let mut full_output = String::new();
    if let Some(path) = file_path {
        full_output.push_str(&format_bat_header(path, term_width));
    }
    full_output.push_str(&rendered);
    if file_path.is_some() {
        full_output.push_str(&format_bat_footer(term_width));
    }

    if let Err(e) = output_with_pager(&full_output, args.no_pager)
        && e.kind() != io::ErrorKind::BrokenPipe
    {
        eprintln!("Error writing output: {}", e);
    }
}
