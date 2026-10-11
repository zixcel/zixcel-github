//! Local API-shape fixture. It never opens a socket or reads real credentials.

mod response;

use std::{fs, io::Read, path::PathBuf};

fn field<'a>(config: &'a str, key: &str) -> &'a str {
    let prefix = format!("{key} = \"");
    config
        .lines()
        .find_map(|line| line.strip_prefix(&prefix).and_then(|s| s.strip_suffix('"')))
        .expect("closed fixture configuration")
}

fn main() {
    if std::env::args().nth(1).as_deref() == Some("--version") {
        println!("curl 8.4.0 fixture");
        return;
    }
    let mut config = String::new();
    std::io::stdin().read_to_string(&mut config).expect("fixture input");
    let method = field(&config, "request");
    let url = field(&config, "url");
    let root = std::env::current_exe().expect("fixture path");
    let calls = root.parent().expect("fixture directory").join("calls.jsonl");
    use std::io::Write;
    let mut file = fs::OpenOptions::new().create(true).append(true).open(calls).expect("calls");
    writeln!(file, "{{\"method\":{method:?},\"url\":{url:?}}}").expect("record fixture request");
    let body = if method == "GET" {
        String::new()
    } else {
        let path = field(&config, "data-binary").strip_prefix('@').expect("body file");
        fs::read_to_string(PathBuf::from(path)).expect("fixture body")
    };
    let (status, response) = response::reply(method, url, &body);
    fs::write(field(&config, "output"), response).expect("fixture response");
    print!("X-GitHub-Request-Id: fixture-request\n");
    print!("{status}");
}
