use clap::*;
use indexmap::IndexMap;
use std::collections::HashMap;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("nrps")
        .about("NRPS structure diagram")
        .after_help(
            r###"
* Input file is a tab-separated file
    * First column: Domain type (A, C, E, CE, T, Te, R, M)
    * Second column: Text (optional, amino acid/name)
    * Third column: Color (optional)

* Colors
    * black: RGB(26,25,25)
    * grey: RGB(129,130,132)
    * red: RGB(188,36,46)
    * brown: RGB(121,37,0)
    * green: RGB(32,128,108)
    * purple: RGB(160,90,150)
    * blue: RGB(0,103,149)

* Examples
    nwr nrps input.tsv -o output.tex

    nwr nrps input.tsv |
        tectonic - &&
        mv texput.pdf nrps.pdf

"###,
        )
        .arg(
            Arg::new("infile")
                .required(true)
                .num_args(1)
                .index(1)
                .help("Input filename. [stdin] for standard input"),
        )
        .arg(
            Arg::new("outfile")
                .long("outfile")
                .short('o')
                .num_args(1)
                .default_value("stdout")
                .help("Output filename. [stdout] for screen"),
        )
        .arg(
            Arg::new("legend")
                .long("legend")
                .action(ArgAction::SetTrue)
                .help("Include legend in the output"),
        )
        .arg(
            Arg::new("color")
                .long("color")
                .short('c')
                .num_args(1)
                .default_value("grey")
                .help("Default color for modules"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    //----------------------------
    // Args
    //----------------------------
    let infile = args.get_one::<String>("infile").unwrap();
    let default_color = args.get_one::<String>("color").unwrap();
    let is_legend = args.get_flag("legend");

    //----------------------------
    // Read TSV file
    //----------------------------
    let mut modules: IndexMap<String, Vec<HashMap<String, String>>> = IndexMap::new();
    let mut current_module = String::from("");
    let mut current_color = default_color.clone();
    let mut module_count = 1;

    // module info
    let mut module_info: IndexMap<String, HashMap<String, String>> = IndexMap::new();
    let mut prev_module = String::from("origin");

    let content = std::fs::read_to_string(infile)?;
    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let fields: Vec<&str> = line.split('\t').collect();
        if fields[0] == "Module" {
            // Get module name from second column or generate default name
            current_module = if fields.len() > 1 && !fields[1].is_empty() {
                fields[1].to_string()
            } else {
                format!("M{}", module_count)
            };
            module_count += 1;

            // Get color from third column or use default
            current_color = if fields.len() > 2 {
                fields[2].to_string()
            } else {
                default_color.clone()
            };

            // Initialize new module vector and info
            modules.insert(current_module.clone(), Vec::new());
            let info = HashMap::from([
                ("id".to_string(), current_module.clone()),
                ("color".to_string(), current_color.clone()),
                ("prev".to_string(), prev_module.clone()),
            ]);
            module_info.insert(current_module.clone(), info);
            prev_module = current_module.clone();
            continue;
        }

        let domain_type = fields[0].to_string();
        let text = if fields.len() > 1 {
            let raw_text = fields[1].to_string();
            if raw_text.starts_with("D-") || raw_text.starts_with("L-") {
                let (prefix, rest) = raw_text.split_at(2);
                format!("{{\\scriptsize {}}}{}", prefix, rest)
            } else {
                raw_text
            }
        } else {
            String::new()
        };
        let color = if fields.len() > 2 {
            fields[2].to_string()
        } else {
            current_color.clone()
        };

        let (dx_before, dx_after) = match domain_type.as_str() {
            "A" => (0.4, 0.4),
            "C" | "E" | "CE" | "M" => (0.4, 0.4),
            "T" => (0.2, 0.2),
            "Te" | "R" => (0.3, 0.3),
            _ => unreachable!(),
        };

        let domain_id = if let Some(domains) = modules.get(&current_module) {
            format!("{}-{}", current_module, domains.len() + 1)
        } else {
            format!("{}-1", current_module)
        };

        let pos = if let Some(domains) = modules.get(&current_module) {
            if domains.is_empty() {
                0.0
            } else {
                let last_domain = domains.last().unwrap();
                let last_pos: f64 = last_domain.get("pos").unwrap().parse().unwrap();
                let last_dx_after: f64 =
                    last_domain.get("dx_after").unwrap().parse().unwrap();
                format!("{:.1}", last_pos + last_dx_after + dx_before)
                    .parse()
                    .unwrap()
            }
        } else {
            0.0
        };

        let domain = HashMap::from([
            ("type".to_string(), domain_type),
            ("text".to_string(), text),
            ("color".to_string(), color),
            ("dx_before".to_string(), dx_before.to_string()),
            ("dx_after".to_string(), dx_after.to_string()),
            ("id".to_string(), domain_id),
            ("pos".to_string(), pos.to_string()),
        ]);

        if let Some(domains) = modules.get_mut(&current_module) {
            domains.push(domain);
        }
    }

    // eprintln!("modules = {:#?}", modules);
    // eprintln!("module_info = {:#?}", module_info);

    // Generate all modules
    let mut all_tex = String::new();
    for (module_name, domains) in &modules {
        let info = module_info.get(module_name).unwrap();
        let module_tex = gen_module(info, domains);
        all_tex.push_str(&module_tex);
        all_tex.push('\n');
    }

    //----------------------------
    // Context
    //----------------------------
    let mut context = tera::Context::new();

    context.insert("outfile", args.get_one::<String>("outfile").unwrap());
    context.insert("all_tex", &all_tex);
    context.insert("is_legend", &is_legend);
    context.insert("default_color", &default_color);

    gen_nrps(&context)?;

    Ok(())
}

