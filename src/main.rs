#![deny(unsafe_code)]

use std::{
    env,
    process,
    sync::OnceLock,
    time::{Duration, Instant},
};

use futures_util::StreamExt;
use regex::{Regex, RegexBuilder};
use reqwest::Client;
use serde_json::Value;
use tokio::time;
use urlencoding::encode;
use strsim::jaro_winkler;

/* ========================= EXIT CODES ========================= */

#[derive(Debug)]
enum ExitCode {
    Network = 1,
    Schema = 2,
    Server = 3,
    NotFound = 4,
}

/* ========================= ANSI ========================= */

fn orange(text: &str) -> String {
    format!("\x1b[38;2;255;165;0m{}\x1b[0m", text)
}

/* ========================= SANITIZE ========================= */

fn sanitize(input: &str) -> String {
    static ANSI_REGEX: OnceLock<Regex> = OnceLock::new();

    let ansi = ANSI_REGEX.get_or_init(|| {
        Regex::new(r"\x1B\[[0-9;]*[A-Za-z]").unwrap()
    });

    let stripped = ansi.replace_all(input, "");

    stripped
        .chars()
        .filter(|&c| matches!(c, '\n' | '\t') || ((c as u32) >= 0x20 && c != '\x7f'))
        .collect()
}

fn trim_wikipedia_tail(text: &str) -> String {
    const STOP_SECTIONS: &[&str] = &[
        "== References ==",
        "== External links ==",
        "== Further reading ==",
        "== Bibliography ==",
        "== Notes ==",
        "== Sources ==",
        "== Citations ==",
        "== See also ==",
        "== Filmography ==",
        "== Discography ==",
        "== Works ==",
        "== Publications ==",
        "== Note ==",
        "== Bibliografia ==",
        "== Collegamenti esterni ==",
        "== Altri progetti ==",
        "== Voci correlate ==",
        "== Opere ==",
        "== Pubblicazioni ==",
    ];

    let mut cut = text.len();

    for marker in STOP_SECTIONS {
        if let Some(pos) = text.find(marker) {
            cut = cut.min(pos);
        }
    }

    text[..cut].trim_end().to_string()
}

/* ========================= MATH CLEANUP ========================= */

fn clean_math(text: &str) -> String {
    let mut out = text.to_string();

    let re1 = Regex::new(r"\{\s*\\displaystyle[^}]*\}").unwrap();
    out = re1.replace_all(&out, "").to_string();

    let re2 = Regex::new(r"\\[a-zA-Z]+").unwrap();
    out = re2.replace_all(&out, "").to_string();

    let re3 = Regex::new(r"[{}]").unwrap();
    out = re3.replace_all(&out, "").to_string();

    let re4 = Regex::new(r"\s{2,}").unwrap();
    out = re4.replace_all(&out, " ").to_string();

    out
}

/* ========================= HIGHLIGHT ========================= */

fn bold_matches(text: &str, query: &str) -> String {
    let pattern = regex::escape(query);

    let re = RegexBuilder::new(&pattern)
        .case_insensitive(true)
        .build()
        .unwrap();

    re.replace_all(text, "\x1b[1m$0\x1b[0m").to_string()
}

fn format_headings(text: &str) -> String {
    static HEADING_RE: OnceLock<Regex> = OnceLock::new();

    let re = HEADING_RE.get_or_init(|| {
        Regex::new(r"==+\s*(.*?)\s*==+").unwrap()
    });

    re.replace_all(text, "\n\n\x1b[1m$1\x1b[0m\n")
        .to_string()
}

/* ========================= WATCHDOG ========================= */

struct EpochSampler {
    epoch_start: Instant,
    bytes: u64,
    low_epochs: u8,
}

impl EpochSampler {
    fn new() -> Self {
        Self {
            epoch_start: Instant::now(),
            bytes: 0,
            low_epochs: 0,
        }
    }

    fn observe(&mut self, chunk_size: usize) -> Result<(), ExitCode> {
        self.bytes += chunk_size as u64;

        let elapsed = self.epoch_start.elapsed().as_secs_f64();

        if elapsed >= 1.0 {
            let rate = self.bytes as f64 / elapsed;

            if rate < 10.0 {
                self.low_epochs += 1;
            } else {
                self.low_epochs = 0;
            }

            if self.low_epochs >= 2 {
                return Err(ExitCode::Network);
            }

            self.bytes = 0;
            self.epoch_start = Instant::now();
        }

        Ok(())
    }
}

