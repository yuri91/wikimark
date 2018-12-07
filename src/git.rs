use serde_derive::Deserialize;
use git2::{Repository, Signature, Error};
use log::info;

pub fn get_repo(path: &str) -> Repository {
    Repository::open_bare(path).unwrap_or_else(|_| {
        info!("Creating bare repo at {}", path);
        Repository::init_bare(path).expect("cannot create repo")
    })
}

pub fn get_file(path: &str, repo: &Repository) -> String {
    let obj = repo
        .revparse_single(&format!("master:{}", path))
        .expect("no spec");
    let blob = obj.peel_to_blob().expect("no blob");
    let content = std::str::from_utf8(blob.content()).expect("not utf8");
    content.to_owned()
}
pub fn list_files(path: &str, repo: &Repository) -> Vec<String> {
    let obj = repo
        .revparse_single(&format!("master:{}", path))
        .expect("no spec");
    let tree = obj.peel_to_tree().expect("no tree");
    tree.iter()
        .filter_map(|e| e.name().map(|e| e.to_owned()))
        .collect()
}

pub fn file_getter(repo_path: &str) -> impl Fn(String) -> String + Clone {
    let repo_path = repo_path.to_owned();
    move |path| {
        let repo = get_repo(&repo_path);
        get_file(&path, &repo)
    }
}

#[derive(Deserialize, Debug)]
pub struct CommitInfo {
    pub content: String,
    pub name: String,
}
pub fn file_committer(repo_path: &str) -> impl Fn(CommitInfo) -> Result<(), Error> + Clone {
    let repo_path = repo_path.to_owned();
    move |info| {
        let repo = get_repo(&repo_path);
        let obj = repo
            .revparse_single("master:")
            .expect("no spec");
        let tree = obj.peel_to_tree()?;
        let mut treebuilder = repo.treebuilder(Some(&tree))?;
        let blob = repo.blob(info.content.as_bytes())?;
        treebuilder.insert(&format!("{}.md", info.name), blob, 0o100644)?;
        let oid = treebuilder.write()?;
        let newtree = repo.find_tree(oid)?;
        let sig = Signature::now("yuri", "yuri@test.com")?;
        let branch = repo.find_branch("master", git2::BranchType::Local)?;
        repo.commit(branch.get().name(), &sig, &sig, "test commit message", &newtree, &[&branch.get().peel_to_commit()?])?;
        Ok(())
    }
}
