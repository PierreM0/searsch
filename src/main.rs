#![allow(clippy::cast_precision_loss)]
use priority_queue::PriorityQueue;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{read_dir, File, ReadDir},
    io::{BufReader, Read, Write},
    path::Path,
};

#[derive(Serialize, Deserialize, Clone)]
struct Document {
    word_count: HashMap<String, usize>,
    lenght: i32,
}
impl Document {
    const fn new(word_count: HashMap<String, usize>, lenght: i32) -> Self {
        Self { word_count, lenght }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct Global {
    word_count: HashMap<String, Document>,
    lenght: i32,
}
impl Global {
    const fn new(word_count: HashMap<String, Document>, lenght: i32) -> Self {
        Self { word_count, lenght }
    }
}

fn words_time_by_documents(document: &str) -> Document {
    let mut word_count = HashMap::new();
    let document = document.split_whitespace();
    let mut counter = 0;
    for word in document {
        counter += 1;
        let word = word.to_lowercase();
        #[allow(clippy::map_entry)]
        if word_count.contains_key(&word) {
            if let Some(x) = word_count.get_mut(&word) {
                *x += 1;
            }
        } else {
            let _ = word_count.insert(word, 1);
        }
    }

    Document::new(word_count, counter)
}

fn scan_all_documents(dir: Option<ReadDir>, scanned_documents: &mut Global) -> Global {
    let Some(dir) = dir else {
        return scanned_documents.clone();
    };

    let mut counter = 0;
    for file in dir {
        counter += 1;

        let file_direntry = file.expect("File is a direntry");

        if file_direntry.file_type().expect("the file type").is_file() {
            let file_name = file_direntry
                .path()
                .to_str()
                .expect("file exists")
                .to_owned();
            let file = File::open(file_name.as_str())
                .expect("File must exists if it was given by a correct directory.");

            let mut buf_reader = BufReader::new(file);
            let mut file_content = String::new();
            buf_reader
                .read_to_string(&mut file_content)
                .expect("read_to_string goes well");

            if scanned_documents
                .word_count
                .contains_key(file_name.as_str())
            {
                eprintln!("file: {file_name} was already in the cache");
            } else {
                let doc = words_time_by_documents(&file_content);

                scanned_documents.word_count.insert(file_name, doc);
            }
        }
    }
    scanned_documents.lenght = counter;
    scanned_documents.clone()
}

fn print_help(_arg: &'static str) {
    eprint!("./searsch -d <directory> -s \"<search query>\"");
    std::process::exit(1);
}

fn main() {
    let mut args = std::env::args();
    args.next();

    let mut dir = String::new();
    let mut search = String::new();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-d" | "--directory" => args.next().map_or_else(|| print_help("-d"), |a| dir = a),
            "-s" | "--search" => args.next().map_or_else(|| print_help("-s"), |a| search = a),
            "-h" | "--help" => print_help(""),
            arg => {
                eprintln!("Uknown argument {arg}.");
                print_help("");
            }
        }
    }
    let mut search_query = Vec::<String>::new();
    search.split(' ').for_each(|e| {
        search_query.push(e.to_string());
    });
    search_in_directory(search_query, dir.as_str());
}

fn scan_path(path: &str) -> Global {
    let dir: Option<ReadDir> = if path.is_empty() {
        None
    } else {
        let directory_path = Path::new(path);
        Some(read_dir(directory_path).unwrap_or_else(|err| {
            eprintln!("Error: {err}");
            std::process::exit(1)
        }))
    };

    let data_path = "data.json";

    let file = File::open(data_path).unwrap_or_else(|_| {
        let _ = File::create(data_path);
        File::open(data_path).expect("No IO errors") // TODO err msg
    });

    let mut documents: Global = serde_json::from_reader(BufReader::new(file))
        .unwrap_or_else(|_| Global::new(HashMap::new(), 0));
    let scanned_documents = scan_all_documents(dir, &mut documents);
    let serialized = serde_json::to_string(&scanned_documents).unwrap_or_else(|_| String::new());

    let _ = File::create(data_path)
        .expect("No IO Errors") // TODO err msg
        .write_all(serialized.as_bytes());

    scanned_documents
}

fn search_in_directory(search: Vec<String>, path: &str) {
    let mut search_terms = Vec::<String>::new();

    for w in search {
        search_terms.push(w.to_lowercase());
    }

    let scanned_documents = scan_path(path);

    let mut priority_queue = PriorityQueue::<String, i32>::new();

    for (document_name, document) in &scanned_documents.word_count {
        let score = score(document, &search_terms, &scanned_documents);
        priority_queue.push(document_name.to_owned(), score);
    }

    for _ in 0..5 {
        println!("{:#?}", priority_queue.pop().unwrap_or(("Empty results.".to_string(), 0)).0);
    }
}

fn idf(word: &str, global_documents: &Global) -> f32 {
    let mut document_containing_word = 0;
    for document in global_documents.word_count.values() {
        if document.word_count.contains_key(word) {
            document_containing_word += 1;
        }
    }
    let numerator = (global_documents.lenght - document_containing_word) as f32 + 0.5;
    let denominator = document_containing_word as f32 + 0.5;

    (1f32 + (numerator / denominator)).log10()
}

fn score(document: &Document, query: &Vec<String>, global_documents: &Global) -> i32 {
    let k1 = 1.2;
    let b = 0.75; // magic numbers
    let zero: usize = 0;

    let mut score: f32 = 0.0;

    let mut d_on_avgdl = 0;
    for doc in global_documents.word_count.values() {
        d_on_avgdl += doc.lenght;
    }
    let d_on_avgdl = d_on_avgdl as f32 / global_documents.lenght as f32;

    for word in query {
        let freq_word_document = document.word_count.get(word).unwrap_or(&zero).to_owned();
        let idf = idf(word, global_documents);
        let numerator = freq_word_document as f32 * (k1 + 1.0);
        let denominator = freq_word_document as f32 + k1.mul_add(b * d_on_avgdl, 1. -b);
        score += idf * (numerator / denominator);
    }
    unsafe { (score * 1000.0).round().to_int_unchecked() } // TODO
}
