extern crate csv;

use std::process;
use encoding_rs::WINDOWS_1252;
use encoding_rs_io::DecodeReaderBytesBuilder;



use std::collections::HashMap;
use std::error::Error;
// use std::ffi::OsString;
//use std::fs::File;
// use std::process;
// use std::io::{self, BufRead, BufWriter};
use regex::Regex;
use std::path::{self, Path};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write, BufRead, BufReader, BufWriter};

fn process_file() -> Result<(), Box<dyn Error>> {
    let file_path = "./data/tfs.csv";
    let file = File::open(file_path)?;

    let out_path = "./data/processed_tfs.csv";
    
    let out_file = OpenOptions::new()
        .read(false)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&out_path)?;



        
//    let mut out_file = File::create(&out_path)?; 
    let mut writer = BufWriter::new(&out_file);
    //     let mut rdr = csv::ReaderBuilder::new()
// //        .delimiter(b'|')
//         .flexible(true)
//         .from_reader(file);

    let transcoded = DecodeReaderBytesBuilder::new()
        .encoding(Some(WINDOWS_1252))
        .build(file);
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'|')
        .from_reader(transcoded);

    // Read and process the headers saving each header in the 
    // HashMap so we can reference the column id
    let mut headers = HashMap::<String, usize>::new();
    let mut index = 0;
    let header_record = rdr.headers()?;

    for value in header_record {
        headers.insert(value.to_string(), index);
        index += 1;
    }

    // Write out the header fields
    for item in header_record.iter() {
        writer.write(&item.as_bytes())?;
        writer.write("|".as_bytes())?;
    }

    // Write out the additional 3 fields
    writer.write("Team".as_bytes())?;
    writer.write("|".as_bytes())?;
    writer.write("SprintYear".as_bytes())?;
    writer.write("|".as_bytes())?;
    writer.write("SprintNumber".as_bytes())?;
    writer.write("|".as_bytes())?;
    writer.write("\n".as_bytes())?;

    let iteration_path_index = match headers.get("IterationPath") {
        Some(i) => i,
        None => { panic!("Cannot find field 'IterationPath' in file"); &0},
    };

    let tags_index = match headers.get("Tags") {
        Some(i) => i,
        None => { panic!("Cannot find field 'Tags' in file"); &0},
    };
    
    let re = Regex::new(r"\\(?P<team>[^\\]*)\\[^\\]*(?P<year>\d{4})\s*-\s*(?P<number>\d{1,2})\s*$").unwrap();

    // Process each record and calculate the new field values then write them to the output file
    for result in rdr.records() {

        let record = result?;

        // Add a couple of columns on the end
        let mut team = "";
        let mut sprint_year = "";
        let mut sprint_number = "";

        if let Some(field_value) = record.get(*iteration_path_index) {
            // Extract the fields out of the iteration path and add them to this record
            if let Some(caps) = re.captures(field_value) {
                team = caps.name("team").unwrap().as_str();
                sprint_year = caps.name("year").unwrap().as_str();
                sprint_number = caps.name("number").unwrap().as_str();
            };
        };

        if let Some(tags) = record.get(*tags_index) {
            if tags.contains(",") {
                println!("{:?} contains , in tags field", record.get(0));
            }
        }

        for item in record.iter() {
            writer.write(&item.as_bytes())?;
            writer.write("|".as_bytes())?;
        }
    
        // Write out the additional 3 fields
        writer.write(team.as_bytes())?;
        writer.write("|".as_bytes())?;
        writer.write(sprint_year.as_bytes())?;
        writer.write("|".as_bytes())?;
        writer.write(sprint_number.as_bytes())?;
        writer.write("|".as_bytes())?;
        writer.write("\n".as_bytes())?;
   
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    process_file()?;

    Ok(())
}






fn process_file2() -> Result<(), Box<dyn Error>> {
    let file_path = "./data/tfs-pipe-delimitered.csv";
    let file = File::open(file_path)?;

    let lines = io::BufReader::new(file).lines();
    let mut first_row = true;

    let out_path = "./data/processed_tfs-pipe-delimitered.csv";
    let mut out_file = File::create(&out_path)?; 



    // let path = Path::new(&out_path);

    // let mut options = OpenOptions::new();
    // options.write(true);
    // let file: Result<File, Error> = options.open(path);

    // let file = match options.open(&path) {
    //     Ok(file) => file,
    //     Err(..) => panic!("File does not exist"),
    // };

    let mut writer = BufWriter::new(&out_file);

    let re = Regex::new(r"\\(?P<team>[^\\]*)\\[^\\]*(?P<year>\d{4})\s*-\s*(?P<number>\d{1,2})\s*$").unwrap();
    let mut headers = HashMap::<String, usize>::new();
    let mut iteration_path_index: usize = 0;

    for item in lines {
        let line = item.unwrap();
        let mut fields: Vec<&str> = line.split('|').collect();

        if first_row {
            first_row = false;
            let mut index = 0;
            for value in &fields {
                headers.insert(value.to_string(), index);
                index += 1;
            }
            println!("Headers={:?}", headers);

            // Add three new fields to the headers
            fields.push("Team");
            fields.push("Sprint Year");
            fields.push("Sprint Number");

            // File write
            for item in &fields {
                writer.write(&item.as_bytes())?;
                writer.write("|".as_bytes())?;
            }
            writer.write("\n".as_bytes())?;

            iteration_path_index = match headers.get(&"IterationPath".to_string()) {
                Some(i) => *i,
                None => { panic!("Cannot find field 'IterationPath' in file"); 0},
            };
        
        } else {
            if let Some(caps) = re.captures(&fields[iteration_path_index]) {
//                println!("fields[7]={}\ncaps={:?}", &fields[7], &caps);

                fields.push(caps.name("team").unwrap().as_str().clone());
                fields.push(caps.name("year").unwrap().as_str().clone());
                fields.push(caps.name("number").unwrap().as_str().clone());

            } else {
                fields.push("");
                fields.push("");
                fields.push("");
            }    
            for item in &fields {
                writer.write(&item.as_bytes())?;
                writer.write("|".as_bytes())?;
            }
            writer.write("\n".as_bytes())?;
        }
//        println!("fields[]={:?}", &fields);
    }

    Ok(())
}



fn process_file3() -> csv::Result<()> {
    let file = File::open("./data/tfs.csv")?;
    let transcoded = DecodeReaderBytesBuilder::new()
        .encoding(Some(WINDOWS_1252))
        .build(file);
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'|')
        .from_reader(transcoded);
    for result in rdr.records() {
        let r = result?;
        println!("{:?}", r);
    }
    Ok(())
}