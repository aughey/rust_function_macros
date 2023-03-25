use anyhow::Result;
use fn_parser::{file_parsing::parse_file, FunctionDefinition};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct FileContent {
    path: String,
    functions: Vec<FunctionDefinition>,
}

fn main() -> Result<()> {
    // files are all the command line arguments
    let files = std::env::args().skip(1);

    let all_content = files
        .map(|f| {
            let functions = parse_file(&f)?;
            let functions = functions.into_iter().collect::<Vec<_>>();

            let fc = FileContent { path: f, functions };
            Ok(fc)
        })
        .collect::<Result<Vec<_>>>()?;

    let json = serde_json::to_string(&all_content)?;

    println!("{}", json);

    Ok(())
}
