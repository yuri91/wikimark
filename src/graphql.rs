use juniper::graphql_object;
use juniper::FieldResult;
use juniper::RootNode;

use crate::git;

type Context = crate::actors::QueryContext<std::sync::Arc<git::Repo>, String>;

pub struct Query;
graphql_object!(Query: Context |&self| {
    field list(&executor) -> FieldResult<Vec<String>> {
        Ok(executor.context().executor_context.list("")?)
    }
    field read(&executor, path: String) -> FieldResult<String> {
        Ok(executor.context().executor_context.read(&path)?)
    }
});

pub struct Mutation;
graphql_object!(Mutation: Context | &self | {
    field commit(&executor, path: String, author: String, message: String, content: String) -> FieldResult<String> {
        Ok(executor.context().executor_context.commit(git::Commit {
            path: &path,
            author: &author,
            message: &message,
            content: &content,
        })?)
    }
});

pub type Schema = RootNode<'static, Query, Mutation>;

pub fn create_schema() -> Schema {
    Schema::new(Query, Mutation)
}
