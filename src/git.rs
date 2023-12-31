use chrono::{FixedOffset, TimeZone};
use gix::{actor::Signature, create, object, Repository, ThreadSafeRepository, Tree};
use serde_derive::{Deserialize, Serialize};
use slug::slugify;
use std::path::Path;

use super::page::{Metadata, RawPage};

type Result<T> = std::result::Result<T, anyhow::Error>;

#[derive(Clone)]
pub struct Repo {
    repo: ThreadSafeRepository,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CommitLog {
    pub msg: String,
    pub author: String,
    pub hash: String,
    pub date: String,
}

impl Repo {
    pub fn open(path: &str) -> Result<Repo> {
        let repo = ThreadSafeRepository::open(path).or_else(|_| {
            ThreadSafeRepository::init(path, create::Kind::Bare, create::Options::default())
        })?;
        Ok(Repo { repo })
    }

    fn get_blob(&self, path: &str) -> Result<Vec<u8>> {
        let repo = self.repo.to_thread_local();
        let id = repo.rev_parse_single(format!("master:{}", path).as_bytes())?;
        let obj = id.object()?;
        let blob = obj.peel_to_kind(object::Kind::Blob)?;
        Ok(blob.data.clone())
    }

    fn get_tree<'a>(repo: &'a Repository, path: &str) -> Result<Tree<'a>> {
        let id = repo.rev_parse_single(format!("master:{}", path).as_bytes())?;
        let obj = id.object()?;
        let tree = obj.peel_to_kind(object::Kind::Tree)?.into_tree();
        Ok(tree)
    }

    pub fn get_file(&self, path: &str) -> Result<String> {
        let data = self.get_blob(path)?;
        let content = String::from_utf8(data.to_owned())?;
        Ok(content)
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
    fn write_page(p: &RawPage) -> Result<String> {
        let mut ret = String::new();
        ret.push_str("---\n");
        let yaml = serde_yaml::to_string(&p.meta)?;
        ret.push_str(&yaml);
        ret.push_str("\n---\n");
        ret.push_str(&p.content);
        Ok(ret)
    }

    pub fn list_files(&self, path: &str) -> Result<Vec<Metadata>> {
        use gix::objs::tree::EntryKind;
        let repo = self.repo.to_thread_local();
        let tree = Self::get_tree(&repo, path)?;
        let mut ret = vec![];
        for e in tree.iter() {
            let e = e?;
            match e.mode().kind() {
                EntryKind::Blob => {
                    let obj = e.object()?;
                    let blob = obj.peel_to_kind(object::Kind::Blob)?;
                    ret.push(Self::parse_page(std::str::from_utf8(&blob.data)?)?.meta);
                }
                EntryKind::Tree => {
                }
                _ => {
                }
            }
        }
        Ok(ret)
    }

    pub fn page_getter(&self, path: &str) -> Result<RawPage> {
        let content = self.get_file(&format!("{}.md", path))?;
        Self::parse_page(&content)
    }

    pub fn page_committer(&self, author: String, page: RawPage) -> Result<String> {
        let link = slugify(&page.meta.title);
        let content = Self::write_page(&page)?;

        let repo = self.repo.to_thread_local();
        let tree = Self::get_tree(&repo, "")?;
        let mut treebuilder = TreeUpdateBuilder::new();

        let blob_id = repo.write_blob(content.as_bytes())?;
        treebuilder.upsert_blob(&format!("{}.md", link), blob_id.into());

        let oid = treebuilder.create_updated(&repo, &tree);
        let newtree = repo.find_object(oid).unwrap();

        let sig = Signature {
            name: author.clone().into(),
            email: format!("{}@peori.space", author).into(),
            time: gix::date::Time::now_local_or_utc(),
        };
        let mut branch = repo.find_reference("master")?;
        let parent = branch.peel_to_id_in_place()?;
        repo.commit_as(
            &sig,
            &sig,
            branch.name().as_bstr(),
            &format!("Edited `{}` from web", page.meta.title),
            newtree.id,
            Some(parent),
        )
        .unwrap();

        Ok(link)
    }

    pub fn get_log(&self) -> Result<Vec<CommitLog>> {
        let repo = self.repo.to_thread_local();
        let head = repo.head_id()?.object()?.id;
        let walk = repo.rev_walk(Some(head));
        let mut ret = Vec::new();
        for info in walk.all()? {
            let info = info?;
            let commit = repo.find_object(info.id)?.into_commit();

            let time = commit.time()?;
            let tz = FixedOffset::east_opt(time.offset)
                .ok_or_else(|| anyhow::anyhow!("wrong timezone offset"))?;
            let date = tz
                .timestamp_opt(time.seconds, 0)
                .single()
                .ok_or_else(|| anyhow::anyhow!("wrong timestamp"))?;
            let date = date.to_rfc2822();

            ret.push(CommitLog {
                author: commit.author()?.name.to_string(),
                msg: commit.message()?.summary().to_string(),
                hash: format!("{}", commit.id()),
                date,
            });
        }
        Ok(ret)
    }
}

