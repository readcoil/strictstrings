extern crate strsim;
use clap::{Arg, Command};
use indicatif::{ProgressBar, ProgressStyle, ProgressDrawTarget};
use lingua::{Language, LanguageDetectorBuilder};
use prettytable::{Table, row, Row, Cell};
use std::collections::HashSet;
use std::fs;
use std::io::{Read};
use std::io::prelude::*;
use std::ops::Not;
use std::path::Path;
use std::str;
use std::time::Instant;
use strsim::normalized_levenshtein;


fn process_text_candidate(text_candidate: &mut Vec<u8>, unique_strings: &mut HashSet<String>, filtered_by_len: &mut HashSet<String>, 
    logging: bool, min_length: usize, max_length: usize) {
    if let Ok(text) = str::from_utf8(text_candidate) {
        // Split the text on both carriage return and newline characters.
        let lines = text.split(|c| c == '\n' || c == '\r');
        for line in lines {
            let cleaned_line = line.trim();
            // Check if the cleaned line is not empty and within the specified length range before inserting.
            if !cleaned_line.is_empty() && cleaned_line.len() >= min_length && cleaned_line.len() <= max_length {
                unique_strings.insert(cleaned_line.to_string());
            } else {
                if logging {
                    filtered_by_len.insert(cleaned_line.to_string());
                }
            }
        }
    }
    text_candidate.clear();  // Always clear the buffer after processing
}


fn is_printable(c: u8) -> bool {
    (c >= 32 && c <= 126) || c == 9 || c == 10 || c == 13
}


fn print_remaining(count_remain: usize, quiet: bool) {
    if quiet.not() {
        if count_remain == 0 {
                println!("No strings found.");
                std::process::exit(1);
        }
        else {
            println!("Remaining strings: {}", count_remain);
        }
    }
}


