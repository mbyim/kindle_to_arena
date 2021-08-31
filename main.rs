use::std::fs;
use::std::env;
use::std::collections::HashMap;
use::reqwest;
use serde_json::Value;
use::chrono::{DateTime, Utc};
use chrono::prelude::*;

fn main() {
    static TOKEN: &str = "==========";
    let args: Vec<String> = env::args().collect();
    let file = &args[1];
    let mut load_arg = "only some"; 
    if args.len() > 2 {
        load_arg = &args[2]; //why did program hang instead of just panicing? idk but works now
        }
    println!("load arg: {}", load_arg);
    println!("file arg is: {}", file);

    //get last added-to-at time/date
    let last_added_at = get_last_added().expect("fetching date failed for some reason");
    println!("{:?}", last_added_at);

    parse_kindle(file, load_arg, TOKEN, last_added_at);

    println!("Done");
}

fn parse_kindle(filename: &str, load_arg: &str, token: &str, last_added_at: DateTime<Utc>) { //post to arena w/in this function or make this function return a list/vec of note structs?
    let contents = fs::read_to_string(filename).expect("something went wrong reading the file");
    //split
    let mut notes: Vec<&str> = contents.split(token).collect();
    notes.retain(|&i| i.contains("- Your Bookmark ") == false); //filter out bookmarks
    println!("{}", notes[0]);
    
    let client = reqwest::blocking::Client::new();
    for note in notes.iter().rev() {
        //parsed is Note struct
        let _parsed = match parse_note(note) {
                        Ok(x) => x,
                        _ => continue,
                        };

        let block = _parsed.construct_block_text();
        //marx real non-modified date: Added on Saturday, June 6, 2020 6:15:21 PM;
        println!("parsed time : {:?}", _parsed.time);
        println!("last added at time : {:?}", last_added_at);
        if load_arg == "all_time" {
            println!("all time load!");
            post_to_arena(&client, &block); //&client
        }
        else if _parsed.time > last_added_at {
            println!("calling post to arena func");
            post_to_arena(&client, &block); //&client
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
        let textblock = format!("{}\n{}\n\n{}\n{}", notetype, self.content, self.book_title, self.location); //weird - notetype var is not working properly?
        textblock
    }
}

fn parse_note(note: &str)-> Result<Note, &str> { //might need to change to result due to datetime_from_str, might be worth splitting it out into diff function
    //split note into lines by newline
    let lines: Vec<&str> = note.trim_start_matches("\r\n").split("\n").collect();

    //if this is blank then return Result<Error> or whatever
    if lines == vec![""] {
        return Err("error");
    };

    //if contains "- Your Note on page" then NoteType Annot else Note
    let notetype = match lines[1].contains("- Your Note on page") {
                    true => NoteType::Annot,
                    false => NoteType::Note,
                    };

    let booktitle = lines[0].to_string().replace("\r", "").replace("'\'", "");
    println!("BOOKTITLE: {:?}", booktitle);

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

fn get_last_added() -> Result<DateTime<Utc>, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let resp = client.get("https://api.are.na/v2/channels/all-kindle-highlights-notes")
                        .bearer_auth("")
                        .send()?; //not sure why i can't use json here
    let data: Value = serde_json::from_str(&resp.text()?).unwrap();
    let date = data["added_to_at"].as_str();
    let date_str = match date { 
                        None => "2020-06-21T23:09:50.164Z",  
                        Some(ref x) => x}; 
    println!("{:?}", date_str);
    let parsed_date = Utc.datetime_from_str(date_str,"%Y-%m-%dT%H:%M:%S%.3fZ"); 
    Ok(parsed_date?)
    
}


//app access token: a75bc0424d821090d622995c1873561d835028755e612ba26c057ffa637e35bf
//personal access token: 8626b72f9df98b7b4cf9aa20b6b52793a69b2d1d29409314dc4e371be9ff1f01
fn post_to_arena(client: &reqwest::blocking::Client, block: &str) { //&requwest
    //construct call
    let url = "https://api.are.na/v2/channels/all-kindle-highlights-notes/blocks";
    let mut params = HashMap::new();
    println!("{:?}", block.to_string());
    params.insert("content", block);//block
    //make call
    let postcall = client.post(url)
                .bearer_auth("")
                .form(&params)
                .send();
    //println!("{:?}", postcall.status());
    println!("sent request")
} 
