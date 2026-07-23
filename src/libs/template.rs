use regex::Regex;
use std::io::Write;
use std::sync::LazyLock;
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

/// Regex matching NCBI FTP/HTTP URLs for rsync conversion.
pub static RE_URL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?xi)(ftp|https?)://ftp\.ncbi\.nlm\.nih\.gov/").unwrap()
});

/// Validate that a string is safe to embed into generated shell scripts and
/// to use as a file or directory name.
///
/// Only ASCII alphanumeric characters, underscores, hyphens and dots are allowed.
/// A leading hyphen is rejected to prevent the value from being interpreted as a command-line flag.
pub fn validate_shell_safe(s: &str) -> anyhow::Result<&str> {
    if s.is_empty() {
        return Err(anyhow::anyhow!("Shell-safe identifier must not be empty"));
    }
    // Reject "." and ".." to prevent path traversal when used as file/directory names.
    if s == "." || s == ".." {
        return Err(anyhow::anyhow!(
            "Shell-safe identifier must not be '.' or '..': '{s}'"
        ));
    }
    // Reject leading hyphen so the value is not mistaken for a CLI flag.
    if s.starts_with('-') {
        return Err(anyhow::anyhow!(
            "Shell-safe identifier must not start with '-': '{s}'"
        ));
    }
    if s.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        Ok(s)
    } else {
        Err(anyhow::anyhow!(
            "Identifier contains characters unsafe for shell usage: '{s}'"
        ))
    }
}

/// Reject strings that would corrupt TSV output or be unsafe in shell contexts.
/// This project only supports NCBI URLs; non-NCBI URLs are intentionally out of scope.
pub fn validate_no_control_chars(s: &str) -> anyhow::Result<&str> {
    if s.chars().any(|c| c.is_ascii_control()) {
        return Err(anyhow::anyhow!("String contains control characters: '{s}'"));
    }
    Ok(s)
}

/// Marker value for stdout output mode.
pub const STDOUT_MARKER: &str = "stdout";

/// Create a writer for the given output location.
///
/// When `outdir` equals `STDOUT_MARKER`, writes to stdout; otherwise writes
/// to `{outdir}/{subdir}/{outname}`. Returns an error instead of panicking
/// when the output file cannot be created.
pub fn open_writer(
    outdir: &str,
    subdir: &str,
    outname: &str,
) -> anyhow::Result<crate::libs::io::Writer> {
    if outdir == STDOUT_MARKER {
        crate::libs::io::writer("stdout")
    } else {
        crate::libs::io::writer(&format!("{outdir}/{subdir}/{outname}"))
    }
}

/// Retrieve the `outdir` string from a Tera context.
fn get_outdir(context: &Context) -> anyhow::Result<&str> {
    context
        .get("outdir")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'outdir' in template context"))
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
    eprintln!("Create {subdir}/{outname}");

    let outdir = get_outdir(context)?;

    let mut writer = open_writer(outdir, subdir, outname)?;

    tera.add_raw_template("t", template_content)?;
    let rendered = tera.render("t", context)?;
    writer.write_all(rendered.as_ref())?;
    writer.flush()?;
    writer.finish()?;

    Ok(())
}
