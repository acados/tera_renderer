use clap::Parser;
use tera::Tera;
use tera::Context;

use std::fs::File;
use std::io::Write;
use std::io;
use std::io::Read;
use std::process;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    template_glob: String,
    template_file: String,
    json_file: String,
    out_file: String
}

fn extract_key_with_prefix(msg: &str, prefix: &str) -> Option<String> {
    msg.split(prefix)
        .nth(1)
        .and_then(|s| s.split('`').next())
        .filter(|s| !s.is_empty())
        .map(String::from)
}

fn extract_missing_context_key(render_error: &tera::Error) -> Option<String> {
    let detailed_error = format!("{:#}", render_error);
    extract_key_with_prefix(&detailed_error, "Variable `")
        .or_else(|| extract_key_with_prefix(&detailed_error, "Field `"))
        .or_else(|| {
            let debug_error = format!("{:?}", render_error);
            extract_key_with_prefix(&debug_error, "Variable `")
                .or_else(|| extract_key_with_prefix(&debug_error, "Field `"))
        })
}

fn main() -> io::Result<()> {
    // read command line arguments
    let args = Args::parse();

    // relative glob to template file
    let template_glob = &args.template_glob;
    // template file path relative to 'template_glob'
    let template_file = &args.template_file; 
    // relative path json file
    let json_file = &args.json_file;
    // relative path to output file
    let out_file = &args.out_file;

    // open json file
    let mut file = File::open(json_file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // println!("template file: {}", template_file);
    // println!("json file: {}"    , json_file);
    // println!("out file: {}"     , out_file);
    // println!("{}", contents);
    
    // Parse the string of data into serde_json::Value.
    let v: serde_json::Value = serde_json::from_str(&contents).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData, format!("Invalid JSON context: {}", e))
    })?;
    // Convert serde_json::Value to tera::Context
    let ctx: Context = Context::from_serialize(&v).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to build template context from JSON: {}", e),
        )
    })?;

    let tera = match Tera::new(template_glob) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Template parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };

    match tera.render(template_file, &ctx) {
        Ok(s) => {
            let mut f_out = File::create(out_file).expect("Unable to create file");
            f_out.write_all(s.as_bytes())?;
        },
        Err(e) => {
            if let Some(missing_key) = extract_missing_context_key(&e) {
                eprintln!(
                    "Error: missing context value `{}` required by template `{}`.",
                    missing_key, template_file
                );
            }
            eprintln!("Render error: {}", e);
            process::exit(1);
        }
    };

    // println!("-> successfully rendered template!\n");
    Ok(())
}