fn gen_module(
    info: &HashMap<String, String>,
    domains: &Vec<HashMap<String, String>>,
) -> String {
    let mut context = tera::Context::new();
    context.insert("info", &info);
    context.insert("domains", &domains);
    context.insert("last_domain", domains.last().unwrap());
    context.insert("first_domain", domains.first().unwrap());

    let template = r###"
    \begin{scope}[shift={([shift={({{ first_domain.dx_before }}cm,0)}]{{ info.prev }}.east)}]
{% for domain in domains -%}
        \node[{{ domain.type }}, {{ domain.color }}] ({{ domain.id }}) at ({{ domain.pos }}cm,0) {};
{% if domain.text != "" -%}
        \node[text=white,anchor=center,align=left] at ({{ domain.id }}) { {{ domain.text }}};
{% endif -%}
{% endfor -%}
        \begin{scope}[on background layer]
            \draw[{{ first_domain.color }}, line width=0.5mm, yshift=-1cm]
                let \p1 = ({{ first_domain.id }}), \p2 = ({{ last_domain.id }}) in
                (\x1,0) -- (\x2,0)
                node[midway, below, text={{ first_domain.color }}] { {{ info.id }} };
            \draw[{{ first_domain.color }}, line width=2mm]
                let \p1 = ({{ last_domain.id }}) in
                (-{{ first_domain.dx_before }}cm,0) -- (\x1 + {{ last_domain.dx_after }}cm,0)
                coordinate ({{ info.id }});
        \end{scope}
    \end{scope}"###;

    let mut tera = tera::Tera::default();
    tera.add_raw_templates(vec![("t", template)]).unwrap();

    tera.render("t", &context).unwrap()
}

fn gen_nrps(context: &tera::Context) -> anyhow::Result<()> {
    let outfile = context.get("outfile").unwrap().as_str().unwrap();
    let all_tex = context.get("all_tex").unwrap().as_str().unwrap();
    let mut writer = intspan::writer(outfile);

    static FILE_TEMPLATE: &str = include_str!("../../../doc/nrps.tex");
    let mut template = FILE_TEMPLATE.to_string();

    {
        // Section color
        let default_color = context.get("default_color").unwrap().as_str().unwrap();
        let color_section = format!(
            r###"%
        draw={},
        fill={},
        text=white
        "###,
            default_color, default_color
        );

        let begin = template.find("%COLOR_BEGIN%").unwrap();
        let end = template.find("%COLOR_END%").unwrap();
        template.replace_range(begin..end, &color_section);
    }

    {
        // Section module
        let begin = template.find("%MODULE_BEGIN%").unwrap();
        let end = template.find("%MODULE_END%").unwrap();
        template.replace_range(begin..end, all_tex);
    }

    {
        // Section legend
        let is_legend = context.get("is_legend").unwrap().as_bool().unwrap();
        let begin = template.find("%LEGEND_BEGIN%").unwrap();
        let end = template.find("%LEGEND_END%").unwrap();
        if !is_legend {
            template.replace_range(begin..end, "");
        }
    }

    let mut tera = tera::Tera::default();
    tera.add_raw_templates(vec![("t", template)])?;

    let rendered = tera.render("t", context)?;
    writer.write_all(rendered.as_ref())?;

    Ok(())
}
