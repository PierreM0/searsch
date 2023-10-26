use priority_queue::PriorityQueue;
use std::{
    collections::HashMap,
    fs::{read_dir, File},
    io::{BufReader, Read},
    path::Path,
};

struct Document {
    word_count: HashMap<String, usize>,
    lenght: i32,
}
impl Document {
    fn new(word_count: HashMap<String, usize>, lenght: i32) -> Self {
        Self { word_count, lenght }
    }
}
struct Global {
    word_count: HashMap<String, Document>,
    lenght: i32,
}

impl Global {
    fn new(word_count: HashMap<String, Document>, lenght: i32) -> Self {
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
        if word_count.contains_key(&word) {
            let _ = word_count.insert(
                word.to_owned(),
                word_count.get(&word).expect("it exists") + 1,
            );
        } else {
            let _ = word_count.insert(word.to_owned(), 1);
        }
    }

    Document::new(word_count, counter)
}

fn main() {
    let binding = vec!["Plain-Text".to_owned()];
    let mut search_terms = Vec::<String>::new();
    for w in binding {
        search_terms.push(w.to_lowercase());
    }

    let directory_path = Path::new("./noboilerplate/scripts/");
    let dir = read_dir(directory_path).unwrap();
    let mut scanned_documents = Global::new(HashMap::new(), 0);

    let mut counter = 0;

    for file in dir {
        counter += 1;

        let file_direntry = file.expect("File is a direntry");

        if file_direntry.file_type().expect("the file type").is_file() {
            let file = File::open(file_direntry.path()).expect("it exists");

            let mut buf_reader = BufReader::new(file);
            let mut file_content = String::new();
            buf_reader
                .read_to_string(&mut file_content)
                .expect("read_to_string goes well");

            let doc = words_time_by_documents(&file_content);

            scanned_documents.word_count.insert(
                file_direntry
                    .path()
                    .to_str()
                    .expect("file exists")
                    .to_owned(),
                doc,
            );
        }
    }
    scanned_documents.lenght = counter;

    let mut priority_queue = PriorityQueue::<String, i32>::new();

    for (document_name, document) in &scanned_documents.word_count {
        let score = score(document, &search_terms, &scanned_documents);
        priority_queue.push(document_name.to_owned(), score);
    }

    for _ in 0..5 {
        println!("{:#?}", priority_queue.pop());
    }
}

fn idf(word: &str, global_documents: &Global) -> f32 {
    let mut document_containing_word = 0;
    for (_, document) in &global_documents.word_count {
        if document.word_count.contains_key(word) {
            document_containing_word += 1;
        }
    }
    let numerator = (global_documents.lenght - document_containing_word) as f32 + 0.5;
    let denominator = document_containing_word as f32 + 0.5;

    return (1f32 + (numerator / denominator)).log10();
}

fn score(document: &Document, query: &Vec<String>, global_documents: &Global) -> i32 {
    let k1 = 1.2;
    let b = 0.75; // magic numbers
    let zero: usize = 0;

    let mut score: f32 = 0.0;

    let mut d_on_avgdl = 0;
    for (_name, doc) in &global_documents.word_count {
        d_on_avgdl += doc.lenght;
    }
    let d_on_avgdl = d_on_avgdl as f32 / global_documents.lenght as f32;

    for word in query {
        let freq_word_document = document.word_count.get(word).unwrap_or(&zero).to_owned();
        let idf = idf(&word, &global_documents);
        let numerator = freq_word_document as f32 * (k1 + 1.0);
        let denominator = freq_word_document as f32 + k1 * (1. - b + b * d_on_avgdl);
        score += idf * (numerator / denominator);
    }
    unsafe { (score * 1000.0).round().to_int_unchecked() } // TODO
}
