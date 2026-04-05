extern crate clap;
use clap::*;

pub mod cmd_nwr;

fn main() -> anyhow::Result<()> {
    let app = Command::new("nwr")
        .version(crate_version!())
        .author(crate_authors!())
        .about("`nwr` is a command line **N**CBI taxonomy and assembly **WR**angler")
        .propagate_version(true)
        .arg_required_else_help(true)
        .color(ColorChoice::Auto)
        // Database
        .subcommand(cmd_nwr::download::make_subcommand())
        .subcommand(cmd_nwr::txdb::make_subcommand())
        .subcommand(cmd_nwr::ardb::make_subcommand())
        // Taxonomy
        .subcommand(cmd_nwr::info::make_subcommand())
        .subcommand(cmd_nwr::lineage::make_subcommand())
        .subcommand(cmd_nwr::member::make_subcommand())
        .subcommand(cmd_nwr::append::make_subcommand())
        .subcommand(cmd_nwr::restrict::make_subcommand())
        .subcommand(cmd_nwr::common::make_subcommand())
        // Assembly
        .subcommand(cmd_nwr::template::make_subcommand())
        .subcommand(cmd_nwr::abbr::make_subcommand())
        .subcommand(cmd_nwr::kb::make_subcommand())
        .subcommand(cmd_nwr::seqdb::make_subcommand())
        .after_help(
            r###"Subcommand groups:

* Database
    * download / txdb / ardb
* Taxonomy
    * info / lineage / member / append / restrict / common
* Assembly
    * template / abbr / kb / seqdb
"###,
        );

    // Check which subcomamnd the user ran...
    match app.get_matches().subcommand() {
        Some(("download", sub_matches)) => cmd_nwr::download::execute(sub_matches),
        Some(("txdb", sub_matches)) => cmd_nwr::txdb::execute(sub_matches),
        Some(("ardb", sub_matches)) => cmd_nwr::ardb::execute(sub_matches),
        Some(("info", sub_matches)) => cmd_nwr::info::execute(sub_matches),
        Some(("lineage", sub_matches)) => cmd_nwr::lineage::execute(sub_matches),
        Some(("member", sub_matches)) => cmd_nwr::member::execute(sub_matches),
        Some(("append", sub_matches)) => cmd_nwr::append::execute(sub_matches),
        Some(("restrict", sub_matches)) => cmd_nwr::restrict::execute(sub_matches),
        Some(("common", sub_matches)) => cmd_nwr::common::execute(sub_matches),
        Some(("template", sub_matches)) => cmd_nwr::template::execute(sub_matches),
        Some(("abbr", sub_matches)) => cmd_nwr::abbr::execute(sub_matches),
        Some(("kb", sub_matches)) => cmd_nwr::kb::execute(sub_matches),
        Some(("seqdb", sub_matches)) => cmd_nwr::seqdb::execute(sub_matches),
        Some((cmd, _)) => Err(anyhow::anyhow!("Unknown subcommand: {}", cmd)),
        None => Err(anyhow::anyhow!("No subcommand provided")),
    }?;

    Ok(())
}
