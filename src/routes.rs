use super::{errors, md2html, page, WikiState};
use axum::{
    body::{Bytes, Full},
    extract::{Path, State, TypedHeader},
    headers::{Header, HeaderName, HeaderValue},
    response::{Html, IntoResponse, Response},
    Json,
};
use minijinja::context;
use std::sync::Arc;

type Result<T> = std::result::Result<T, errors::AppError>;

#[derive(Debug)]
pub struct User(String);

static USER_HEADER_NAME: HeaderName = HeaderName::from_static("x-forwarded-user");
impl Header for User {
    fn name() -> &'static HeaderName {
        &USER_HEADER_NAME
    }
    fn decode<'i, I>(values: &mut I) -> std::result::Result<Self, axum::headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        let user = std::env::var("WIKIMARK_USER").ok();
        if let Some(u) = user {
            return Ok(User(u.to_string()));
        }
        let value = values.next().ok_or_else(axum::headers::Error::invalid)?;
        Ok(User(
            value
                .to_str()
                .map_err(|_| axum::headers::Error::invalid())?
                .to_owned(),
        ))
    }
    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        let v = HeaderValue::from_str(&self.0).expect("Username is invalid as header value");
        values.extend(std::iter::once(v));
    }
}

/// A CSS response.
///
/// Will automatically get `Content-Type: text/css`.
#[derive(Clone, Copy, Debug)]
pub struct Css<T>(pub T);

impl<T> IntoResponse for Css<T>
where
    T: Into<Full<Bytes>>,
{
    fn into_response(self) -> Response {
        (
            [(
                http::header::CONTENT_TYPE,
                HeaderValue::from_static("text/css"),
            )],
            self.0.into(),
        )
            .into_response()
    }
}

impl<T> From<T> for Css<T> {
    fn from(inner: T) -> Self {
        Self(inner)
    }
}

type UserHeader = TypedHeader<User>;

pub async fn index(State(state): State<Arc<WikiState>>, user: Option<UserHeader>) -> Html<String> {
    let templ = state.env.get_template("index.html").unwrap();
    let user_str = user.as_ref().map(|u| u.0 .0.as_str());
    Html(templ.render(context!( user => user_str )).unwrap())
}

pub async fn page(
    State(state): State<Arc<WikiState>>,
    user: Option<UserHeader>,
    Path(fname): Path<String>,
) -> Result<Html<String>> {
    let templ = state.env.get_template("page.html").unwrap();
    let user_str = user.as_ref().map(|u| u.0 .0.as_str());
    let md = state.repo.page_getter(&fname)?;
    let page = md2html::parse(&md.content, &md.meta);
    Ok(Html(templ.render(context!(
        user => user_str,
        toc => page.toc,
        meta => md.meta,
        content => page.content,
    ))?))
}

pub async fn pages(
    State(state): State<Arc<WikiState>>,
    user: Option<UserHeader>,
) -> Result<Html<String>> {
    let templ = state.env.get_template("pages.html").unwrap();
    let user_str = user.as_ref().map(|u| u.0 .0.as_str());
    let pages = state.repo.list_files("")?;
    Ok(Html(templ.render(context!(
        user => user_str,
        pages,
    ))?))
}

pub async fn changelog(
    State(state): State<Arc<WikiState>>,
    user: Option<UserHeader>,
) -> Result<Html<String>> {
    let templ = state.env.get_template("changelog.html").unwrap();
    let user_str = user.as_ref().map(|u| u.0 .0.as_str());
    Ok(Html(templ.render(context!(
        user => user_str,
        log => state.repo.get_log()?,
        commit_url_prefix => state.commit_url_prefix,
    ))?))
}

pub async fn edit(State(state): State<Arc<WikiState>>, user: UserHeader) -> Result<Html<String>> {
    let templ = state.env.get_template("edit.html").unwrap();
    let user_str = user.0 .0.as_str();
    Ok(Html(templ.render(context!(
        user => user_str,
    ))?))
}

pub async fn md(
    State(state): State<Arc<WikiState>>,
    Path(page): Path<String>,
) -> Result<Json<page::RawPage>> {
    let ret = state.repo.page_getter(&page)?;
    Ok(Json(ret))
}

pub async fn commit(
    State(state): State<Arc<WikiState>>,
    user: UserHeader,
    Json(info): Json<page::RawPage>,
) -> Result<String> {
    let ret = state.repo.page_committer(user.0 .0, info)?;
    Ok(ret)
}

pub async fn css() -> Css<String> {
    Css(super::CSS.to_owned())
}

pub async fn assets(Path(path): Path<String>) -> Result<Response> {
    if let Some(f) = super::STATIC_ASSETS.get_file(&path) {
        let mime = mime_guess::from_path(&path).first_or_octet_stream();
        Ok((
            [(
                http::header::CONTENT_TYPE,
                HeaderValue::from_str(mime.essence_str())?,
            )],
            f.contents(),
        )
            .into_response())
    } else {
        Ok(http::StatusCode::NOT_FOUND.into_response())
    }
}
