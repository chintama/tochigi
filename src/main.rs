use actix::{Actor, StreamHandler};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use log::*;
use structopt::StructOpt;

struct Geometry;

impl Actor for Geometry {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<ws::Message, ws::ProtocolError> for Geometry {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Text(text) => ctx.text(text),
            ws::Message::Binary(bin) => ctx.binary(bin),
            _ => (),
        }
    }
}

fn index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(Geometry {}, &req, stream);
    info!("{:?}", resp);
    resp
}

#[derive(StructOpt)]
struct Opt {
    #[structopt(name = "bind", default_value = "127.0.0.1:8080")]
    bind: String,
}

fn main() {
    env_logger::init();

    let opt = Opt::from_args();

    HttpServer::new(|| App::new().route("/ws/", web::get().to(index)))
        .bind(&opt.bind)
        .unwrap()
        .run()
        .unwrap();
}
