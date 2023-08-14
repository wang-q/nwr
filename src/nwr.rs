extern crate clap;
use clap::*;

mod cmd_nwr;

fn main() -> anyhow::Result<()> {
    let app = Command::new("nwr")
        .version(crate_version!())
        .author(crate_authors!())
        .about("`nwr` is a command line tool for working with NCBI taxonomy, assembly reports and Newick files")
        .propagate_version(true)
        .arg_required_else_help(true)
        .color(ColorChoice::Auto)
        .subcommand(cmd_nwr::append::make_subcommand())
        .subcommand(cmd_nwr::ardb::make_subcommand())
        .subcommand(cmd_nwr::comment::make_subcommand())
        .subcommand(cmd_nwr::download::make_subcommand())
        .subcommand(cmd_nwr::indent::make_subcommand())
        .subcommand(cmd_nwr::info::make_subcommand())
        .subcommand(cmd_nwr::kb::make_subcommand())
        .subcommand(cmd_nwr::lineage::make_subcommand())
        .subcommand(cmd_nwr::member::make_subcommand())
        .subcommand(cmd_nwr::order::make_subcommand())
        .subcommand(cmd_nwr::restrict::make_subcommand())
        .subcommand(cmd_nwr::template::make_subcommand())
        .subcommand(cmd_nwr::tex::make_subcommand())
        .subcommand(cmd_nwr::txdb::make_subcommand())
        .after_help(
            r###"
Subcommand groups:

* Database
    * download
    * txdb
    * ardb

* Taxonomy
    * info
    * lineage
    * member
    * append
    * restrict

* Newick
    * indent
    * order
    * comment
    * tex

* Assembly
    * template
    * kb

"###,
        );

    // Check which subcomamnd the user ran...
    match app.get_matches().subcommand() {
        Some(("append", sub_matches)) => cmd_nwr::append::execute(sub_matches),
        Some(("ardb", sub_matches)) => cmd_nwr::ardb::execute(sub_matches),
        Some(("comment", sub_matches)) => cmd_nwr::comment::execute(sub_matches),
        Some(("download", sub_matches)) => cmd_nwr::download::execute(sub_matches),
        Some(("indent", sub_matches)) => cmd_nwr::indent::execute(sub_matches),
        Some(("info", sub_matches)) => cmd_nwr::info::execute(sub_matches),
        Some(("kb", sub_matches)) => cmd_nwr::kb::execute(sub_matches),
        Some(("lineage", sub_matches)) => cmd_nwr::lineage::execute(sub_matches),
        Some(("member", sub_matches)) => cmd_nwr::member::execute(sub_matches),
        Some(("order", sub_matches)) => cmd_nwr::order::execute(sub_matches),
        Some(("restrict", sub_matches)) => cmd_nwr::restrict::execute(sub_matches),
        Some(("template", sub_matches)) => cmd_nwr::template::execute(sub_matches),
        Some(("tex", sub_matches)) => cmd_nwr::tex::execute(sub_matches),
        Some(("txdb", sub_matches)) => cmd_nwr::txdb::execute(sub_matches),
        _ => unreachable!(),
    }
    .unwrap();

    Ok(())
}

// TODO: abbr_name.pl
// TODO: nw_reroot
// TODO: nw_topology
