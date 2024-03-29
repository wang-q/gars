extern crate clap;
use clap::*;

pub mod cmd_gars;

fn main() -> anyhow::Result<()> {
    let app = Command::new("gars")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Genome Analyst with Rust and rediS")
        .propagate_version(true)
        .arg_required_else_help(true)
        .color(ColorChoice::Auto)
        .subcommand(cmd_gars::env::make_subcommand())
        .subcommand(cmd_gars::status::make_subcommand())
        .subcommand(cmd_gars::gen::make_subcommand())
        .subcommand(cmd_gars::locate::make_subcommand())
        .subcommand(cmd_gars::range::make_subcommand())
        .subcommand(cmd_gars::clear::make_subcommand())
        .subcommand(cmd_gars::feature::make_subcommand())
        .subcommand(cmd_gars::fsw::make_subcommand())
        .subcommand(cmd_gars::anno::make_subcommand())
        .subcommand(cmd_gars::sliding::make_subcommand())
        .subcommand(cmd_gars::peak::make_subcommand())
        .subcommand(cmd_gars::tsv::make_subcommand());

    // Check which subcomamnd the user ran...
    match app.get_matches().subcommand() {
        Some(("env", sub_matches)) => cmd_gars::env::execute(sub_matches),
        Some(("status", sub_matches)) => cmd_gars::status::execute(sub_matches),
        Some(("gen", sub_matches)) => cmd_gars::gen::execute(sub_matches),
        Some(("locate", sub_matches)) => cmd_gars::locate::execute(sub_matches),
        Some(("range", sub_matches)) => cmd_gars::range::execute(sub_matches),
        Some(("clear", sub_matches)) => cmd_gars::clear::execute(sub_matches),
        Some(("feature", sub_matches)) => cmd_gars::feature::execute(sub_matches),
        Some(("fsw", sub_matches)) => cmd_gars::fsw::execute(sub_matches),
        Some(("anno", sub_matches)) => cmd_gars::anno::execute(sub_matches),
        Some(("sliding", sub_matches)) => cmd_gars::sliding::execute(sub_matches),
        Some(("peak", sub_matches)) => cmd_gars::peak::execute(sub_matches),
        Some(("tsv", sub_matches)) => cmd_gars::tsv::execute(sub_matches),
        _ => unreachable!(),
    }
    .unwrap();

    Ok(())
}

// TODO: sliding windows of waves
// TODO: `gars count`
