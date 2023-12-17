use clap::Parser;
use tera::Tera;
use tera::Context;

use std::fs::File;
use std::io::Write;
use std::io::Read;

#[derive(Parser)]
#[command(version)]
struct Args {
    template_glob: String,
    template_file: String,
    json_file: String,
    out_file: String
}

fn main() -> Result<(), tera::Error> {
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
    // Parse the string of data into serde_json::Value.
    let v: serde_json::Value = serde_json::from_str(&contents)?;
    // Convert serde_json::Value to tera::Context
    let ctx: Context = Context::from_serialize(&v)?;
    // load template
    let tera = Tera::new(template_glob)?;
    // render template
    let s = tera.render(template_file, &ctx)?;
    let mut f_out = File::create(out_file).expect("Unable to create file");
    f_out.write_all(s.as_bytes())?;
     println!("Successfully rendered template: {}", template_file);
    Ok(())
}
