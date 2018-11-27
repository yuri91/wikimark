use pulldown_cmark::{html, Event, Parser, Tag};
use serde::ser::{Serialize, SerializeSeq, SerializeStruct, Serializer};
use slab_tree::{NodeRef, Tree};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::html::{
    start_highlighted_html_snippet, styled_line_to_highlighted_html, IncludeBackground,
};
use syntect::parsing::{SyntaxReference, SyntaxSet};

use std::borrow::Cow::{Borrowed, Owned};

fn get_syntax_for_block<'a>(set: &'a SyntaxSet, hint: &str) -> &'a SyntaxReference {
    set.find_syntax_by_name(hint).unwrap_or_else(|| {
        set.find_syntax_by_extension(hint)
            .unwrap_or_else(|| set.find_syntax_plain_text())
    })
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
pub struct TocTree(Tree<Section>);
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

enum ParsingPhase<'a> {
    Normal,
    Code(HighlightLines<'a>),
    Header(String),
}

pub struct ParseContext {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}
impl ParseContext {
    pub fn new() -> ParseContext {
        ParseContext {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }
}

pub fn parse(md: &str, parse_context: &ParseContext) -> Page {
    let theme = &parse_context.theme_set.themes["base16-ocean.dark"];
    let parser = Parser::new(&md);
    let mut out = String::new();
    let mut phase = ParsingPhase::Normal;
    let mut toc_tree = TocTree(Tree::new(Section {
        link: "link".to_owned(),
        title: "title".to_owned(),
        level: 0,
    }));
    let mut cur_section = toc_tree.0.root_mut().node_id();

    {
        let toc = &mut toc_tree;
        let parser = parser.map(move |event| match event {
            Event::Start(Tag::CodeBlock(ref info)) => {
                let syntax = get_syntax_for_block(&parse_context.syntax_set, info);
                let highlighter = HighlightLines::new(syntax, theme);
                phase = ParsingPhase::Code(highlighter);
                let snippet = start_highlighted_html_snippet(theme);
                Event::Html(Owned(snippet.0))
            }
            Event::End(Tag::CodeBlock(_)) => {
                phase = ParsingPhase::Normal;
                Event::Html(Borrowed("</pre>"))
            }
            Event::Text(text) => match phase {
                ParsingPhase::Code(ref mut highlighter) => {
                    let ranges = highlighter.highlight(&text, &parse_context.syntax_set);
                    let h = styled_line_to_highlighted_html(&ranges[..], IncludeBackground::Yes);
                    Event::Html(Owned(h))
                }
                ParsingPhase::Header(ref mut h) => {
                    h.push_str(&text);
                    Event::Text(text)
                }
                ParsingPhase::Normal => Event::Text(text),
            },
            Event::Start(Tag::Header(level)) => {
                if level <= toc.0.get_mut(cur_section).unwrap().data().level {
                    cur_section = toc
                        .0
                        .get_mut(cur_section)
                        .unwrap()
                        .parent()
                        .expect("no parent")
                        .node_id();
                }
                cur_section = toc
                    .0
                    .get_mut(cur_section)
                    .unwrap()
                    .append(Section {
                        link: "".to_owned(),
                        title: "title".to_owned(),
                        level,
                    }).node_id();
                phase = ParsingPhase::Header(String::new());
                event
            }
            Event::End(Tag::Header(_)) => {
                phase = match phase {
                    ParsingPhase::Header(ref h) => {
                        toc.0.get_mut(cur_section).unwrap().data().title = h.clone();
                        ParsingPhase::Normal
                    }
                    _ => panic!("impossible phase"),
                };
                event
            }
            _ => event,
        });
        html::push_html(&mut out, parser);
    }
    Page {
        toc: toc_tree,
        title: "test".to_owned(),
        slug: "test".to_owned(),
        content: out,
    }
}
