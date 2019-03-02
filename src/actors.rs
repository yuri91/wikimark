use actix::{Actor, Handler, Message, SyncContext};
use actix_web::Error;
use juniper::http::GraphQLRequest;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GraphQLData(GraphQLRequest);

pub struct GraphQLDataMessage<MC> {
    pub message_context: MC,
    pub data: GraphQLData,
}

impl<C> Message for GraphQLDataMessage<C> {
    type Result = Result<String, Error>;
}

pub struct GraphQLExecutor<EC, Q, M>
where
    Q: juniper::GraphQLType + 'static,
    M: juniper::GraphQLType + 'static,
{
    pub executor_context: EC,
    pub schema: std::sync::Arc<juniper::RootNode<'static, Q, M>>,
}

impl<EC, Q, M> Actor for GraphQLExecutor<EC, Q, M>
where
    Q: juniper::GraphQLType + 'static,
    M: juniper::GraphQLType + 'static,
    EC: 'static,
{
    type Context = SyncContext<Self>;
}

pub struct QueryContext<EC, MC> {
    pub executor_context: EC,
    pub message_context: MC,
}

impl<EC, MC> juniper::Context for QueryContext<EC, MC> {}

impl<MC, EC, Q, M> Handler<GraphQLDataMessage<MC>> for GraphQLExecutor<EC, Q, M>
where
    EC: Clone + 'static,
    Q: juniper::GraphQLType<Context = QueryContext<EC, MC>> + 'static,
    M: juniper::GraphQLType<Context = QueryContext<EC, MC>> + 'static,
{
    type Result = Result<String, Error>;

    fn handle(&mut self, msg: GraphQLDataMessage<MC>, _: &mut Self::Context) -> Self::Result {
        let context = QueryContext {
            executor_context: self.executor_context.clone(),
            message_context: msg.message_context,
        };
        let res = msg.data.0.execute(&self.schema, &context);
        let res_text = serde_json::to_string(&res)?;
        Ok(res_text)
    }
}
