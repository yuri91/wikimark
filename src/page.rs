use serde_derive::{Deserialize, Serialize};
use slab_tree::{NodeRef, Tree};

pub struct TocNode<'a>(NodeRef<'a, Section>);

impl<'a> TocNode<'a> {
    pub fn title(&self) -> &str {
        &self.0.data().title
    }
    pub fn link(&self) -> &str {
        &self.0.data().link
    }
    pub fn level(&self) -> i32 {
        self.0.data().level
    }
    pub fn children(&'a self) -> Vec<TocNode<'a>> {
        self.0.children().map(|c| TocNode(c)).collect()
    }
}

pub struct TocTree(pub Tree<Section>);
impl TocTree {
    pub fn root<'a>(&'a self) -> TocNode<'a> {
        TocNode(self.0.root().unwrap())
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
    pub toc: TocTree,
    pub content: String,
}

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
