use crate::core::content;
use crate::model::VALID_NFR_CATEGORIES;

/// Generate a skeleton .spec file for FR or NFR types.
pub fn run_scaffold(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("missing scaffold type. Valid types: spec, nfr");
        return 1;
    }
    match args[0].as_str() {
        "spec" => {
            print!("{}", content::fr_scaffold());
            0
        }
        "nfr" => {
            if args.len() < 2 {
                eprintln!(
                    "missing category for nfr scaffold. Valid categories: performance, reliability, security, observability, scalability, cost, operability"
                );
                return 1;
            }
            let category = &args[1];
            if !VALID_NFR_CATEGORIES.contains(&category.as_str()) {
                eprintln!(
                    "unknown nfr category: {category}. Valid categories: performance, reliability, security, observability, scalability, cost, operability"
                );
                return 1;
            }
            print!("{}", content::nfr_scaffold(category));
            0
        }
        other => {
            eprintln!("unknown scaffold type: {other}. Valid types: spec, nfr");
            1
        }
    }
}
