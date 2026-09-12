use std::env;
use std::fs;
use std::io::{self, IsTerminal, Read, Write};

mod layout;
mod render;
mod syntax;
mod table;
mod terminal;
mod viewer;

use layout::{apply_centering, compute_layout, format_bat_footer, format_bat_header};

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
    -p, --no-pager               Print directly to stdout without interactive viewer
    -w, --width <COLS>           Override display width (0 for full terminal width)
    -c, --completions <SHELL>    Generate shell completion script (zsh, fish, bash)
    -v, -V, --version            Print version information
    -h, --help                   Print help information

KEYBINDINGS (Interactive Mode):
    j / k, ↓ / ↑           Scroll down / up by 1 line
    d / u, Ctrl+d / u      Scroll down / up by half page
    f / b, PageDown / Up   Scroll down / up by full page
    gg / G                 Jump to beginning / end of document
    <number>G              Jump to specific line number
    /pattern, ?pattern     Search forward / backward
    n / N                  Next / previous search match
    :q, q, ZZ              Quit viewer
    :help, F1              Toggle in-app help screen
    Touchpad / Mouse       Smooth vertical scrolling
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

    let file_path = args.file.as_deref().filter(|p| *p != "-");

    // Full terminal window interactive viewer (Neovim-style) when stdout is a TTY
    if !args.no_pager && io::stdout().is_terminal() {
        let mut v = viewer::Viewer::new(&content, file_path, args.width);
        if let Err(e) = v.run() {
            eprintln!("Error running viewer: {}", e);
            std::process::exit(1);
        }
        return;
    }

    // Direct stdout output (when piped, redirected, or --no-pager is requested)
    let layout = compute_layout(args.width, None, false);
    let renderer = render::MarkdownRenderer::new(layout.content_width);
    let rendered = renderer.render(&content);

    let mut full_output = String::new();
    if let Some(path) = file_path {
        full_output.push_str(&format_bat_header(path, layout.content_width));
    }
    full_output.push_str(&rendered);
    if file_path.is_some() {
        full_output.push_str(&format_bat_footer(layout.content_width));
    }

    let final_output = apply_centering(&full_output, layout.left_pad);
    let mut stdout_lock = io::stdout().lock();
    let _ = stdout_lock.write_all(final_output.as_bytes());
    let _ = stdout_lock.write_all(b"\n");
}
