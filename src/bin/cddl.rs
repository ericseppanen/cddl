#[macro_use]
extern crate clap;

use cddl::{
  compile_cddl_from_str, parser, validate_cbor_from_slice, validate_cbor_named,
  validate_json_from_str,
};
use clap::{App, AppSettings, SubCommand};
use crossterm::{Color, Colored};
use serde_json;
use std::{error::Error, fs};

fn main() -> Result<(), Box<dyn Error>> {
  let app = App::new("cddl")
                    .version(crate_version!())
                    .author(crate_authors!())
                    .about("Tool for verifying conformance of CDDL definitions against RFC 8610 and for validating JSON documents")
                    .setting(AppSettings::SubcommandRequiredElseHelp)
                    .subcommand(SubCommand::with_name("dump-cddl")
                                .about("dump CDDL AST")
                                .arg_from_usage("-c --cddl=<FILE> 'CDDL input file'"))
                    .subcommand(SubCommand::with_name("compile-cddl")
                                .about("compiles CDDL against RFC 8610")
                                .arg_from_usage("-c --cddl=<FILE> 'CDDL input file'"))
                    .subcommand(SubCommand::with_name("compile-json")
                                .about("compiles JSON")
                                .arg_from_usage("-j --json=<FILE> 'JSON input file'"))
                    .subcommand(SubCommand::with_name("validate")
                                .about("validate JSON against CDDL definition")
                                .arg_from_usage("-c --cddl=<FILE> 'CDDL input file'")
                                .arg_from_usage("-j --json=<FILE> 'JSON input file"))
                    .subcommand(SubCommand::with_name("validate-cbor")
                                .about("validate CBOR against CDDL definition")
                                .arg_from_usage("-c --cddl=<FILE> 'CDDL input file'")
                                .arg_from_usage("--typename [TYPENAME] 'typename to validate")
                                .arg_from_usage("-b --cbor=<FILE> 'CBOR input file"));

  let matches = app.get_matches();

  if let Some(matches) = matches.subcommand_matches("dump-cddl") {
    let cddl_filename = matches.value_of("cddl").unwrap();
    let cddl_str = fs::read_to_string(cddl_filename)?;
    let ast = parser::cddl_from_str(&cddl_str)
      .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    println!("{:#?}", ast);
    return Ok(());
  }

  if let Some(matches) = matches.subcommand_matches("compile-cddl") {
    if let Some(c) = matches.value_of("cddl") {
      match compile_cddl_from_str(&fs::read_to_string(c)?) {
        Ok(()) => {
          println!("{}{} is conformant", Colored::Fg(Color::Green), c);
        }
        Err(e) => {
          eprintln!("{}{} is not conformant. {}", Colored::Fg(Color::Red), c, e);
        }
      }

      return Ok(());
    }
  }

  if let Some(matches) = matches.subcommand_matches("compile-json") {
    if let Some(c) = matches.value_of("json") {
      let file = std::fs::File::open(c)?;
      let reader = std::io::BufReader::new(file);
      let _: serde_json::Value = serde_json::from_reader(reader)?;

      return Ok(());
    }
  }

  if let Some(matches) = matches.subcommand_matches("validate") {
    if let Some(cddl) = matches.value_of("cddl") {
      if let Some(json) = matches.value_of("json") {
        match validate_json_from_str(&fs::read_to_string(cddl)?, &fs::read_to_string(json)?) {
          Ok(()) => {
            println!("{}Validation successful", Colored::Fg(Color::Green));
          }
          Err(e) => {
            eprintln!("{}Validation failed. {}", Colored::Fg(Color::Red), e);
          }
        }

        return Ok(());
      }
    }
  }

  if let Some(matches) = matches.subcommand_matches("validate-cbor") {
    let cddl = matches.value_of("cddl").unwrap();
    let cddl = fs::read_to_string(cddl)?;
    let cbor = matches.value_of("cbor").unwrap();
    let cbor = &fs::read(cbor)?;
    let result = match matches.value_of("typename") {
      Some(typename) => validate_cbor_named(&cddl, &typename, &cbor),
      None => validate_cbor_from_slice(&cddl, &cbor),
    };
    match result {
      Ok(()) => {
        println!("{}Validation successful", Colored::Fg(Color::Green));
      }
      Err(e) => {
        eprintln!("{}Validation failed. {}", Colored::Fg(Color::Red), e);
      }
    }
  }

  Ok(())
}
