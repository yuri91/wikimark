use serde::ser::{Serialize, SerializeSeq, SerializeStruct, Serializer};
use slab_tree::{NodeRef, Tree};

#[derive(Serialize, Debug)]
pub struct PageInfo {
    pub title: String,
    pub permalink: String,
}

pub struct TocChildren<'a>(&'a NodeRef<'a, Section>);
impl<'a> Serialize for TocChildren<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_seq(None)?;
        for c in self.0.children() {
            state.serialize_element(&TocNode(&c))?;
        }
        state.end()
    }
}
pub struct TocNode<'a>(&'a NodeRef<'a, Section>);
impl<'a> Serialize for TocNode<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Section", 3)?;
        state.serialize_field("title", &self.0.data().title)?;
        state.serialize_field("link", &self.0.data().link)?;
        state.serialize_field("level", &self.0.data().link)?;
        state.serialize_field("children", &TocChildren(&self.0))?;
        state.end()
    }
}
pub struct TocTree(pub Tree<Section>);
impl Serialize for TocTree {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let root = self.0.root();
        TocNode(&root).serialize(serializer)
    }
}

#[derive(Serialize)]
pub struct Page {
    pub toc: TocTree,
    pub slug: String,
    pub content: String,
    pub title: String,
}
#[derive(Serialize, Debug)]
pub struct Section {
    pub link: String,
    pub title: String,
    pub level: i32,
}
