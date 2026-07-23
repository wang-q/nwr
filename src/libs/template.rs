use lazy_static::lazy_static;
use regex::Regex;
use std::io::Write;
use tera::{Context, Tera};

/// Assembly level code for a complete genome.
pub const LEVEL_COMPLETE_GENOME: &str = "1";
/// Assembly level code for a chromosome-level assembly.
pub const LEVEL_CHROMOSOME: &str = "2";
/// Assembly level code for a scaffold-level assembly.
pub const LEVEL_SCAFFOLD: &str = "3";
/// Assembly level code for a contig-level assembly (same as scaffold here).
pub const LEVEL_CONTIG: &str = "3"; // Same as SCAFFOLD - both are treated as level 3
/// Assembly level code for other incomplete assemblies.
pub const LEVEL_OTHER: &str = "5";

lazy_static! {
    static ref RE_URL: Regex =
        Regex::new(r#"(?xi)(ftp|https?)://ftp\.ncbi\.nlm\.nih\.gov/"#).unwrap();
}

/// Validate that a string is safe to embed into generated shell scripts and
/// to use as a file or directory name. Only ASCII alphanumeric characters,
/// underscores, hyphens and dots are allowed. A leading hyphen is rejected
/// to prevent the value from being interpreted as a command-line flag.
pub fn validate_shell_safe(s: &str) -> anyhow::Result<&str> {
    if s.is_empty() {
        return Err(anyhow::anyhow!("Shell-safe identifier must not be empty"));
    }
    // Reject "." and ".." to prevent path traversal when used as file/directory names.
    if s == "." || s == ".." {
        return Err(anyhow::anyhow!(
            "Shell-safe identifier must not be '.' or '..': '{}'",
            s
        ));
    }
    // Reject leading hyphen so the value is not mistaken for a CLI flag.
    if s.starts_with('-') {
        return Err(anyhow::anyhow!(
            "Shell-safe identifier must not start with '-': '{}'",
            s
        ));
    }
    if s.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        Ok(s)
    } else {
        Err(anyhow::anyhow!(
            "Identifier contains characters unsafe for shell usage: '{}'",
            s
        ))
    }
}

/// Reject strings that would corrupt TSV output or be unsafe in shell contexts.
/// This project only supports NCBI URLs; non-NCBI URLs are intentionally out of scope.
pub fn validate_no_control_chars(s: &str) -> anyhow::Result<&str> {
    if s.chars().any(|c| c.is_ascii_control()) {
        return Err(anyhow::anyhow!(
            "String contains control characters: '{}'",
            s
        ));
    }
    Ok(s)
}

/// Marker value for stdout output mode.
pub const STDOUT_MARKER: &str = "stdout";

/// Create a writer for the given output location.
/// When `outdir` equals `STDOUT_MARKER`, writes to stdout; otherwise writes
/// to `{outdir}/{subdir}/{outname}`. Returns an error instead of panicking
/// when the output file cannot be created.
pub fn open_writer(
    outdir: &str,
    subdir: &str,
    outname: &str,
) -> anyhow::Result<Box<dyn std::io::Write>> {
    if outdir == STDOUT_MARKER {
        crate::libs::io::writer("stdout")
    } else {
        crate::libs::io::writer(&format!("{}/{}/{}", outdir, subdir, outname))
    }
}

/// Retrieve the `outdir` string from a Tera context.
fn get_outdir(context: &Context) -> anyhow::Result<&str> {
    context
        .get("outdir")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'outdir' in template context"))
}

/// Write a two-column `species.tsv` (`key<TAB>species`) from a single map in
/// the context. Used by the Count and Protein output stages.
fn write_species_tsv(
    context: &Context,
    subdir: &str,
    map_key: &str,
) -> anyhow::Result<()> {
    let outname = "species.tsv";
    eprintln!("Create {}/{}", subdir, outname);

    let outdir = get_outdir(context)?;
    let map = context
        .get(map_key)
        .and_then(|v| v.as_object())
        .ok_or_else(|| anyhow::anyhow!("Missing '{}' in template context", map_key))?;

    let mut writer = open_writer(outdir, subdir, outname)?;
    for (key, value) in map {
        let species = value.as_str().ok_or_else(|| {
            anyhow::anyhow!("'{}' value for '{}' is not a string", map_key, key)
        })?;
        writeln!(writer, "{}\t{}", key, species)?;
    }
    writer.flush()?;
    Ok(())
}

/// Render a single shell script from a Tera template.
///
/// `tera` must already contain the shared `"header"` template; each call
/// registers `template_content` as `"t"` and renders it. Reusing a single
/// `Tera` instance avoids re-parsing the header on every call.
pub fn render_shell_script(
    tera: &mut Tera,
    context: &Context,
    template_content: &str,
    subdir: &str,
    outname: &str,
) -> anyhow::Result<()> {
    eprintln!("Create {}/{}", subdir, outname);

    let outdir = get_outdir(context)?;

    let mut writer = open_writer(outdir, subdir, outname)?;

    tera.add_raw_template("t", template_content)?;
    let rendered = tera.render("t", context)?;
    writer.write_all(rendered.as_ref())?;
    writer.flush()?;

    Ok(())
}

