use std::fs::File;
use std::io::{Read, Write};
use std::{thread, time};

use clap::Parser as ArgsParser;
use pulldown_cmark::{html, Parser as MarkdownParser};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Meta {
    title: String,

    #[allow(dead_code)]
    description: String,

    #[allow(dead_code)]
    #[serde(rename = "viewcount")]
    view_count: usize,

    #[allow(dead_code)]
    #[serde(rename = "createtime")]
    created_at: String,

    #[allow(dead_code)]
    #[serde(rename = "updatetime")]
    updated_at: String,
}

/// Automatically generate a HTML document out of a Markdown file served by Hedgedoc.
#[derive(ArgsParser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// URL of Hedgedoc note.
    #[arg(short, long)]
    url: String,

    /// Path to HTML template document.
    #[arg(short, long, default_value = "template.html")]
    template_path: String,

    /// Path to HTML output document.
    #[arg(short, long, default_value = "index.html")]
    output_path: String,

    /// Wait x seconds before checking for new update.
    #[arg(short, long, default_value_t = 60 * 10)]
    frequency: u64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut last_updated_at: String = String::new();
    let args = Args::parse();

    // Read template file
    let mut file = File::open(args.template_path)?;
    let mut html_template = String::new();
    file.read_to_string(&mut html_template)?;

    loop {
        // Get name and last update from meta-data
        let meta = reqwest::blocking::get(format!("{}/info", args.url))?.json::<Meta>()?;

        if meta.updated_at != last_updated_at {
            println!(
                "Detected update at {}, generate static website",
                &meta.updated_at
            );

            last_updated_at = meta.updated_at;

            let document = reqwest::blocking::get(format!("{}/download", args.url))?.text()?;

            // Parse markdown and convert it to HTML
            let parser = MarkdownParser::new(&document);
            let mut html_document = String::new();
            html::push_html(&mut html_document, parser);

            // Insert document into template
            let output = html_template
                .replace("{document}", &html_document)
                .replace("{title}", &meta.title);

            // Write HTML to file
            let mut file = File::create(&args.output_path)?;
            file.write_all(output.as_bytes())?;

            println!("Written HTML file to {}", args.output_path);
        }

        thread::sleep(time::Duration::from_secs(args.frequency));
    }
}
