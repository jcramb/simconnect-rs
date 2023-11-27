use std::path::{Path, PathBuf};
use scraper::{Html, Selector};
use semver::Version;

const VERSION_TXT: &'static str = "simconnect-sys/sdk/version.txt";
const RELEASE_URL: &'static str = "https://docs.flightsimulator.com/html/Introduction/Release_Notes.htm";

fn main() {

    // scrape SimConnect release notes
    let html = reqwest::blocking::get(RELEASE_URL).unwrap().text().unwrap();
    let document = Html::parse_document(&html);
    let selector = Selector::parse("h2").unwrap();
    let el = document.select(&selector).next().unwrap();

    // extract the semantic version 
    let line = el.inner_html();
    let re = regex::Regex::new(r"[0-9]+\.[0-9]+\.[0-9]+").unwrap();
    let latest = match re.find(&line) {
        Some(semver) => semver,
        None => {
            println!("Failed to extract version from '{}'", line);
            std::process::exit(1)
        }
    };

    // load current version
    let version_txt = workspace_dir().join(VERSION_TXT);
    let current = std::fs::read_to_string(version_txt).unwrap();
    let latest_version = Version::parse(latest.into()).unwrap();
    let current_version = Version::parse(&current).unwrap();

    // display versions
    println!("Latest Version:  {}", latest_version);
    println!("Current Version: {}", current_version);

    // indicate new SDK version with exit code of 1
    if latest_version > current_version {
        println!("New SDK version available.");
        std::process::exit(1)
    } 
    println!("SimConnect SDK is latest version.");
}
     
fn workspace_dir() -> PathBuf {
    let output = std::process::Command::new(env!("CARGO"))
        .arg("locate-project")
        .arg("--workspace")
        .arg("--message-format=plain")
        .output()
        .unwrap()
        .stdout;
    let cargo_path = Path::new(std::str::from_utf8(&output).unwrap().trim());
    cargo_path.parent().unwrap().to_path_buf()
}