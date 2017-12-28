mod calc;

use alloc::Vec;

use dev;
use dev::kbd::Kbd;
use dev::text_video::{TextColor, TextStyle};
use kio;

const PROMPT_STYLE: TextStyle = TextStyle {
    foreground: TextColor::White,
    background: TextColor::Black,
};

const ERROR_STYLE: TextStyle = TextStyle {
    foreground: TextColor::LightRed,
    background: TextColor::Black,
};

pub fn start() {
    print_header();
    loop {
        let cmd = prompt();
        exec(&cmd);
    }
}

fn print_header() {
    const HEADER: &str = include_str!("header.txt");
    const COLORS: [TextColor; 8] = [
        TextColor::Green,
        TextColor::Green,
        TextColor::Yellow,
        TextColor::Red,
        TextColor::Red,
        TextColor::LightBlue,
        TextColor::LightBlue,
        TextColor::White,
    ];

    println!();

    for (line, &foreground) in HEADER.lines().zip(COLORS.iter().cycle()) {
        let style = TextStyle {
            foreground,
            background: TextColor::Black,
        };

        kio::with_output_style(style, || {
            println!("{}", line);
        })
    }
}

fn prompt() -> Vec<u8> {
    let kbd_dev = dev::mgr::get_device("kbd0").unwrap();
    let kbd = kbd_dev.downcast::<Kbd>();
    let mut line_vec = Vec::new();

    kio::with_output_style(PROMPT_STYLE, || {
        print!("> ");

        loop {
            let key: u8 = kbd.wait().into();
            if key == 0 {
                continue;
            }
            if key == b'\n' {
                println!();
                break;
            }
            print!("{}", key as char);
            line_vec.push(key);
        }
    });

    line_vec
}

fn exec(cmd: &[u8]) {
    match cmd {
        b"lsdev" => {
            let mut all = dev::mgr::all();
            all.sort_unstable_by_key(|d| d.name());
            for dev in all.iter() {
                println!("{}", dev.name());
            }
        }

        expr => match calc::eval(expr) {
            Ok(result) => println!("{}", result),
            Err(error) => {
                kio::with_output_style(ERROR_STYLE, || {
                    println!("error: {}", error);
                });
            }
        },
    }
}
