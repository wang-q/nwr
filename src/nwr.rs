extern crate clap;
use clap::*;

pub mod cmd_nwr;

fn main() -> anyhow::Result<()> {
    let app = Command::new("nwr")
        .version(crate_version!())
        .author(crate_authors!())
        .about("`nwr` is a command line tool for working with NCBI taxonomy, Newick files and assembly reports")
        .propagate_version(true)
        .arg_required_else_help(true)
        .color(ColorChoice::Auto)
        .subcommand(cmd_nwr::download::make_subcommand())
        .subcommand(cmd_nwr::txdb::make_subcommand())
        .subcommand(cmd_nwr::ardb::make_subcommand())
        .subcommand(cmd_nwr::info::make_subcommand())
        .subcommand(cmd_nwr::lineage::make_subcommand())
        .subcommand(cmd_nwr::member::make_subcommand())
        .subcommand(cmd_nwr::append::make_subcommand())
        .subcommand(cmd_nwr::restrict::make_subcommand())
        .subcommand(cmd_nwr::common::make_subcommand())
        .subcommand(cmd_nwr::template::make_subcommand())
        .subcommand(cmd_nwr::kb::make_subcommand())
        .subcommand(cmd_nwr::seqdb::make_subcommand())
        .subcommand(cmd_nwr::data::make_subcommand())
        .subcommand(cmd_nwr::ops::make_subcommand())
        .subcommand(cmd_nwr::pl_condense::make_subcommand())
        .subcommand(cmd_nwr::viz::make_subcommand())
        .after_help(
            r###"
Subcommand groups:

* Database
    * download / txdb / ardb

* Taxonomy
    * info / lineage / member / append / restrict / common

* Assembly
    * template
    * kb
    * seqdb

* Newick
    * data
        * label / stat / distance
    * ops (operation)
        * order / rename / replace / topo / subtree / prune / reroot
        * pl-condense
    * viz (visualization)
        * indent / comment / tex

"###,
        );

    // Check which subcomamnd the user ran...
    match app.get_matches().subcommand() {
        // Database
        Some(("download", sub_matches)) => cmd_nwr::download::execute(sub_matches),
        Some(("txdb", sub_matches)) => cmd_nwr::txdb::execute(sub_matches),
        Some(("ardb", sub_matches)) => cmd_nwr::ardb::execute(sub_matches),
        // Taxonomy
        Some(("info", sub_matches)) => cmd_nwr::info::execute(sub_matches),
        Some(("lineage", sub_matches)) => cmd_nwr::lineage::execute(sub_matches),
        Some(("member", sub_matches)) => cmd_nwr::member::execute(sub_matches),
        Some(("append", sub_matches)) => cmd_nwr::append::execute(sub_matches),
        Some(("restrict", sub_matches)) => cmd_nwr::restrict::execute(sub_matches),
        Some(("common", sub_matches)) => cmd_nwr::common::execute(sub_matches),
        // Assembly
        Some(("template", sub_matches)) => cmd_nwr::template::execute(sub_matches),
        Some(("kb", sub_matches)) => cmd_nwr::kb::execute(sub_matches),
        Some(("seqdb", sub_matches)) => cmd_nwr::seqdb::execute(sub_matches),
        // Newick data
        Some(("data", sub_matches)) => cmd_nwr::data::execute(sub_matches),
        // Newick operation
        Some(("ops", sub_matches)) => cmd_nwr::ops::execute(sub_matches),
        Some(("pl-condense", sub_matches)) => cmd_nwr::pl_condense::execute(sub_matches),
        // Newick visualization
        Some(("viz", sub_matches)) => cmd_nwr::viz::execute(sub_matches),
        _ => unreachable!(),
    }?;

    Ok(())
}

// TODO: `compgen -c nw_`
