#[macro_use]
extern crate simple_log;

extern crate clap;
use std::{collections::HashMap, sync::mpsc::channel, thread};

use anyhow::{bail, Context, Result};
use clap::{App, Arg, SubCommand};

use simple_log::LogConfigBuilder;

fn main() {
    let matches = App::new("NBundle")
        .version("1.0")
        .author("MicroBlock")
        .about("Bundle everything to one javascript file.")
        .arg(
            Arg::with_name("dir")
                .short('d')
                .long("dir")
                .value_name("Dir")
                .help("Sets the monitoring dir.")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .short('o')
                .long("output")
                .value_name("Output")
                .help("Sets the output file.")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("main")
                .short('m')
                .long("main")
                .value_name("Main")
                .help("Sets the main file.")
                .takes_value(true),
        )
        .get_matches();

    let config = LogConfigBuilder::builder()
        .path("./nbundle_log.log")
        .size(1 * 100)
        .roll_count(10)
        .time_format("%Y-%m-%d %H:%M:%S.%f") //E.g:%H:%M:%S.%f
        .level("debug")
        .output_file()
        .output_console()
        .build();

    simple_log::new(config).unwrap();

    let dir: String = matches.get_one::<String>("dir").unwrap().clone();
    let output: String = matches.get_one::<String>("output").unwrap().clone();
    let main: String = matches
        .get_one::<String>("main")
        .unwrap_or(&"main.js".to_string())
        .clone();

    use notify::{watcher, RecommendedWatcher, RecursiveMode, Result, Watcher};
    use std::time::Duration;

    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();
    watcher
        .watch(dir.clone(), RecursiveMode::Recursive)
        .unwrap();
    info!("Watching dir {}", dir);
    loop {
        match rx.recv() {
            Ok(event) => {
                let result = build(&main, &dir, &mut HashMap::new());
                match result {
                    Ok(result) => match std::fs::write(&output, result) {
                        Ok(_) => {
                            info!("Successfully bundled");
                        }
                        Err(err) => {
                            error!("Failed to bundle!\nFailed to write the file.\n{}", err)
                        }
                    },
                    Err(err) => {
                        error!("Failed to compile!\n{}", err);
                    }
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}

fn build(file: &String, dir: &String, last_builds: &mut HashMap<String, String>) -> Result<String> {
    use regex::Regex;
    use serde_json::Value;
    use uuid::Uuid;

    let build_id = Uuid::new_v4();

    let mut data_json: Value = serde_json::from_str("{}")?;

    let path = format!("{}/{}", dir, file);
    if let Some(result) = last_builds.get(&path) {
        return Ok(result.clone());
    }

    let content = std::fs::read_to_string(&path)?;
    let mut content_processed = content.clone();
    let mut flag_used_nbundle_build_data=false;

    // String match
    let re = Regex::new(r#"[",',`]#([\S!#,\s]*?)#[",',`]"#)?;

    for capt in re.captures_iter(&content) {
        let replacer = capt
            .get(0)
            .context(format!("Get pattern error at file {}", &file))?
            .as_str();
        let pattern = capt
            .get(1)
            .context(format!("Get pattern error at file {}", &file))?
            .as_str();
        let result = parse_pattern(&pattern.to_string(), dir, &path, last_builds)?;
        if file.ends_with(".js") {
            let key = format!("nbundle-string-match-<{}>", Uuid::new_v4());
            flag_used_nbundle_build_data=true;
            data_json[&key] = serde_json::Value::String(result);

            content_processed = content_processed.replace(
                replacer,
                &format!("window['nbundle-build-{}']['{}']", build_id, &key),
            );
        } else {
            content_processed = content_processed.replace(replacer, &format!("\"{}\"", result));
        }
    }

    // Comments match
    let re = Regex::new(r#"/\*#([\S!#,\s]*?)#\*/"#)?;

    for capt in re.captures_iter(&content) {
        let replacer = capt
            .get(0)
            .context(format!("Get pattern error at file {}", &file))?
            .as_str();
        let pattern = capt
            .get(1)
            .context(format!("Get pattern error at file {}", &file))?
            .as_str();
        let result = parse_pattern(&pattern.to_string(), dir, &path, last_builds)?;
        content_processed = content_processed.replace(replacer, &result);
    }

    // // Normal match
    // let re = Regex::new(r#"#([\S!#,\s]*?)#"#)?;

    // for capt in re.captures_iter(&content) {
    //     let replacer = capt
    //         .get(0)
    //         .context(format!("Get pattern error at file {}", &file))?
    //         .as_str();
    //     let pattern = capt
    //         .get(1)
    //         .context(format!("Get pattern error at file {}", &file))?
    //         .as_str();
    //     let result = parse_pattern(&pattern.to_string(), dir, &path, last_builds)?;
    //     content_processed = content_processed.replace(replacer, &format!("\"{}\"", result));
    // }

    if file.ends_with(".js")&&flag_used_nbundle_build_data {
        content_processed = format!(
            "window['nbundle-build-{}']={}; // NBundle Datas\n\n{}",
            build_id,
            data_json.to_string(),
            content_processed
        );
    }

    Ok(content_processed)
}

fn parse_pattern(
    pattern: &String,
    dir: &String,
    file: &String,
    last_builds: &mut HashMap<String, String>,
) -> Result<String> {
    let mut patterns = pattern.split(" ");
    let command = patterns
        .next()
        .context(format!("Get pattern error at file {}", &file))?;
    match command {
        "require" => {
            let file=&patterns
            .next()
            .context(format!("Get pattern error at file {}", &file))?
            .to_string();
            let result = build(
                file,
                dir,
                last_builds,
            )?;
            if file.ends_with(".js") {
                Ok(format!("(function(){{{}}})()", result))
            } else {
                Ok(format!("{}", result))
            }
        }
        "raw_require" => {
            let result = build(
                &patterns
                    .next()
                    .context(format!("Get pattern error at file {}", &file))?
                    .to_string(),
                dir,
                last_builds,
            )?;
            Ok(format!("{}", result))
        }
        _ => bail!(format!("Unknown pattern {} in file {}", command, file)),
    }
}
