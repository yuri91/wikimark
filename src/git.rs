use chrono::{FixedOffset, TimeZone};
use gix::{actor::Signature, create, object, Repository, ThreadSafeRepository, Tree};
use serde_derive::{Deserialize, Serialize};
use std::path::Path;

type Result<T> = std::result::Result<T, anyhow::Error>;

#[derive(Clone)]
pub struct ThreadSafeRepo {
    repo: ThreadSafeRepository,
}

#[derive(Clone)]
pub struct Repo {
    repo: Repository,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CommitLog {
    pub msg: String,
    pub author: String,
    pub hash: String,
    pub date: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CommitData {
    pub msg: String,
    pub author: String,
    pub added: Vec<(String, String)>,
    pub removed: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum EntryKind {
    File,
    Dir,
}

#[derive(Debug)]
pub struct Entry {
    pub kind: EntryKind,
    pub name: String,
    pub id: gix::ObjectId,
}

impl ThreadSafeRepo {
    pub fn open(path: &str) -> Result<ThreadSafeRepo> {
        let repo = ThreadSafeRepository::open(path).or_else(|_| {
            ThreadSafeRepository::init(path, create::Kind::Bare, create::Options::default())
        })?;
        Ok(ThreadSafeRepo { repo })
    }
    pub fn local(&self) -> Repo {
        Repo {
            repo: self.repo.to_thread_local(),
        }
    }
}

impl Repo {
    fn get_blob(&self, path: &str) -> Result<Vec<u8>> {
        let id = self
            .repo
            .rev_parse_single(format!("master:{}", path).as_bytes())?;
        let obj = id.object()?;
        let blob = obj.peel_to_kind(object::Kind::Blob)?;
        Ok(blob.data.clone())
    }

    pub fn get_blob_from_id(&self, id: gix::ObjectId) -> Result<Vec<u8>> {
        let obj = self.repo.try_find_object(id)?.unwrap();
        let blob = obj.peel_to_kind(object::Kind::Blob)?;
        Ok(blob.data.clone())
    }

    pub fn get_tree<'a>(&'a self, path: &str) -> Result<Tree<'a>> {
        let id = self
            .repo
            .rev_parse_single(format!("master:{}", path).as_bytes())?;
        let obj = id.object()?;
        let tree = obj.peel_to_kind(object::Kind::Tree)?.into_tree();
        Ok(tree)
    }

    pub fn get_tree_from_id(&self, id: gix::ObjectId) -> Result<gix::Tree<'_>> {
        let obj = self.repo.try_find_object(id)?.unwrap();
        let tree = obj.peel_to_kind(object::Kind::Tree)?.into_tree();
        Ok(tree)
    }

    pub fn get_file(&self, path: &str) -> Result<String> {
        let data = self.get_blob(path)?;
        let content = String::from_utf8(data.to_owned())?;
        Ok(content)
    }

    pub fn list_entries<'a>(tree: &'a Tree<'_>) -> Result<impl Iterator<Item = Entry> + 'a> {
        use gix::objs::tree;
        Ok(tree.iter().filter_map(|e| {
            let e = e.ok()?;
            let kind = match e.mode().kind() {
                tree::EntryKind::Blob => EntryKind::File,
                tree::EntryKind::Tree => EntryKind::Dir,
                _ => {
                    return None;
                }
            };
            Some(Entry {
                name: e.filename().to_string(),
                id: e.object_id(),
                kind,
            })
        }))
    }

    pub fn commit(&self, data: &CommitData) -> Result<gix::ObjectId> {
        let tree = self.get_tree("")?;
        let mut treebuilder = TreeUpdateBuilder::new();

        for (path, content) in &data.added {
            let blob_id = self.repo.write_blob(content.as_bytes())?;
            treebuilder.upsert_blob(path, blob_id.into());
        }

        let oid = treebuilder.create_updated(&self.repo, &tree);
        let newtree = self.repo.find_object(oid).unwrap();

        let sig = Signature {
            name: data.author.clone().into(),
            email: format!("{}@peori.space", &data.author).into(),
            time: gix::date::Time::now_local_or_utc(),
        };
        let mut branch = self.repo.find_reference("master")?;
        let parent = branch.peel_to_id()?;
        let mut committer_buf = gix::date::parse::TimeBuf::default();
        let mut author_buf = gix::date::parse::TimeBuf::default();
        Ok(self
            .repo
            .commit_as(
                sig.to_ref(&mut committer_buf),
                sig.to_ref(&mut author_buf),
                branch.name().as_bstr(),
                &data.msg,
                newtree.id,
                Some(parent),
            )
            .unwrap()
            .into())
    }

    pub fn get_log(&self) -> Result<Vec<CommitLog>> {
        let head = self.repo.head_id()?.object()?.id;
        let walk = self.repo.rev_walk(Some(head));
        let mut ret = Vec::new();
        for info in walk.all()? {
            let info = info?;
            let commit = self.repo.find_object(info.id)?.into_commit();

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
