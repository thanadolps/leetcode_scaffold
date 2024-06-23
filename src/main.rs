mod api;
mod generate;
mod parse;

use std::{
    fs::{self, OpenOptions},
    io::Write,
    process::Command,
};

use color_eyre::{eyre::Context, Result};

fn main() -> Result<()> {
    color_eyre::install()?;

    let Some(kw) = std::env::args().nth(1) else {
        eprintln!(
            "Usage: {} <problem-name-or-id>",
            std::env::args().next().unwrap_or("creator".to_owned()),
        );
        std::process::exit(1);
    };

    // Fetch problem data
    let problem = api::get_question_name(&kw)
        .wrap_err("failed to find problem corresponding the input name")?;
    eprintln!("Found coresponding problem: {problem}");
    let (name, question) =
        api::fetch_problem_full_data(&problem).wrap_err("failed to fetch problem's data")?;

    // Generate crate data
    let crate_name = format!("{}_{}", name.replace('-', "_"), question.questionFrontendId);
    let path = &crate_name;
    let crate_code = generate::generate_code(&question)?;

    // Create crate from data
    create_crate(path, &crate_name, &crate_code)?;

    Ok(())
}

fn create_crate(path: &str, crate_name: &str, crate_code: &str) -> Result<()> {
    const TEMPLATE_CARGO: &str = include_str!("template-cargo.toml");

    Command::new("cargo-ws")
        .args([
            "ws",
            "create",
            path,
            "--lib",
            "--name",
            crate_name,
            "--edition",
            "2021",
        ])
        .status()?;

    // Write cargo.toml
    let crate_cargo = TEMPLATE_CARGO.trim_start_matches("[dependencies]").trim();
    OpenOptions::new()
        .append(true)
        .open(format!("{path}/Cargo.toml"))?
        .write_all(crate_cargo.as_bytes())?;

    // Write code
    fs::write(format!("{path}/src/lib.rs"), crate_code)?;
    Ok(())
}
