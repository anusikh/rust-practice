mod lobby;
mod messages;
mod routes;

use std::time::{Duration, Instant};

use actix::{
    dev::ContextFutureSpawner, fut, Actor, ActorContext, ActorFuture, Addr, AsyncContext, Handler,
    StreamHandler, WrapFuture,
};
use actix_web::{App, HttpServer};
use actix_web_actors::ws;
use actix_web_actors::ws::Message::Text;
use lobby::Lobby;
use messages::{ClientActorMessage, Connect, Disconnect, WsMessage};
use routes::start_connection;
use uuid::Uuid;

// Every ws connection is an actor and Lobby itself is an actor

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

struct WsConn {
    room: Uuid,
    lobby_addr: Addr<Lobby>,
    hb: Instant,
    id: Uuid,
}

impl WsConn {
    pub fn new(room: Uuid, lobby: Addr<Lobby>) -> WsConn {
        WsConn {
            id: Uuid::new_v4(),
            room,
            hb: Instant::now(),
            lobby_addr: lobby,
        }
    }

    // handling heartbeat, pinging the client ,disconnect if response doesn't come
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(
            HEARTBEAT_INTERVAL,
            |act, ctx: &mut ws::WebsocketContext<WsConn>| {
                if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                    println!("disconnecting since heartbeat failed");
                    act.lobby_addr.do_send(Disconnect {
                        room_id: act.room,
                        id: act.id,
                    });
                    ctx.stop();
                    return;
                }

                ctx.ping(b"PING");
            },
        );
    }
}

impl Actor for WsConn {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);

        let addr = ctx.address();
        self.lobby_addr
            .send(Connect {
                addr: addr.recipient(),
                lobby_id: self.room,
                self_id: self.id,
            })
            .into_actor(self)
            .then(|res, _, ctx| {
                match res {
                    Ok(_res) => (),
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> actix::prelude::Running {
        self.lobby_addr.do_send(Disconnect {
            id: self.id,
            room_id: self.room,
        });
        actix::prelude::Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsConn {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            Ok(ws::Message::Continuation(_)) => {
                ctx.stop();
            }
            Ok(ws::Message::Nop) => (),
            Ok(Text(s)) => self.lobby_addr.do_send(ClientActorMessage {
                id: self.id,
                msg: s,
                room_id: self.room,
            }),
            Err(e) => panic!("{}", e),
        }
    }
}

// sending text to the actor's mailbox
impl Handler<WsMessage> for WsConn {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(msg.0);
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let chat_server = Lobby::default().start(); // create and spin up a lobby

    HttpServer::new(move || {
        App::new()
            .service(start_connection) // rename with "as" import or naming conflict
            .data(chat_server.clone()) // register the lobby
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
