use git2::{Repository, Signature};
use log::info;
use serde_derive::Deserialize;

pub use git2::Error;
pub type Result<T> = std::result::Result<T, Error>;

pub struct Commit<'a> {
    pub author: &'a str,
    pub message: &'a str,
    pub path: &'a str,
    pub content: &'a str,
}
pub struct Repo {
    repo: Repository,
}
impl Repo {
    pub fn new(path: &str) -> Result<Repo> {
        let repo = Repository::open_bare(path).or_else(|_| {
            info!("Creating bare repo at {}", path);
            Repository::init_bare(path)
        })?;
        Ok(Repo { repo })
    }
    pub fn read(&self, path: &str) -> Result<String> {
        let obj = self.repo.revparse_single(&format!("master:{}", path))?;
        let blob = obj.peel_to_blob()?;
        let content = std::str::from_utf8(blob.content()).expect("not utf8");
        Ok(content.to_owned())
    }
    pub fn list(&self, path: &str) -> Result<Vec<String>> {
        let obj = self.repo.revparse_single(&format!("master:{}", path))?;
        let tree = obj.peel_to_tree()?;
        Ok(tree
            .iter()
            .map(|e| e.name().map(|n| n.to_owned()))
            .filter_map(|o| o)
            .collect())
    }
    pub fn commit(&self, c: Commit) -> Result<String> {
        let obj = self.repo.revparse_single("master:")?;
        let tree = obj.peel_to_tree()?;

        let mut treebuilder = self.repo.treebuilder(Some(&tree))?;
        let blob = self.repo.blob(c.content.as_bytes())?;
        treebuilder.insert(c.path, blob, 0o100_644)?;

        let oid = treebuilder.write()?;
        let newtree = self.repo.find_tree(oid)?;

        let sig = Signature::now(c.author, &format!("{}@peori.space", c.author))?;
        let branch = self.repo.find_branch("master", git2::BranchType::Local)?;
        self.repo.commit(
            branch.get().name(),
            &sig,
            &sig,
            "Edited from web interface",
            &newtree,
            &[&branch.get().peel_to_commit()?],
        )?;

        Ok(c.path.to_owned())
    }
}
