use git2::{Repository, Signature};
use serde_derive::Deserialize;
use slug::slugify;
use chrono::{FixedOffset, TimeZone};

use super::page::{Metadata, RawPage};

pub use git2::Error;
type Result<T> = std::result::Result<T, anyhow::Error>;

pub struct Repo {
    repo: Repository,
}

#[derive(Deserialize, Debug)]
pub struct CommitInfo {
    pub content: String,
    pub title: String,
    pub private: bool,
}

#[derive(Deserialize, Debug)]
pub struct CommitLog {
    pub msg: String,
    pub author: String,
    pub hash: String,
    pub date: String,
}

impl Repo {
    pub fn open(path: &str) -> Result<Repo> {
        let repo = Repository::open_bare(path).or_else(|_| Repository::init_bare(path))?;
        Ok(Repo { repo })
    }

    pub fn get_file(&self, path: &str) -> Result<String> {
        let obj = self.repo.revparse_single(&format!("master:{}", path))?;
        let blob = obj.peel_to_blob()?;
        let content = std::str::from_utf8(blob.content()).expect("not utf8");
        Ok(content.to_owned())
    }

    fn parse_page(content: &str) -> Result<RawPage> {
        if !content.starts_with("---") {
            anyhow::bail!("missing YAML front matter");
        }
        let yaml = content.trim_start_matches("---");
        let (yaml, md) = yaml
            .split_once("---")
            .ok_or_else(|| anyhow::anyhow!("malformed YAML front matter"))?;
        let meta = serde_yaml::from_str(yaml).expect("invalid YAML front matter");
        Ok(RawPage {
            meta,
            content: md.to_owned(),
        })
    }
    fn write_page(p: RawPage) -> Result<String> {
        let mut ret = String::new();
        ret.push_str("---\n");
        let yaml = serde_yaml::to_string(&p.meta)?;
        ret.push_str(&yaml);
        ret.push_str("\n---\n");
        ret.push_str(&p.content);
        Ok(ret)
    }

    pub fn list_files(&self, path: &str) -> Result<Vec<Metadata>> {
        let obj = self.repo.revparse_single(&format!("master:{}", path))?;
        let tree = obj.peel_to_tree()?;
        Ok(tree
            .iter()
            .map(|e| {
                e.to_object(&self.repo)
                    .map_err(|e| anyhow::anyhow!(e))
                    .and_then(|o| Ok(o.peel_to_blob()?))
                    .and_then(|b| Ok(Self::parse_page(std::str::from_utf8(b.content())?)?.meta))
            })
            .filter_map(std::result::Result::ok)
            .collect())
    }

    pub fn page_getter(&self, path: &str) -> Result<RawPage> {
        let content = self.get_file(&format!("{}.md", path))?;
        Self::parse_page(&content)
    }

    pub fn page_committer(&self, author: String, info: CommitInfo) -> Result<String> {
        let link = slugify(&info.title);

        let obj = self.repo.revparse_single("master:")?;
        let tree = obj.peel_to_tree()?;

        let meta = Metadata {
            title: info.title.clone(),
            link: link.clone(),
            private: info.private,
        };
        let page = RawPage {
            meta,
            content: info.content,
        };
        let content = Self::write_page(page)?;
        let mut treebuilder = self.repo.treebuilder(Some(&tree))?;
        let blob = self.repo.blob(content.as_bytes())?;
        treebuilder.insert(&format!("{}.md", link), blob, 0o100_644)?;

        let oid = treebuilder.write()?;
        let newtree = self.repo.find_tree(oid)?;

        let sig = Signature::now(&author, &format!("{}@peori.space", author))?;
        let branch = self.repo.find_branch("master", git2::BranchType::Local)?;
        self.repo.commit(
            branch.get().name(),
            &sig,
            &sig,
            &format!("Edited `{}` from web", info.title),
            &newtree,
            &[&branch.get().peel_to_commit()?],
        )?;

        Ok(link)
    }

    pub fn get_log(&self) -> Result<Vec<CommitLog>> {
        let mut walk = self.repo.revwalk()?;
        walk.push_head()?;
        let mut ret = Vec::new();
        for oid in walk {
            let commit = self.repo.find_commit(oid?)?;

            let time = commit.time();
            let tz = FixedOffset::east_opt(time.offset_minutes()*60).ok_or_else(|| anyhow::anyhow!("wrong timezone offset"))?;
            let date = tz.timestamp_opt(time.seconds(), 0).single().ok_or_else(|| anyhow::anyhow!("wrong timestamp"))?;
            let date = date.to_rfc2822();

            ret.push(CommitLog {
                author: commit.author().name().ok_or_else(|| anyhow::anyhow!("author not utf8"))?.to_owned(),
                msg: commit.message().ok_or_else(|| anyhow::anyhow!("msg not utf8"))?.to_owned(),
                hash: format!("{}", commit.id()),
                date
            });
        }
        Ok(ret)
    }
}
