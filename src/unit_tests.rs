use crate::boundary::github_name;

#[test]
fn repository_boundary_rejects_owner_qualified_names() {
    assert!(github_name("repository", "example-core").is_ok());
    assert!(github_name("repository", "owner/repository").is_err());
}
