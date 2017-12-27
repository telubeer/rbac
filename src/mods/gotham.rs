use gotham::handler::NewHandlerService;
//use gotham_derive;
use gotham;
use hyper::server::Http;
use hyper::Method;

use gotham::handler::{NewHandler};
use gotham::middleware::pipeline::new_pipeline;

use gotham::router::route::{Extractors, Route, RouteImpl, Delegation};
use gotham::router::route::dispatch::{new_pipeline_set, finalize_pipeline_set, PipelineSet,
                                      PipelineHandleChain, DispatcherImpl};
use gotham::router::route::matcher::MethodOnlyRouteMatcher;
use gotham::router::request::path::NoopPathExtractor;
use gotham::router::request::query_string::NoopQueryStringExtractor;

use gotham::router::Router;
use gotham::router::tree::TreeBuilder;
use gotham::router::tree::node::{NodeBuilder, SegmentType};
use gotham::router::response::finalizer::ResponseFinalizerBuilder;

use gotham::state::{State/*, FromState, StateData*/};
use gotham::middleware::{NewMiddleware, Middleware};
use gotham::http::response::create_response;
use hyper::{StatusCode, Body};
use serde_json;
use serde_json::Value;
use gotham::handler::HandlerFuture;

use hyper::server::{Request, Response};
use hyper::{Chunk, Error};
use json::{parse, stringify, JsonValue};
use futures::stream::Stream;
use futures::Future;

use mime::{TEXT_JAVSCRIPT};
use std::result::Result;
use std::io::Result as IoResult;

use super::rbac::{Data, UserId};
use std::sync::{Arc, RwLock};

pub fn start(addr: String, data_arc: Arc<RwLock<Data>>) {
    let a = addr.parse().unwrap();
    let server = Http::new()
        .bind(&a, NewHandlerService::new(router(data_arc)))
        .unwrap();

    println!("Listening on http://{}", server.local_addr().unwrap());
    server.run().unwrap();
}

pub fn check(state: State, req: Request) -> (State, Response) {
    let response = {
        let data_arc = state.borrow::<Rbac>().unwrap_or_else(|| {
            panic!("data middleware fuckup")
        }).get().clone();

        let data = data_arc.read().unwrap();

        let items = parse_body(req.body()).unwrap();
        let mut out: JsonValue = array![];
        for item in items.as_array().unwrap().iter() {
            let user_id: UserId = item["user_id"].as_u64().unwrap() as u32;
            let action = &item["action"];
            let params = &item["params"];
            let mut res: JsonValue = array![];
            for param in params.as_array().unwrap().iter() {
                let pp = serde_json::to_string(param).unwrap();
                let v = parse(&pp).unwrap();
                let result = data.check_access(
                    user_id,
                    action.to_string(),
                    &v,
                );
                let _ = res.push(result);
            }
            let _ = out.push(res);
        }

        let response = create_response(
            &state,
            StatusCode::Ok,
            Some((stringify(out).into_bytes(), TEXT_JAVSCRIPT)),
        );
        response
    };

    (state, response)
}

fn parse_body(body: Body) -> Result<Value, Error> {
    body
        .concat2()
        .and_then(move |body: Chunk| {
            Ok(serde_json::from_slice(&body).unwrap())
        })
        .wait()
}

#[derive(StateData, Clone)]
pub struct Rbac {
    value: Arc<RwLock<Data>>
}

impl Rbac {
    pub fn get(&self) -> Arc<RwLock<Data>> {
        self.value.clone()
    }
}

struct MiddlewareWithStateData {
    data: Rbac
}

impl Middleware for MiddlewareWithStateData {
    fn call<Chain>(self, mut state: State, req: Request, chain: Chain) -> Box<HandlerFuture>
        where Chain: FnOnce(State, Request) -> Box<HandlerFuture> + Send + 'static
    {
        state.put(self.data);
        chain(state, req)
    }
}

impl NewMiddleware for MiddlewareWithStateData {
    type Instance = MiddlewareWithStateData;

    fn new_middleware(&self) -> IoResult<MiddlewareWithStateData> {
        Ok(MiddlewareWithStateData {
            data: self.data.clone()
        })
    }
}


pub fn router(data_arc: Arc<RwLock<Data>>) -> Router {
    // Start to build the Tree structure which our Router will rely on to match and dispatch
    // Requests entering our application.
    let mut tree_builder = TreeBuilder::new();

    // There is a single PipelineSet in use for this Router, which we refer to as global.
    // It utilises a single `Middleware` that helps the application maintain data between Requests
    // by using an in memory backend.
    //
    // Pipelines are very powerful and can be nested at different levels in your application.
    //
    // You can also assign multiple Middleware instances to a Pipeline each will be evaluated in
    // order of definition for each Request entering the system.
    let ps_builder = new_pipeline_set();
    let (ps_builder, global) = ps_builder.add(
        new_pipeline()
            .add(
                MiddlewareWithStateData {
                    data: Rbac {
                        value: data_arc
                    }
                }
            )
//               .add(
//                   NewSessionMiddleware::default()
//                       .insecure()
//                       .with_session_type::<Session>(),
//               )
            .build(),
    );
    let ps = finalize_pipeline_set(ps_builder);

    // Add a Route directly to the root of our `Tree` so that `Requests` for `/` are handled by
    // the `welcome` controller. Each function within the `welcome` controller represents a complete
    // `Handler` in Gotham parlance.
    /* tree_builder.add_route(
        static_route(
            vec![Method::Get, Method::Head], // Use this Route for Get and Head Requests
            || Ok(welcome::index),
            (global, ()), // This signifies that the active Pipelines for this route consist only of the global pipeline
            ps.clone(),
        )
    ); */
    // All the pipelines we've created for this Router

    // Create a Node to represent the Request path /check
    let mut check_node = NodeBuilder::new("check", SegmentType::Static);

    check_node.add_route(static_route(
        vec![Method::Post], // Use this Route for Post Requests
        || Ok(check),
        (global, ()),
        ps.clone(),
    ));


    // Add the todo node to the tree to complete this path
    tree_builder.add_child(check_node);

    let tree = tree_builder.finalize();

    let response_finalizer_builder = ResponseFinalizerBuilder::new();
    let response_finalizer = response_finalizer_builder.finalize();

    Router::new(tree, response_finalizer)
}

fn static_route<NH, P, C>(
    methods: Vec<Method>,
    new_handler: NH,
    active_pipelines: C,
    ps: PipelineSet<P>,
) -> Box<Route + Send + Sync>
    where
        NH: NewHandler + 'static,
        C: PipelineHandleChain<P> + Send + Sync + 'static,
        P: Send + Sync + 'static,
{
    // Requests must have used the specified method(s) in order for this Route to match.
    //
    // You could define your on RouteMatcher of course.. perhaps you'd like to only match on
    // requests that are made using the GET method and send a User-Agent header for a particular
    // version of browser you'd like to make fun of....
    let matcher = MethodOnlyRouteMatcher::new(methods);

    // For Requests that match this Route we'll dispatch them to new_handler via the pipelines
    // defined in active_pipelines.
    //
    // n.b. We also specify the set of all known pipelines in the application so the dispatcher can
    // resolve the pipeline references provided in active_pipelines. For this application that is
    // only the global pipeline.
    let dispatcher = DispatcherImpl::new(new_handler, active_pipelines, ps);
    let extractors: Extractors<NoopPathExtractor, NoopQueryStringExtractor> = Extractors::new();
    let route = RouteImpl::new(
        matcher,
        Box::new(dispatcher),
        extractors,
        Delegation::Internal,
    );
    Box::new(route)
}