use serde_derive::{Deserialize, Serialize};
use slab_tree::{Tree, RemoveBehavior};

#[derive(Serialize, Deserialize, Default)]
pub struct TocItem {
    pub section: Section,
    pub children: Vec<TocItem>,
}
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
                }
            }
        }
        Toc(stack.pop().unwrap().1)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Metadata {
    pub title: String,
    pub link: String,
    #[serde(default)]
    pub private: bool,
}

pub struct Page {
    pub toc: Toc,
    pub content: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Section {
    pub link: String,
    pub title: String,
    pub level: i32,
}

#[derive(Serialize, Deserialize)]
pub struct RawPage {
    pub meta: Metadata,
    pub content: String,
}