/// Generate ASSEMBLY/url.tsv and url_rsync.tsv.
pub fn gen_ass_data(context: &Context) -> anyhow::Result<()> {
    let outname = "url.tsv";
    let outname_rsync = "url_rsync.tsv";
    eprintln!("Create ASSEMBLY/{}", outname);
    eprintln!("Create ASSEMBLY/{}", outname_rsync);

    let outdir = get_outdir(context)?;
    let ass_url_of = context
        .get("ass_url_of")
        .and_then(|v| v.as_object())
        .ok_or_else(|| anyhow::anyhow!("Missing 'ass_url_of' in template context"))?;
    let ass_species_of = context
        .get("ass_species_of")
        .and_then(|v| v.as_object())
        .ok_or_else(|| {
            anyhow::anyhow!("Missing 'ass_species_of' in template context")
        })?;

    // Collect (key, url, species) once so both url.tsv and url_rsync.tsv share
    // the same extraction/error-handling path.
    let mut rows: Vec<(&String, String, String)> = Vec::new();
    for (key, value) in ass_url_of {
        let url = value.as_str().ok_or_else(|| {
            anyhow::anyhow!("ass_url_of value for '{}' is not a string", key)
        })?;
        let species = ass_species_of
            .get(key)
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "ass_species_of value for '{}' is missing or not a string",
                    key
                )
            })?;

        rows.push((key, url.to_string(), species.to_string()));
    }

    let mut writer = open_writer(outdir, "ASSEMBLY", outname)?;
    for (key, url, species) in &rows {
        writeln!(writer, "{}\t{}\t{}", key, url, species)?;
    }

    // Flush url.tsv before creating the second writer so buffered data is not
    // silently lost (BufWriter swallows flush errors on drop) if the next
    // open_writer call fails.
    writer.flush()?;

    let mut writer_rsync = open_writer(outdir, "ASSEMBLY", outname_rsync)?;
    for (key, url, species) in &rows {
        let rsync = RE_URL.replace(url, "ftp.ncbi.nlm.nih.gov::");
        writeln!(writer_rsync, "{}\t{}\t{}", key, rsync, species)?;
    }
    writer_rsync.flush()?;

    Ok(())
}

/// Generate BioSample/sample.tsv.
pub fn gen_bs_data(context: &Context) -> anyhow::Result<()> {
    let outname = "sample.tsv";
    eprintln!("Create BioSample/{}", outname);

    let outdir = get_outdir(context)?;
    let bs_name_of = context
        .get("bs_name_of")
        .and_then(|v| v.as_object())
        .ok_or_else(|| anyhow::anyhow!("Missing 'bs_name_of' in template context"))?;
    let bs_species_of = context
        .get("bs_species_of")
        .and_then(|v| v.as_object())
        .ok_or_else(|| anyhow::anyhow!("Missing 'bs_species_of' in template context"))?;

    let mut writer = open_writer(outdir, "BioSample", outname)?;

    for (key, value) in bs_name_of {
        let name = value.as_str().ok_or_else(|| {
            anyhow::anyhow!("bs_name_of value for '{}' is not a string", key)
        })?;
        let species =
            bs_species_of
                .get(key)
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "bs_species_of value for '{}' is missing or not a string",
                        key
                    )
                })?;

        writeln!(writer, "{}\t{}\t{}", key, name, species)?;
    }
    writer.flush()?;

    Ok(())
}

/// Generate MinHash/species.tsv.
pub fn gen_mh_data(context: &Context) -> anyhow::Result<()> {
    let outname = "species.tsv";
    eprintln!("Create MinHash/{}", outname);

    let outdir = get_outdir(context)?;
    let mh_species_of = context
        .get("mh_species_of")
        .and_then(|v| v.as_object())
        .ok_or_else(|| anyhow::anyhow!("Missing 'mh_species_of' in template context"))?;
    let mh_level_of = context
        .get("mh_level_of")
        .and_then(|v| v.as_object())
        .ok_or_else(|| anyhow::anyhow!("Missing 'mh_level_of' in template context"))?;

    let mut writer = open_writer(outdir, "MinHash", outname)?;

    for (key, value) in mh_species_of {
        let species = value.as_str().ok_or_else(|| {
            anyhow::anyhow!("mh_species_of value for '{}' is not a string", key)
        })?;
        let level = mh_level_of
            .get(key)
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "mh_level_of value for '{}' is missing or not a string",
                    key
                )
            })?;

        writeln!(writer, "{}\t{}\t{}", key, species, level)?;
    }
    writer.flush()?;

    Ok(())
}

/// Generate Count/species.tsv.
pub fn gen_count_data(context: &Context) -> anyhow::Result<()> {
    write_species_tsv(context, "Count", "count_species_of")
}

/// Generate Protein/species.tsv.
pub fn gen_pro_data(context: &Context) -> anyhow::Result<()> {
    write_species_tsv(context, "Protein", "pro_species_of")
}
