use git2::{Repository, Signature};
use serde_derive::Deserialize;
use slug::slugify;

use super::page::{Metadata, RawPage};

pub use git2::Error;
type Result<T> = std::result::Result<T, anyhow::Error>;

fn transpose<T, E>(o: Option<std::result::Result<T, E>>) -> std::result::Result<Option<T>, E> {
    match o {
        Some(Ok(x)) => Ok(Some(x)),
        Some(Err(e)) => Err(e),
        None => Ok(None),
    }
}

pub struct Repo {
    repo: Repository,
}

#[derive(Deserialize, Debug)]
pub struct CommitInfo {
    pub content: String,
    pub title: String,
}
impl Repo {
    pub fn open(path: &str) -> Result<Repo> {
        let repo = Repository::open_bare(path).or_else(|_| {
            Repository::init_bare(path)
        })?;
        Ok(Repo {
            repo
        })
    }

    pub fn get_file(&self, path: &str) -> Result<String> {
        let obj = self.repo.revparse_single(&format!("master:{}", path))?;
        let blob = obj.peel_to_blob()?;
        let content = std::str::from_utf8(blob.content()).expect("not utf8");
        Ok(content.to_owned())
    }
    pub fn list_files(&self, path: &str) -> Result<Vec<Metadata>> {
        let obj = self.repo.revparse_single(&format!("master:meta{}", path))?;
        let tree = obj.peel_to_tree()?;
        Ok(tree
            .iter()
            .map(|e| {
                e.to_object(&self.repo).and_then(|o| o.peel_to_blob()).map(|b| {
                    serde_json::from_str(std::str::from_utf8(b.content()).expect("not utf8"))
                        .expect("not json")
                })
            })
            .filter_map(std::result::Result::ok)
            .collect())
    }

    pub fn page_getter(&self, path: String) -> Result<RawPage> {
        let content = self.get_file(&format!("{}.md", &path))?;
        let meta = self.get_file(&format!("meta/{}.json", &path))?;
        let meta = serde_json::from_str(&meta).expect("invalid json");
        Ok(RawPage { meta, content })
    }

    pub fn page_committer(&self, author: String, info: CommitInfo) -> Result<String> {
        let link = slugify(&info.title);

        let obj = self.repo.revparse_single("master:")?;
        let tree = obj.peel_to_tree()?;

        let mut treebuilder = self.repo.treebuilder(Some(&tree))?;
        let blob = self.repo.blob(info.content.as_bytes())?;
        treebuilder.insert(&format!("{}.md", link), blob, 0o100_644)?;

        let meta = super::page::Metadata {
            title: info.title,
            link,
        };
        let blob = self.repo.blob(
            serde_json::to_string(&meta)
                .expect("cannot serialize")
                .as_bytes(),
        )?;
        let mut metatreebuilder = self.repo.treebuilder(
            transpose(
                tree.get_name("meta")
                    .map(|t| t.to_object(&self.repo).and_then(|t| t.peel_to_tree())),
            )?
            .as_ref(),
        )?;
        metatreebuilder.insert(&format!("{}.json", meta.link), blob, 0o100_644)?;
        let oid = metatreebuilder.write()?;
        treebuilder.insert("meta", oid, 0o040_000)?;

        let oid = treebuilder.write()?;
        let newtree = self.repo.find_tree(oid)?;

        let sig = Signature::now(&author, &format!("{}@peori.space", author))?;
        let branch = self.repo.find_branch("master", git2::BranchType::Local)?;
        self.repo.commit(
            branch.get().name(),
            &sig,
            &sig,
            "Edited from web interface",
            &newtree,
            &[&branch.get().peel_to_commit()?],
        )?;

        Ok(meta.link)
    }
}
