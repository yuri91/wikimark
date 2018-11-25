use pulldown_cmark::{html, Event, Parser, Tag};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::html::{
    start_highlighted_html_snippet, styled_line_to_highlighted_html, IncludeBackground,
};
use syntect::parsing::{SyntaxReference, SyntaxSet};

use tera::Tera;

use std::borrow::Cow::{Borrowed, Owned};

fn get_syntax_for_block<'a>(set: &'a SyntaxSet, hint: &str) -> &'a SyntaxReference {
    set.find_syntax_by_name(hint).unwrap_or_else(|| {
        set.find_syntax_by_extension(hint)
            .unwrap_or_else(|| set.find_syntax_plain_text())
    })
}

#[derive(Serialize, Debug)]
struct Page {
    toc: Vec<Section>,
    slug: String,
    content: String,
    title: String,
}
#[derive(Serialize, Debug)]
struct Section {
    link: String,
    children: Vec<Section>,
}

enum ParsingPhase<'a> {
    Normal,
    Code(HighlightLines<'a>),
    Header(String),
}

struct State {
    parse_context: ParseContext,
    tera: Tera,
}
struct ParseContext {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}
impl ParseContext {
    fn new() -> ParseContext {
        ParseContext {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }
}

fn parse(path: &str, parse_context: &ParseContext) -> Page {
    let theme = &parse_context.theme_set.themes["base16-ocean.dark"];
    let md = std::fs::read_to_string(path).expect("md file not found");
    let parser = Parser::new(&md);
    let mut out = String::new();
    let mut phase = ParsingPhase::Normal;
    let mut toc = Vec::new();

    let parser = parser.map(|event| match event {
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
        _ => event,
    });
    html::push_html(&mut out, parser);
    Page {
        toc: toc,
        title: "test".to_owned(),
        slug: "test".to_owned(),
        content: out,
    }
}

fn render(page: &Page, tera: &Tera) -> tera::Result<String> {
    let mut ctx = tera::Context::new();
    ctx.insert("page", &page);
    tera.render("page.html", &ctx)
}

pub fn renderer(templates_path: &str) -> impl Fn(String) -> String + Clone {
    let tera = compile_templates!(templates_path);
    let state = std::sync::Arc::new(State {
        parse_context: ParseContext::new(),
        tera,
    });
    let md2html = move |fname: String| {
        let state = state.clone();
        let page = parse(&fname, &state.parse_context);
        render(&page, &state.tera).expect("template rendering failed")
    };
    md2html
}
