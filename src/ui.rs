use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub fn spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["◌", "○", "◎", "◉", "●", "◉", "◎", "○", "◌"])
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}

pub fn success(msg: &str) {
    println!("{} {}", "✓".green(), msg);
}

pub fn removed(msg: &str) {
    println!("{} {}", "−".red(), msg);
}

pub fn warning(msg: &str) {
    println!("{} {}", "!".yellow(), msg);
}

#[allow(dead_code)]
pub fn action(msg: &str) {
    println!("{} {}", "→".cyan(), msg);
}

