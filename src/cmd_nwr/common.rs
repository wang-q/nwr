use clap::*;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("common")
        .about("Output the common tree of terms")
        .after_help(include_str!("../../docs/help/common.md"))
        .arg(
            Arg::new("terms")
                .help("The NCBI Taxonomy ID or scientific name")
                .required(true)
                .num_args(1..)
                .index(1),
        )
        .arg(
            Arg::new("dir")
                .long("dir")
                .short('d')
                .num_args(1)
                .value_name("DIR")
                .help("Specify the NWR data directory"),
        )
        .arg(
            Arg::new("outfile")
                .short('o')
                .long("outfile")
                .num_args(1)
                .default_value("stdout")
                .help("Output filename (default: stdout)"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let mut writer = intspan::writer(args.get_one::<String>("outfile").unwrap());

    let nwrdir = nwr::get_nwr_dir(args, "dir")?;
    let conn = nwr::connect_txdb(&nwrdir)?;

    let terms: Vec<String> = args
        .get_many::<String>("terms")
        .ok_or_else(|| anyhow::anyhow!("No terms provided"))?
        .cloned()
        .collect();

    let out_string = nwr::libs::taxonomy::common::run(&conn, &terms)?;
    writer.write_all((out_string + "\n").as_ref())?;

    Ok(())
}
