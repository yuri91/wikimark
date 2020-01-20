use once_cell::sync::Lazy;
use pulldown_cmark::{html, CowStr, Event, Parser, Tag};
use slug::slugify;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::html::{
    start_highlighted_html_snippet, styled_line_to_highlighted_html, IncludeBackground,
};
use syntect::parsing::{SyntaxReference, SyntaxSet};

fn get_syntax_for_block<'a>(set: &'a SyntaxSet, hint: &str) -> &'a SyntaxReference {
    set.find_syntax_by_name(hint).unwrap_or_else(|| {
        set.find_syntax_by_extension(hint)
            .unwrap_or_else(|| set.find_syntax_plain_text())
    })
}

use super::page::{Metadata, Page, Section, TocTree};
use slab_tree::Tree;

enum ParsingPhase<'a> {
    Normal,
    Code(Box<HighlightLines<'a>>),
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

static PARSE_CONTEXT: Lazy<ParseContext> = Lazy::new(ParseContext::new);

pub fn parse(md: &str, meta: &Metadata) -> Page {
    let theme = &PARSE_CONTEXT.theme_set.themes["base16-ocean.dark"];
    let parser = Parser::new(&md);
    let mut out = String::new();
    let mut phase = ParsingPhase::Normal;
    let mut tree = Tree::new();
    tree.set_root(Section {
        link: meta.link.clone(),
        title: meta.title.clone(),
        level: 0,
    });
    let mut toc_tree = TocTree(tree);
    let mut cur_section = toc_tree.0.root_mut().unwrap().node_id();

    {
        let toc = &mut toc_tree;
        let parser = parser.map(move |event| match event {
            Event::Start(Tag::CodeBlock(ref info)) => {
                let syntax = get_syntax_for_block(&PARSE_CONTEXT.syntax_set, info);
                let highlighter = Box::new(HighlightLines::new(syntax, theme));
                phase = ParsingPhase::Code(highlighter);
                let snippet = start_highlighted_html_snippet(theme);
                Event::Html(CowStr::Boxed(snippet.0.into_boxed_str()))
            }
            Event::End(Tag::CodeBlock(_)) => {
                phase = ParsingPhase::Normal;
                Event::Html(CowStr::Borrowed("</pre>"))
            }
            Event::Text(text) => match phase {
                ParsingPhase::Code(ref mut highlighter) => {
                    let ranges = highlighter.highlight(&text, &PARSE_CONTEXT.syntax_set);
                    let h = styled_line_to_highlighted_html(&ranges[..], IncludeBackground::Yes);
                    Event::Html(CowStr::Boxed(h.into_boxed_str()))
                }
                ParsingPhase::Header(ref mut h) => {
                    h.push_str(&text);
                    Event::Text(text)
                }
                ParsingPhase::Normal => Event::Text(text),
            },
            Event::Start(Tag::Heading(level)) => {
                let level = level as i32;
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
                        link: String::new(),
                        title: String::new(),
                        level,
                    })
                    .node_id();
                phase = ParsingPhase::Header(String::new());
                event
            }
            Event::End(Tag::Heading(_)) => {
                phase = match phase {
                    ParsingPhase::Header(ref h) => {
                        toc.0.get_mut(cur_section).unwrap().data().title = h.clone();
                        toc.0.get_mut(cur_section).unwrap().data().link = slugify(h);
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
        content: out,
    }
}
