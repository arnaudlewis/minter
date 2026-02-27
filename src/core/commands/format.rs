use crate::core::content;

/// Display the DSL grammar reference for FR or NFR spec types.
pub fn run_format(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("missing format type. Valid types: spec, nfr");
        return 1;
    }
    match args[0].as_str() {
        "spec" => {
            println!("{}", content::fr_grammar());
            0
        }
        "nfr" => {
            println!("{}", content::nfr_grammar());
            0
        }
        other => {
            eprintln!("unknown format type: {other}. Valid types: spec, nfr");
            1
        }
    }
}