enum UpdateEntry {
    Blob(gix::ObjectId),
    Tree(UpdateTree),
}

type UpdateTree = std::collections::BTreeMap<Vec<u8>, UpdateEntry>;

struct TreeUpdateBuilder {
    update_tree: UpdateTree,
}

impl TreeUpdateBuilder {
    fn new() -> Self {
        Self {
            update_tree: UpdateTree::new(),
        }
    }

    fn upsert_blob(&mut self, path: &str, oid: gix::ObjectId) {
        let path = Path::new(path);
        let ancestors = path.parent().unwrap();
        let file_name = path.file_name().unwrap().as_encoded_bytes();

        let mut ct = &mut self.update_tree;

        for comp in ancestors.components() {
            let comp = comp.as_os_str().as_encoded_bytes();
            let entry = ct
                .entry(comp.to_owned())
                .or_insert_with(|| UpdateEntry::Tree(UpdateTree::new()));

            if let UpdateEntry::Tree(t) = entry {
                ct = t;
            } else {
                panic!("blob already inserted");
            }
        }

        if ct.contains_key(file_name) {
            panic!("tree already inserted with same filename as blob");
        }

        ct.insert(file_name.to_owned(), UpdateEntry::Blob(oid));
    }

    fn create_updated(self, repo: &gix::Repository, tree: &Tree<'_>) -> gix::ObjectId {
        Self::create_inner(self.update_tree, tree, repo)
    }

    fn create_inner(
        tree: UpdateTree,
        current: &gix::Tree<'_>,
        repo: &gix::Repository,
    ) -> gix::ObjectId {
        use gix::objs::{
            tree::{Entry, EntryKind},
            Tree,
        };

        let mut nt = Tree::empty();
        let tree_ref = current.decode().unwrap();

        // Since they are stored in a btreemap we don't have to worry about
        // sorting here to satisfy the constraints of Tree
        for (filename, entry) in tree {
            match entry {
                UpdateEntry::Blob(oid) => {
                    nt.entries.push(Entry {
                        mode: EntryKind::Blob.into(),
                        oid,
                        filename: filename.clone().into(),
                    });
                }
                UpdateEntry::Tree(ut) => {
                    // Check if there is already an existing tree
                    let current_tree = tree_ref.entries.iter().find_map(|tre| {
                        if tre.filename == filename && tre.mode.is_tree() {
                            Some(repo.find_object(tre.oid).unwrap().into_tree())
                        } else {
                            None
                        }
                    });
                    let current_tree = current_tree.unwrap_or_else(|| repo.empty_tree());

                    let oid = Self::create_inner(ut, &current_tree, repo);
                    nt.entries.push(Entry {
                        mode: EntryKind::Tree.into(),
                        oid,
                        filename: filename.into(),
                    });
                }
            }
        }

        // Insert all the entries from the old tree that weren't added/modified
        // in this builder
        for entry in tree_ref.entries {
            if let Err(i) = nt
                .entries
                .binary_search_by_key(&entry.filename, |e| e.filename.as_ref())
            {
                nt.entries.insert(
                    i,
                    Entry {
                        mode: entry.mode,
                        oid: entry.oid.into(),
                        filename: entry.filename.to_owned(),
                    },
                );
            }
        }

        repo.write_object(nt).unwrap().detach()
    }
}
