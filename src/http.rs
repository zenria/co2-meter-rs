use crate::datastore::{DataStore, DebugHistory};
use crate::sensor::mh_z19::MHZ19Response;
use actix::Addr;
use actix_web::server::HttpServer;
use actix_web::{App, HttpRequest, HttpResponse};
use futures::Future;

struct HttpAppState {
    mhz19_datastore: Addr<DataStore<MHZ19Response>>,
}

fn debug(
    req: &HttpRequest<HttpAppState>,
) -> Box<Future<Item = HttpResponse, Error = actix_web::Error>> {
    let s: Addr<DataStore<MHZ19Response>> = req.state().mhz19_datastore.clone();
    Box::new(
        s.send(DebugHistory)
            .map_err(|e| {
                error!("unable to get history {}", e);
                actix_web::Error::from(e)
            })
            .and_then(|history| Ok(HttpResponse::Ok().content_type("text/plain").body(history))),
    )
}
pub fn launch_http_server(bind_address: &str, mhz19_datastore: Addr<DataStore<MHZ19Response>>) {
    HttpServer::new(move || {
        App::with_state(HttpAppState {
            mhz19_datastore: mhz19_datastore.clone(),
        })
        .resource("/debug", |r| r.f(debug))
    })
    .workers(1)
    .bind(&bind_address)
    .unwrap()
    .start();
}
