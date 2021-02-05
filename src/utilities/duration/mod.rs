use std::{
    time::Duration
};

crate fn parse_duration(duration: String) -> Duration {
    let mut acc = 0u64;
    let mut dur = 0u64;

    for c in duration.chars() {
        match c {
            | '0'..'9' => {
                acc *= 10;
                acc += c.to_digit(10).unwrap() as u64;
            },
            | 'd' | 'D' => {
                dur += acc * 24 * 60 * 60;
                acc = 0;
            },
            | 'm' | 'M' => {
                dur += acc * 60;
                acc = 0;
            },
            | 'h' | 'H' => {
                dur += acc * 60 * 60;
                acc = 0;
            },
            | 's' | 'S' => {
                dur += acc * 60;
                acc = 0;
            },
            _ => unreachable!(),
        }
    }

    Duration::from_secs(dur)
}