fn main() -> Result<(), Box<dyn std::error::Error>>  {
    let matches = Command::new("StrictStrings")
        .version("0.1.0")
        .author("Julian Gutmanis <https://github.com/readcoil>")
        .about("Performs strict filtering on strings within a file contents.")
        .arg(Arg::new("infile")
            .help("Input file to process")
            .required(true)
            .index(1))
        .arg(Arg::new("outfile")
            .short('o')
            .long("out")
            .takes_value(true)
            .help("Output file write filtered strings")
            .required(false))
        .arg(Arg::new("threshold")
            .short('t')
            .long("language")
            .takes_value(true)
            .value_name("FLOAT")
            .help("Sets a custom language detection threshold")
            .default_value("0.5"))
        .arg(Arg::new("similarity")
            .short('s')
            .long("similarity")
            .takes_value(true)
            .value_name("FLOAT")
            .help("Sets a custom similarity filtering threshold")
            .default_value("0.8"))
        .arg(Arg::new("quiet")
            .short('q')
            .long("quiet")
            .takes_value(false)
            .help("Silences all output"))
        .arg(Arg::new("logdir")
            .short('l')
            .long("logs")
            .takes_value(true)
            .help("Output filtered values to log directory"))
        .arg(Arg::new("bytes")
            .short('b')
            .long("bytes")
            .takes_value(false)
            .help("Print byte representation after strings"))
        .arg(Arg::new("min")
            .short('m')
            .long("min")
            .takes_value(true)
            .value_name("MIN")
            .help("Minimum length of strings to process")
            .default_value("6"))
        .arg(Arg::new("max")
            .short('M')
            .long("max")
            .takes_value(true)
            .value_name("MAX")
            .help("Maximum length of strings to process")
            .default_value("200"))
        .arg(Arg::new("wslen")
            .short('W')
            .long("wslen")
            .takes_value(true)
            .value_name("wslen")
            .help("Maximum length of strings without whitespace")
            .default_value("30"))
        .get_matches();

    // impossible ngrams to filter.
    // note these are not filtered if '.' is present in the string.
    // currently only filters bigrams.
    let ngrams: HashSet<&str> = [
        "bk", "fq", "jc", "jt", "mj", "qh", "qx", "vj", "wz", "zh",
        "bq", "fv", "jd", "jv", "mq", "qj", "qy", "vk", "xb", "zj",
        "bx", "fx", "jf", "jw", "mx", "qk", "qz", "vm", "xg", "zn",
        "cb", "fz", "jg", "jx", "mz", "ql", "sx", "vn", "xj", "zq",
        "cf", "gq", "jh", "jy", "pq", "qm", "sz", "vp", "xk", "zr",
        "cg", "gv", "jk", "jz", "pv", "qn", "tq", "vq", "xv", "zs",
        "cj", "gx", "jl", "kq", "px", "qo", "tx", "vt", "xz", "zx",
        "cp", "hk", "jm", "kv", "qb", "qp", "vb", "vw", "yq",
        "cv", "hv", "jn", "kx", "qc", "qr", "vc", "vx", "yv",
        "cw", "hx", "jp", "kz", "qd", "qs", "vd", "vz", "yz",
        "cx", "hz", "jq", "lq", "qe", "qt", "vf", "wq", "zb",
        "dx", "iy", "jr", "lx", "qf", "qv", "vg", "wv", "zc",
        "fk", "jb", "js", "mg", "qg", "qw", "vh", "wx", "zg",
    ].iter().cloned().collect();

    let infile = matches.value_of("infile").unwrap();
    let lang_threshold: f64 = matches.value_of_t("threshold").unwrap_or_else(|e| e.exit());
    let leven_threshold: f64 = matches.value_of_t("similarity").unwrap_or_else(|e| e.exit());
    let quiet: bool = matches.is_present("quiet");
    let print_bytes: bool = matches.is_present("bytes");
    let min_length: usize = matches.value_of_t("min").unwrap_or_else(|e| e.exit());
    let max_length: usize = matches.value_of_t("max").unwrap_or_else(|e| e.exit());
    let wslen: usize = matches.value_of_t("wslen").unwrap_or_else(|e| e.exit());

    if !quiet {
        println!("Processing file:         {}", infile);
        println!("Language Threshold:      {}", lang_threshold);
        println!("Similarity Threshold:    {}", leven_threshold);
        println!("Minimum string length:   {}", min_length);
        println!("Maximum string length:   {}", max_length);
    }

    let start_time = Instant::now();
    let languages = vec![Language::English, Language::French,
                         Language::German, Language::Spanish,
                         Language::Russian, Language::Chinese];
    let detector = LanguageDetectorBuilder::from_languages(&languages).build();
    
    let mut file = fs::File::open(infile)?;
    let outfile_option = matches.value_of("outfile").map(String::from);
    let log_dir_option = matches.value_of("logdir").map(String::from);
    let logging = log_dir_option.is_some();

    if let Some(ref log_dir) = log_dir_option {
        let path = Path::new(log_dir);
        if !path.exists() {
            fs::create_dir_all(path).expect("Failed to create directory");
        }

    } 

    // Get file size for progress bar
    if !quiet {
        println!("Grabbing strings.");
    }

    let file_size = file.metadata()?.len() as u64;
    let pb_file = ProgressBar::new(file_size);
    pb_file.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
        .progress_chars("#>-"));

    // if quiet arg, set the progress_bar to hidden
    if quiet {
        pb_file.set_draw_target(ProgressDrawTarget::hidden());
    }

    let mut buffer = [0u8; 1024];
    let mut text_candidate = Vec::new();
    let mut unique_strings = HashSet::new();
    let mut filtered_by_len = HashSet::new();
    let mut lang_strings = HashSet::new();
    let mut final_strings = Vec::new();


    // String extraction loop
    while let Ok(bytes_read) = file.read(&mut buffer) {
        if !quiet {
            pb_file.inc(bytes_read as u64);
        }

        if bytes_read == 0 {
            break;
        }

        // Accumulate printable characters, including newlines.
        for &byte in buffer[0..bytes_read].iter() {
            if is_printable(byte) {
                text_candidate.push(byte);
            } else {
                // Process and clear the buffer when encountering a non-printable character
                process_text_candidate(&mut text_candidate, &mut unique_strings, &mut filtered_by_len, logging, min_length, max_length);
            }
        }
    }

    // Final processing to handle any remaining data
    if !text_candidate.is_empty() {
        process_text_candidate(&mut text_candidate, &mut unique_strings, &mut filtered_by_len, logging, min_length, max_length);
    }

    if logging {
        if let Some(ref log_dir) = log_dir_option {
            let mut log_file = fs::File::create(format!("{}/filtered_by_len.txt", log_dir))?;
            for string in filtered_by_len.iter() {
                log_file.write_all(string.as_bytes())?;
                log_file.write_all(b"\n")?;
            }
        }
    }

    // Filter strings over 30 characters that do not have whitespace or target special characters.
    // this avoids binary data that is not overly useful.
    if !quiet {
        println!("Filtering large strings without whitespace.");
    }  
    let mut filtered_by_whitespace = Vec::new();
    let mut remaining = Vec::new();

    for s in &unique_strings {
        if s.len() >= wslen {
            let contains_unencoded_whitespace = s.chars().any(|c| c.is_whitespace());
            let contains_encoded_space = s.contains("%20");
            let contains_encoded_tab = s.contains("%09");
            let contains_encoded_newline = s.contains("%0A") || s.contains("%0D%0A");
            let contains_encoded_carriage_return = s.contains("%0D");
            let contains_encoded_form_feed = s.contains("%0C");
            let contains_encoded_backslash = s.contains("%5C");
            let contains_encoded_arrowbracket = s.contains("%3E") || s.contains("%3C");
            let contains_encoded_colon = s.contains("%3A");
            let contains_encoded_slash = s.contains("%2F");

            if contains_unencoded_whitespace || contains_encoded_space || contains_encoded_tab || 
               contains_encoded_newline || contains_encoded_carriage_return || 
               contains_encoded_form_feed || contains_encoded_backslash || contains_encoded_slash ||
               contains_encoded_colon || contains_encoded_arrowbracket {
                remaining.push(s);
            } else {
                filtered_by_whitespace.push(s);
            }
        } else {
            remaining.push(s);
        }
    }
    if logging {
        if let Some(ref log_dir) = log_dir_option {
            let mut log_file = fs::File::create(format!("{}/filtered_by_whitespace.txt", log_dir))?;
            for string in filtered_by_whitespace.iter() {
                log_file.write_all(string.as_bytes())?;
                log_file.write_all(b"\n")?;
            }
        }
    }

    // English language detection
    let total_strings = unique_strings.len() as u64;
    let mut filtered_by_lang = Vec::new();

    println!("Total strings: {}", total_strings);
    if total_strings == 0 {
        if !quiet {
            println!("No strings found.");
        }
        std::process::exit(1);
    }

    let pb_lang = ProgressBar::new(total_strings);
    pb_lang.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")?
        .progress_chars("#>-"));

    // if quiet arg, set the progress_bar to hidden
    if quiet {
        pb_lang.set_draw_target(ProgressDrawTarget::hidden());
    } else {
        println!("Filtering English language.");
    }

    for text in unique_strings {
        if !quiet {
            pb_lang.inc(1);
        }
        
        let detected_languages = detector
            .compute_language_confidence_values(&text)
            .into_iter()
            .filter(|(lang, confidence)| *lang == Language::English && *confidence > lang_threshold)
            .collect::<Vec<_>>();

        if !detected_languages.is_empty() {
            let _ = lang_strings.insert(text.to_string());
        } else {
            filtered_by_lang.push(text);
        }
    }

    if logging {
        if let Some(ref log_dir) = log_dir_option {
            let mut log_file = fs::File::create(format!("{}/filtered_by_lang.txt", log_dir))?;
            for string in filtered_by_lang.iter() {
                log_file.write_all(string.as_bytes())?;
                log_file.write_all(b"\n")?;
            }
        }
    }

    // Print remaining strings
    print_remaining(lang_strings.len(), quiet);

    // filter impossible ngrams
    let lang_string_count: u64 = lang_strings.len().try_into().expect("Conversion to u64 failed");

    let pb_ngram = ProgressBar::new(lang_string_count);
    pb_ngram.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")?
        .progress_chars("#>-"));

    if quiet {
        pb_ngram.set_draw_target(ProgressDrawTarget::hidden());
    } else {
        println!("Filtering impossible ngrams.");
    }

    let remaining_no_ngrams: Vec<_> = lang_strings.iter().filter(|&s| {
        // Keep the string if it contains a dot (catch urls etc)
        if s.contains('.') {
            return true;
        }
        // Remove the string if it contains any ngrams.
        !s.chars().collect::<Vec<_>>().windows(2).any(|window| {
            let ngram_str: String = window.iter().collect();
            ngrams.contains(&ngram_str.as_str())
        })
    }).cloned().collect();

    let total_after_ngrams = remaining_no_ngrams.len() as u64;
    
    if logging {
        let remaining_no_ngrams_set: HashSet<_> = remaining_no_ngrams.iter().cloned().collect();
        let removed_by_ngram: Vec<_> = lang_strings.difference(&remaining_no_ngrams_set).cloned().collect();
        
        if let Some(ref log_dir) = log_dir_option {
            let mut log_file = fs::File::create(format!("{}/filtered_by_ngram.txt", log_dir))?;
            for string in removed_by_ngram.iter() {
                log_file.write_all(string.as_bytes())?;
                log_file.write_all(b"\n")?;
            }
        }
    }
    

    print_remaining(total_after_ngrams as usize, quiet);

    // Convert to Vec and sort
    let mut sorted_strings: Vec<_> = remaining_no_ngrams.into_iter().collect();
    sorted_strings.sort_by_key(|s| s.to_lowercase());

    // Levenshtein similarity filtering
    let sorted_cnt = sorted_strings.len() as u64;        

    let pb_sim = ProgressBar::new(sorted_cnt);
    pb_sim.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")?
        .progress_chars("#>-"));

    // if quiet arg, set the progress_bar to hidden
    if quiet {
        pb_sim.set_draw_target(ProgressDrawTarget::hidden());
    } else {
        println!("Removing similar strings.");
    }

    print_remaining(sorted_strings.len(), quiet);

    let mut current_string = &sorted_strings[0];
    let mut filtered_by_leven = Vec::new();

    for i in 1..sorted_strings.len() {
        if !quiet {
            pb_sim.inc(1);
        }

        let similarity = normalized_levenshtein(current_string, &sorted_strings[i]);
        
        if similarity < leven_threshold {
            final_strings.push(current_string.to_string());
            current_string = &sorted_strings[i];
        } else {
            filtered_by_leven.push(sorted_strings[i].to_string());
        }
    }
    if logging {
        if let Some(ref log_dir) = log_dir_option {
            let mut log_file = fs::File::create(format!("{}/filtered_by_leven.txt", log_dir))?;
            for string in filtered_by_leven.iter() {
                log_file.write_all(string.as_bytes())?;
                log_file.write_all(b"\n")?;
            }
        }
    }

    final_strings.push(current_string.to_string());
    final_strings.sort_by_key(|s| s.to_lowercase());



    // Print results
    if !quiet {
        println!("Final strings: {}\n\n", final_strings.len());
    }

    if print_bytes {
        let mut table = Table::new();
        table.add_row(row!["String", "UTF-Bytes", "Bytes"]);

        for string in final_strings.iter() {
            table.add_row(Row::new(vec![
                Cell::new(string),
                Cell::new(&format!("{:?}", string)),
                Cell::new(&format!("{:?}", string.as_bytes())),
            ]));
        }
        table.printstd();
    }
    else {
        for string in final_strings.iter() {
            println!("{}", string);
        }
    }

    if let Some(ref outfile) = outfile_option {
        let mut fout = fs::File::create(outfile)?;
        for string in final_strings.iter() {
            fout.write_all(string.as_bytes())?;
            fout.write_all(b"\n")?;
        }
    } 

    let final_cnt = final_strings.len() as u64;
    if !quiet {
        println!("\n\nUnique Strings:           {}", total_strings);
        println!("Language Filtered:        {}", sorted_cnt);
        println!("Ngram Filtered:           {}", total_after_ngrams);
        println!("Levenshtein Filtered:     {}", final_cnt);
    }

    let duration = start_time.elapsed();
    if !quiet {
        println!("\nExecution time: {:?}", duration);
    }

    Ok(())
}
