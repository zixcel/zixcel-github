//! Deterministic GitHub response shapes for the local subprocess fixture.

pub fn reply(method: &str, url: &str, body: &str) -> (u16, String) {
    let body = body.replace(' ', "");
    let repository = "{\"id\":12345,\"full_name\":\"example-org/repo\",\"private\":true}";
    match method {
        "GET" if url.ends_with("/users/example-org") => {
            (200, "{\"login\":\"example-org\",\"type\":\"Organization\"}".into())
        }
        "POST" if url.ends_with("/orgs/example-org/repos") => {
            assert!(body.contains("\"private\":true"));
            assert!(body.contains("\"auto_init\":true"));
            assert!(body.contains("\"name\":\"repo\""));
            (201, repository.into())
        }
        "GET" if url.contains("/git/ref/heads/") => {
            (200, format!("{{\"object\":{{\"sha\":\"{}\"}}}}", "a".repeat(40)))
        }
        "POST" if url.ends_with("/git/blobs") => {
            assert!(body.contains("\"encoding\":\"base64\""));
            (201, format!("{{\"sha\":\"{}\"}}", "b".repeat(40)))
        }
        "POST" if url.ends_with("/git/trees") => {
            assert!(!body.contains("\"base_tree\""));
            assert!(body.contains("\"path\":\"README.md\""));
            (201, format!("{{\"sha\":\"{}\"}}", "c".repeat(40)))
        }
        "POST" if url.ends_with("/git/commits") => {
            assert!(body.contains(&format!("\"parents\":[\"{}\"]", "a".repeat(40))));
            assert!(body.contains(&format!("\"tree\":\"{}\"", "c".repeat(40))));
            (201, format!("{{\"sha\":\"{}\"}}", "d".repeat(40)))
        }
        "PATCH" if url.contains("/git/refs/heads/") => {
            assert!(body.contains("\"force\":false"));
            (200, format!("{{\"ref\":\"refs/heads/main\",\"object\":{{\"sha\":\"{}\"}}}}", "d".repeat(40)))
        }
        "GET" if url.ends_with("/repos/example-org/repo") => (200, repository.into()),
        "DELETE" if url.ends_with("/repos/example-org/repo") => (204, String::new()),
        _ => (404, "{}".into()),
    }
}
