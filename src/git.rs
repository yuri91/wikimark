use git2::{Repository, Signature};
use log::info;
use serde_derive::Deserialize;
use slug::slugify;

use super::page::{Metadata, RawPage};

pub use git2::Error;
pub type Result<T> = std::result::Result<T, Error>;

pub fn get_repo(path: &str) -> Result<Repository> {
    Repository::open_bare(path).or_else(|_| {
        info!("Creating bare repo at {}", path);
        Repository::init_bare(path)
    })
}

pub fn get_file(path: &str, repo: &Repository) -> Result<String> {
    let obj = repo.revparse_single(&format!("master:{}", path))?;
    let blob = obj.peel_to_blob()?;
    let content = std::str::from_utf8(blob.content()).expect("not utf8");
    Ok(content.to_owned())
}
pub fn list_files(path: &str, repo: &Repository) -> Result<Vec<Metadata>> {
    let obj = repo.revparse_single(&format!("master:meta{}", path))?;
    let tree = obj.peel_to_tree()?;
    Ok(tree
        .iter()
        .map(|e| {
            e.to_object(&repo).and_then(|o| o.peel_to_blob()).map(|b| {
                serde_json::from_str(std::str::from_utf8(b.content()).expect("not utf8"))
                    .expect("not json")
            })
        })
        .filter_map(Result::ok)
        .collect())
}

pub fn page_getter(path: String, repo_path: &str) -> Result<RawPage> {
    let repo = get_repo(&repo_path)?;
    let content = get_file(&format!("{}.md", &path), &repo)?;
    let meta = get_file(&format!("meta/{}.json", &path), &repo)?;
    let meta = serde_json::from_str(&meta).expect("invalid json");
    Ok(RawPage { meta, content })
}

#[derive(Deserialize, Debug)]
pub struct CommitInfo {
    pub content: String,
    pub title: String,
}
fn transpose<T, E>(o: Option<std::result::Result<T, E>>) -> std::result::Result<Option<T>, E> {
    match o {
        Some(Ok(x)) => Ok(Some(x)),
        Some(Err(e)) => Err(e),
        None => Ok(None),
    }
}
pub fn page_committer(author: String, info: CommitInfo, repo_path: &str) -> Result<String> {
    let link = slugify(&info.title);
    let repo = get_repo(&repo_path)?;

    let obj = repo.revparse_single("master:")?;
    let tree = obj.peel_to_tree()?;

    let mut treebuilder = repo.treebuilder(Some(&tree))?;
    let blob = repo.blob(info.content.as_bytes())?;
    treebuilder.insert(&format!("{}.md", link), blob, 0o100_644)?;

    let meta = super::page::Metadata {
        title: info.title,
        link,
    };
    let blob = repo.blob(
        serde_json::to_string(&meta)
            .expect("cannot serialize")
            .as_bytes(),
    )?;
    let mut metatreebuilder = repo.treebuilder(
        transpose(
            tree.get_name("meta")
                .map(|t| t.to_object(&repo).and_then(|t| t.peel_to_tree())),
        )?
        .as_ref(),
    )?;
    metatreebuilder.insert(&format!("{}.json", meta.link), blob, 0o100_644)?;
    let oid = metatreebuilder.write()?;
    treebuilder.insert("meta", oid, 0o040_000)?;

    let oid = treebuilder.write()?;
    let newtree = repo.find_tree(oid)?;

    let sig = Signature::now(&author, &format!("{}@peori.space", author))?;
    let branch = repo.find_branch("master", git2::BranchType::Local)?;
    repo.commit(
        branch.get().name(),
        &sig,
        &sig,
        "Edited from web interface",
        &newtree,
        &[&branch.get().peel_to_commit()?],
    )?;

    Ok(meta.link)
}
