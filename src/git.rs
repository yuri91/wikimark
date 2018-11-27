use git2::Repository;

pub fn get_repo(path: &str) -> Repository {
    Repository::open_bare(path).unwrap_or_else(|_| {
        info!("Creating bare repo at {}", path);
        Repository::init_bare(path).expect("cannot create repo")
    })
}

pub fn get_file(path: &str, repo: &Repository) -> String {
    let obj = repo.revparse_single(&format!("master:{}", path)).expect("no spec");
    let blob = obj.peel_to_blob().expect("no blob");
    let content = std::str::from_utf8(blob.content()).expect("not utf8");
    content.to_owned()
}
pub fn list_files(path: &str, repo: &Repository) -> Vec<String> {
    let obj = repo.revparse_single(&format!("master:{}", path)).expect("no spec");
    let tree = obj.peel_to_tree().expect("no tree");
    tree.iter().filter_map(|e| {
        e.name().map(|e| e.to_owned())
    }).collect()
}

pub fn file_getter(repo_path: &str) -> impl Fn(String) -> String + Clone {
    let repo_path = repo_path.to_owned();
    move |path| {
        let repo = get_repo(&repo_path);
        get_file(&path, &repo)
    }
}
