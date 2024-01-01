use serde_derive::{Deserialize, Serialize};
use serde_yaml::Value;
use slab_tree::{RemoveBehavior, Tree};
use slug::slugify;
use std::collections::{BTreeMap};

use crate::git::{CommitData, EntryKind, Repo};

type Result<T> = std::result::Result<T, anyhow::Error>;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct TocItem {
    pub section: Section,
    pub children: Vec<TocItem>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Toc(pub TocItem);

impl Toc {
    pub fn new(mut tree: Tree<Section>) -> Toc {
        let mut stack = vec![(tree.root_id().unwrap(), TocItem::default())];
        while let Some((id, mut toc_it)) = stack.pop() {
            let node = tree.get(id).unwrap();
            if let Some(c) = node.children().next() {
                stack.push((id, toc_it));
                stack.push((c.node_id(), TocItem::default()));
            } else {
                toc_it.section = tree.remove(id, RemoveBehavior::OrphanChildren).unwrap();
                if let Some((p_id, mut p_toc_it)) = stack.pop() {
                    p_toc_it.children.push(toc_it);
                    stack.push((p_id, p_toc_it));
                } else {
                    stack.push((id, toc_it));
                    break;
                }
            }
        }
        Toc(stack.pop().unwrap().1)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    pub title: String,
    #[serde(default)]
    pub private: bool,
    #[serde(flatten)]
    pub other: BTreeMap<String, Value>,
}

pub struct Page {
    pub toc: Toc,
    pub content: String,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Section {
    pub link: String,
    pub title: String,
    pub level: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RawPage {
    pub meta: Metadata,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PageEntry {
    pub meta: Metadata,
    pub link: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PageUpdate {
    pub page: RawPage,
    pub dir: String,
}

pub fn parse_page(content: &str) -> Result<RawPage> {
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

pub fn list_files(repo: &Repo, path: &str, recursive: bool) -> Result<Vec<PageEntry>> {
    let tree = repo.get_tree(path)?;
    let mut ret = vec![];
    let mut stack = vec![(tree.id().into(), path.to_owned())];
    while let Some((id, prefix)) = stack.pop() {
        let tree = repo.get_tree_from_id(id).unwrap();
        for e in Repo::list_entries(&tree)? {
            match e.kind {
                EntryKind::File => {
                    if e.name == "_index.md" || !e.name.ends_with(".md") {
                        continue;
                    }
                    let name = &e.name[0..(e.name.len() - 3)];
                    let blob = repo.get_blob_from_id(e.id)?;
                    ret.push(PageEntry {
                        meta: parse_page(std::str::from_utf8(&blob)?)?.meta,
                        link: format!("{prefix}{name}"),
                    });
                }
                EntryKind::Dir => {
                    if let Ok(c) = repo.get_file(&format!("{path}{}/_index.md", e.name)) {
                        let link = format!("{prefix}{}/", e.name);
                        ret.push(PageEntry {
                            meta: parse_page(&c)?.meta,
                            link: link.clone(),
                        });
                        if recursive {
                            stack.push((e.id, link));
                        }
                    }
                }
            }
        }
    }
    ret.sort_by(|x, y| x.link.cmp(&y.link));
    Ok(ret)
}

pub fn get_page(repo: &Repo, path: &str) -> Result<RawPage> {
    let content = if path.ends_with('/') || path.is_empty() {
        repo.get_file(&format!("{}_index.md", path))?
    } else {
        repo.get_file(&format!("{}.md", path))?
    };
    parse_page(&content)
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

pub fn commit_page(repo: &Repo, author: String, update: PageUpdate) -> Result<String> {
    let fname = slugify(&update.page.meta.title);
    let mut dir = update.dir;
    if !dir.ends_with('/') {
        dir.push('/');
    }
    let link = format!("{dir}{fname}");
    let path = format!("{link}.md");
    let content = write_page(&update.page)?;

    let data = CommitData {
        author,
        removed: vec![],
        added: vec![(path, content)],
        msg: format!("Edited `{}` from web", update.page.meta.title),
    };
    repo.commit(&data)?;
    Ok(link)
}