/* ========================= TRANSPORT ========================= */

async fn transport(
    client: &Client,
    req: reqwest::Request,
    sampler: &mut EpochSampler,
) -> Result<Vec<u8>, ExitCode> {
    let res = client.execute(req).await.map_err(|_| ExitCode::Network)?;

    match res.status().as_u16() {
        404 => return Err(ExitCode::NotFound),
        s if s >= 500 => return Err(ExitCode::Server),
        _ => {}
    }

    let mut buf = Vec::new();
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|_| ExitCode::Network)?;

        sampler.observe(chunk.len())?;
        buf.extend_from_slice(&chunk);
    }

    Ok(buf)
}

/* ========================= SEARCH EN + IT ========================= */

async fn search_best(client: &Client, query: &str) -> Result<(String, String), ExitCode> {
    let mut sampler = EpochSampler::new();

    let mut best_title = None;
    let mut best_score = 0.0;
    let mut best_lang = "en";

    for (lang, base) in [
        ("en", "https://en.wikipedia.org"),
        ("it", "https://it.wikipedia.org"),
    ] {
        let url = format!(
            "{}/w/api.php?action=opensearch&search={}&limit=5&namespace=0&format=json",
            base,
            encode(query)
        );

        let req = client.get(url).build().map_err(|_| ExitCode::Network)?;
        let bytes = transport(client, req, &mut sampler).await?;

        let json: Value = serde_json::from_slice(&bytes).map_err(|_| ExitCode::Schema)?;
        let arr = json.as_array().ok_or(ExitCode::Schema)?;
        let titles = arr[1].as_array().ok_or(ExitCode::Schema)?;

        for t in titles {
            if let Some(title) = t.as_str() {
                let score = jaro_winkler(&query.to_lowercase(), &title.to_lowercase());

                if score > best_score {
                    best_score = score;
                    best_title = Some(title.to_string());
                    best_lang = lang;
                }
            }
        }
    }

    let title = best_title.ok_or(ExitCode::NotFound)?;
    Ok((title, best_lang.to_string()))
}

/* ========================= LONG EXTRACT ========================= */

async fn fetch_extract(
    client: &Client,
    title: &str,
    lang: &str,
) -> Result<String, ExitCode> {
    let mut sampler = EpochSampler::new();

    let base = if lang == "it" {
        "https://it.wikipedia.org"
    } else {
        "https://en.wikipedia.org"
    };

    // ADDED &redirects=1 HERE
    let url = format!(
        "{}/w/api.php?action=query&prop=extracts&explaintext=1&redirects=1&formatversion=2&titles={}&format=json",
        base,
        encode(title)
    );

    let req = client.get(url).build().map_err(|_| ExitCode::Network)?;
    let bytes = transport(client, req, &mut sampler).await?;

    let v: Value = serde_json::from_slice(&bytes).map_err(|_| ExitCode::Schema)?;

    let pages = v["query"]["pages"]
        .as_array()
        .ok_or(ExitCode::Schema)?;

    let page = pages.get(0).ok_or(ExitCode::NotFound)?;

    let extract = page["extract"]
        .as_str()
        .ok_or(ExitCode::NotFound)?
        .to_string();

    Ok(extract)
}

/* ========================= PIPELINE ========================= */

async fn run(query: String) -> Result<(), ExitCode> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("WikiCLI")
        .build()
        .map_err(|_| ExitCode::Network)?;

    let (title, lang) = search_best(&client, &query).await?;
    let summary = fetch_extract(&client, &title, &lang).await?;

    let clean = sanitize(&summary);
    let clean = clean_math(&clean);
    let clean = trim_wikipedia_tail(&clean);
    let clean = format_headings(&clean);
    let highlighted = bold_matches(&clean, &query);

    println!("{} {}", orange("Searching for:"), title);
    println!("{} {}", orange("Found:"), highlighted);

    Ok(())
}

/* ========================= MAIN ========================= */

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: wiki <query>");
        process::exit(1);
    }

    let query = args[1..].join(" ");

    let result = time::timeout(Duration::from_secs(10), run(query)).await;

    match result {
        Ok(Ok(())) => process::exit(0),
        Ok(Err(e)) => process::exit(e as i32),
        Err(_) => process::exit(1),
    }
}
