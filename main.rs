use::std::fs;
use::std::env;
use::std::collections::HashMap;
use::reqwest;
use serde_json::{Value, Error};
use::chrono::{DateTime, Utc}; //, Utc, NaiveDate, NaiveDateTime, NaiveTime};
use::chrono::format::ParseError;
use chrono::prelude::*;
//https://rust-lang-nursery.github.io/rust-cookbook/datetime/parse.html#parse-string-into-datetime-struct

fn main() {
    static TOKEN: &str = "==========";
    let args: Vec<String> = env::args().collect();

    let file = &args[1];
    println!("file arg is: {}", file);

    //get last added-to-at time/date
    let last_added_at = get_last_added().expect("fetching date failed for some reason");
    println!("{:?}", last_added_at);

    parse_kindle(file, TOKEN, last_added_at);

    println!("Done");
}

fn parse_kindle(filename: &str, token: &str, last_added_at: DateTime<Utc>) { //post to arena w/in this function or make this function return a list/vec of note structs?
    let contents = fs::read_to_string(filename).expect("something went wrong reading the file");
    //println!("contents: {}", contents);

    //split
    let mut notes: Vec<&str> = contents.split(token).collect();
    notes.retain(|&i| i.contains("- Your Bookmark ") == false); //filter out bookmarks
    println!("{}", notes[0]);
    
    //i should do note parsing on a for loop and compare parsed date vs added-to-at in real time
    //instead of doing it all at once at the end (would be more resource friendly - less on the heap?)
    //just doing one block parsing and block constructing to test functions
    let client = reqwest::Client::new();
    for note in notes.iter().rev() {
        //parsed is Note struct
        let _parsed = match parse_note(note) {
                        Ok(x) => x,
                        _ => continue,
                        };

        let block = _parsed.construct_block_text();

        //this appears w/ the new lines visible? not sure if i need to convert
        //or if this will resolve nicely in the actual text block, will need to test
        //compare dates to see if should post or not
        println!("parsed time : {:?}", _parsed.time);
        println!("last added at time : {:?}", last_added_at);
        if _parsed.time > last_added_at {
            println!("calling post to arena func");
            post_to_arena(&client, &block);
        }
        else {
            println!("breaking for loop");
            break;
        }
        println!("--------------------------------------------------------");
    }
}

//create a struct and impl for note
#[derive(Debug)]
enum NoteType {
Note,
Annot,
}

struct Note {
    note_type: NoteType,
    book_title: String,
    time: DateTime<Utc>,
    location: String,
    content: String
}

impl Note {
    fn construct_block_text(&self) -> String {
        let notetype = match self.note_type {
                        NoteType::Annot => "(my note)\n",
                        NoteType::Note => "\n",
                        };
        //whats the best way to construct a string in this situation? .push_str(notetype)
        let textblock = format!("{}\n{}\n{}\n{}", notetype, self.content, self.book_title, self.location);
        //textblock.push(note.content).push("\n").push(note.book_title).push_str("\n").push(note.location);       
        textblock
    }
}

//need to make this Result bc if might return ""
fn parse_note(note: &str)-> Result<Note, &str> { //might need to change to result due to datetime_from_str, might be worth splitting it out into diff function
    println!("IN PARSE NOTE FUNC");
    //println!("{:?}", note);
    //split note into lines by newline
    let lines: Vec<&str> = note.trim_start_matches("\r\n").split("\n").collect();
    println!("{:?}", lines);

    //if contains "- Your Note on page" then NoteType Annot else Note
    let notetype = match lines[1].contains("- Your Note on page") {
                    true => NoteType::Annot,
                    false => NoteType::Note,
                    };
    //println!("{:?}", notetype);
    
    let booktitle = lines[0].to_string().replace("\r", "").replace("'\'", "");
    println!("BOOKTITLE: {:?}", booktitle);

    //if this is blank then return Result<Error> or whatever
    if booktitle == "" {
        return Err("error");
    };

    let dt = match Utc.datetime_from_str(lines[1].replace("\r", "").split(" | Added on ").collect::<Vec<&str>>()[1], "%A, %B %d, %Y %I:%M:%S %p") {
                    Ok(x) => x, 
                    _ => Utc::now() //on parse error just make the time right now
                        };
    println!("{:?}", dt);
    
    let note_location = lines[1].split(" on ").collect::<Vec<&str>>()[1].split(" | ").collect::<Vec<&str>>()[0].to_string(); //um not great?
    println!("{:?}", note_location);

    let note_content = lines[3..].join(" ").to_string().replace("\r", "").replace("'\'", "");
    println!("{:?}", note_content);

    //make and return struct
     Ok(Note {
            note_type: notetype,
            book_title: booktitle,
            time: dt,
            location: note_location,
            content: note_content,
            })
}

//return added-to-at coverted to utc so we can compare + parse against blocks
fn get_last_added() -> Result<DateTime<Utc>, Box<dyn std::error::Error>> {//Box<dyn std::error::Error>> { //anyhow::Error
    let client = reqwest::blocking::Client::new();
    let resp = client.get("https://api.are.na/v2/channels/all-kindle-highlights-notes")
                        .bearer_auth("8626b72f9df98b7b4cf9aa20b6b52793a69b2d1d29409314dc4e371be9ff1f01")
                        .send()?; //not sure why i can't use json here
    let data: Value = serde_json::from_str(&resp.text()?).unwrap();
    let date = data["added_to_at"].as_str();
    let date_str = match date { 
                        None => "2020-06-21T23:09:50.164Z",  
                        Some(ref x) => x}; 
    println!("{:?}", date_str);
    //println!("{:?}", date.as_str());
    //let parsed_date = DateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S%.3fZ"); 
    let parsed_date = Utc.datetime_from_str(date_str,"%Y-%m-%dT%H:%M:%S%.3fZ"); 
    //`%Y-%m-%dT%H:%M:%S%z`
    //Ok(Utc.ymd(2014, 11, 28).and_hms(12, 0, 9))
    //DateTime<Utc> 
    Ok(parsed_date?)
    
}


//arena auth info:
//app access token: a75bc0424d821090d622995c1873561d835028755e612ba26c057ffa637e35bf
//personal access token: 8626b72f9df98b7b4cf9aa20b6b52793a69b2d1d29409314dc4e371be9ff1f01
//?:content="test text block creation"
//use "added-to-at" field:
fn post_to_arena(client: &reqwest::Client, block: &str) {
    //construct call
    let url = "https://api.are.na/v2/channels/all-kindle-highlights-notes/blocks";
    let mut params = HashMap::new();
    params.insert("content", block);
    //make call
    let call = client.post(url).form(&params).send();
    println!("sent request")
}

