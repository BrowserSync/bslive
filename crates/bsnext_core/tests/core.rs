use actix::Actor;
use bsnext_core::server::actor::ServerActor;
use bsnext_core::server::handler_listen::Listen;
use bsnext_core::servers_supervisor::get_servers_handler::GetServersMessage;
use bsnext_dto::GetServersMessageResponse;
use bsnext_input::route::{Route, RouteKind};
use bsnext_input::server_config::{ServerConfig, ServerIdentity};

async fn system_test_01() {
    let c = ServerConfig {
        identity: ServerIdentity::named(),
        routes: vec![Route {
            path: "/".to_string(),
            kind: RouteKind::Html {
                html: "hello world!".to_string(),
            },
            ..Default::default()
        }],
        watchers: vec![],
    };

    struct Parent {}
    impl actix::Actor for Parent {
        type Context = actix::Context<Self>;
    }
    impl actix::Handler<GetServersMessage> for Parent {
        type Result = GetServersMessageResponse;

        fn handle(&mut self, _msg: GetServersMessage, _ctx: &mut Self::Context) -> Self::Result {
            todo!("woop!")
        }
    }

    let p = Parent {};
    let p = p.start();
    let s = ServerActor::new_from_config(c).start();

    let a = s
        .send(Listen {
            parent: p.recipient(),
        })
        .await;

    match a {
        Ok(Ok(addr)) => {
            assert!(addr.is_ipv4())
        }
        Ok(Err(e)) => {
            unreachable!("{:?}", e)
        }
        Err(e) => {
            unreachable!("{:?}", e)
        }
    }
}

#[actix_rt::test]
async fn test_init() {
    system_test_01().await
}
