use actix::{Actor, StreamHandler};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use gunma::{components::*, protocol::*};
use log::*;
use structopt::StructOpt;

type Context = ws::WebsocketContext<Geometry>;

struct Geometry;

struct Sender<'a> {
    ctx: &'a mut Context,
}

impl<'a> Sender<'a> {
    fn new(ctx: &'a mut Context) -> Self {
        Self { ctx }
    }

    fn send(&mut self, msg: Message) {
        info!("Sending {:?}", msg);

        match serde_json::to_vec(&msg) {
            Ok(bin) => {
                let _ = self.ctx.binary(bin);
            }
            Err(e) => error!("Coudln't send message: {}: {:?}", e, msg),
        }
    }
}

impl Actor for Geometry {
    type Context = Context;
}

impl StreamHandler<ws::Message, ws::ProtocolError> for Geometry {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Binary(bin) => {
                let msg = match serde_json::from_slice(&bin) {
                    Ok(msg) => msg,
                    Err(e) => return warn!("Couldn't parse message: {}", e),
                };

                let sender = Sender::new(ctx);
                handler(sender, msg);
            }
            msg => warn!("Invalid message: {:?}", msg),
        }
    }
}

fn handler(mut sender: Sender, msg: Message) {
    match msg {
        Message::GetAllTerrain => {
            use rand::prelude::*;

            info!("Got terrain request");

            for i in 0..100 {
                let x = i as f32;

                sender.send(Message::Terrain(Terrain {
                    id: 0,
                    asset: Asset(200),
                    pos: Pos::new(x * 1000.0, 0.0),
                    size: Size::new(1000.0, 100.0),
                }));
            }

            for _ in 0..100 {
                let mut rng = rand::thread_rng();
                let px: f32 = rng.gen_range(-1000.0, 5000.0);
                let py: f32 = rng.gen_range(-2000.0, 100.0);
                let sx: f32 = rng.gen_range(50.0, 100.0);
                let sy: f32 = rng.gen_range(50.0, 100.0);

                sender.send(Message::Terrain(Terrain {
                    id: 0,
                    asset: Asset(200),
                    pos: Pos::new(px, py),
                    size: Size::new(sx, sy),
                }));
            }

            sender.send(Message::EndTerrain);
        }
        msg => warn!("Received unsupported request: {:?}", msg),
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
