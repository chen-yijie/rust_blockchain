mod api;
mod blockchain;

use std::env;
use std::sync::Mutex;
use actix_web::{ web, App, HttpServer };

#[actix_rt::main]
async fn main() -> std::io::Result<()> {

    // --p 指定监听端口
    let args: Vec<String> = env::args().collect();

    let port = match args.as_slice() {
        [_, key , value] => {
            if key == "--p" {
                value
            }else {
                panic!( "Illegal arguments passed to the program." );
            }
        }
        _ => "5000",
    };

    println!( "listen port: {}", port );

    // Rust 线程之间共享数据需要包装
    let sharedchain = web::Data::new( Mutex::new( blockchain::Blockchain::new() ) );

    let node_identifier = web::Data::new( uuid::Uuid::new_v4().to_simple().to_string() );

    HttpServer::new( move || {
        App::new()
            .app_data( sharedchain.clone() )
            .app_data( node_identifier.clone() )
            .data( web::JsonConfig::default().limit( 4096 ) )
            .service( web::resource( "/mine" ).route( web::get().to( api::mine ) ) )
            .service( web::resource( "/chain" ).route( web::get().to( api::chain ) ) )
            .service( web::resource( "/transactions/new" ).route(web::post().to(api::new_transaction ) ) )
            .service( web::resource( "/nodes/register" ).route(web::post().to(api::register_node ) ) )
            .service( web::resource( "/nodes/resolve" ).route(web::get().to(api::resolve_nodes ) ) )
    } )
    .bind( format!( "127.0.0.1:{}", port ) )?
    .run()
    .await
}
