/// Generate a skeleton .spec file for FR or NFR types.
pub fn run_scaffold(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("missing scaffold type. Valid types: fr, nfr");
        return 1;
    }
    match args[0].as_str() {
        "fr" => {
            print_fr_scaffold();
            0
        }
        "nfr" => {
            if args.len() < 2 {
                eprintln!("missing category for nfr scaffold. Valid categories: performance, reliability, security, observability, scalability, cost, operability");
                return 1;
            }
            let category = &args[1];
            let valid = [
                "performance",
                "reliability",
                "security",
                "observability",
                "scalability",
                "cost",
                "operability",
            ];
            if !valid.contains(&category.as_str()) {
                eprintln!(
                    "unknown nfr category: {category}. Valid categories: performance, reliability, security, observability, scalability, cost, operability"
                );
                return 1;
            }
            print_nfr_scaffold(category);
            0
        }
        other => {
            eprintln!("unknown scaffold type: {other}. Valid types: fr, nfr");
            1
        }
    }
}

fn print_fr_scaffold() {
    print!(
        "\
spec my-feature v0.1.0
title \"My Feature\"

description
  Describe what this feature does.

motivation
  Explain why this feature is needed.

behavior do-something [happy_path]
  \"The system does something successfully\"

  given
    The system is in a valid state

  when perform-action

  then returns result
    assert status == \"success\"
"
    );
}

fn print_nfr_scaffold(category: &str) {
    print!(
        "\
nfr {category} v0.1.0
title \"{title} Requirements\"

description
  Describe the {category} requirements.

motivation
  Explain why these {category} requirements matter.


constraint example-constraint [metric]
  \"Describe what this constraint measures\"

  metric \"The metric being measured\"
  threshold < 1s

  verification
    environment all
    benchmark \"Describe the benchmark procedure\"
    pass \"Describe what constitutes passing\"

  violation medium
  overridable yes
",
        category = category,
        title = capitalize(category),
    );
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}
