use actix_web::{dev::AsyncResult, Error, FromRequest, HttpRequest};

use log::info;

pub struct Identity {
    pub name: String,
}
impl<S> FromRequest<S> for Identity {
    type Config = ();
    type Result = AsyncResult<Identity, Error>;

    #[inline]
    fn from_request(req: &HttpRequest<S>, _: &Self::Config) -> Self::Result {
        //let name = match req.headers()
        //    .get("X-Forwarded-User")
        //    .ok_or_else(|| error::ErrorInternalServerError("Error querying channels"))
        //{
        //    Ok(n) => n,
        //    Err(e) => return AsyncResult::err(e),
        //};
        //let name = match name.to_str().map_err(|_| {
        //    error::ErrorInternalServerError("Header value contains invalid characters")
        //}) {
        //    Ok(n) => n.to_owned(),
        //    Err(e) => return AsyncResult::err(e),
        //};
        let name = "yuri".to_owned();
        info!("X-Forwarded-User: {}", name);
        AsyncResult::ok(Identity { name })
    }
}
