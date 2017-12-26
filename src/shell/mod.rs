use alloc::String;

use dev::text_video::{TextColor, TextStyle};
use kio;

const PROMPT_STYLE: TextStyle = TextStyle {
    foreground: TextColor::White,
    background: TextColor::Black,
};

pub fn start() {
    print_header();
    loop {
        let cmd = prompt();
        println!("{:#?}", cmd);
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

fn prompt() -> String {
    kio::with_output_style(PROMPT_STYLE, || {
        print!("> ");

        // TODO:
        loop {}
    });

    "".into()
}
