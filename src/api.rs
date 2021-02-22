use crate::blockchain::{Block, Blockchain, Transaction};

use std::sync::Mutex;
use actix_web::{ web, HttpRequest, HttpResponse };
use serde::{Deserialize, Serialize};

/// 挖矿响应消息
#[derive( Serialize )]
pub struct MiningRespose {
    message: String,
    index: u64,
    transactions: Vec<Transaction>,
    proof: u64,
    previous_hash: String,
}

//交易请求信息
#[derive( Serialize, Deserialize )]
pub struct TransactionRequest {
    sender: String,
    recipient: String,
    amount: i64,
}

///返回消息体
#[derive( Serialize, Deserialize )]
pub struct MessageResponse {
    message: String,
}

///链结构体，代表现在网络上的最长链
#[derive( Serialize, Deserialize )]
pub struct Chain {
    pub chain: Vec<Block>,
    pub length: usize,
}

///节点注册请求信息
#[derive( Deserialize )]
pub struct RegisterRequest {
    nodes: Vec<String>,
}

///节点注册响应信息
#[derive( Serialize )]
pub struct RegisterResponse {
    message: String,
    total_nodes: Vec<String>,
}
//解决冲突响应信息
#[derive( Serialize )]
pub struct ResolveResponse {
    message: String,
    chain: Vec<Block>,
}

///发起新交易
pub fn new_transaction(
    state: web::Data<Mutex<Blockchain>>,
    req: web::Json<TransactionRequest>,
) -> HttpResponse {
    let sender = req.sender.to_owned();
    let recipient = req.recipient.to_owned();
    let index = state
        .lock()
        .unwrap()
        .new_transaction( &sender, &recipient, req.amount );
    HttpResponse::Created().json( MessageResponse {
        message: format! {"Transaction will be added to Block {}", index},
    } )
}

/// 矿工挖矿
pub fn mine(
    node_identifier: web::Data<String>,
    state: web::Data<Mutex<Blockchain>>,
    _req: HttpRequest,
) -> HttpResponse {
    let ( proof, previous_hash ) = {
        let blockchain = state.lock().unwrap();
        let last_block = blockchain.last_block().unwrap();
        let proof = Blockchain::proof_of_work( &last_block );
        let previous_hash = Blockchain::hash( last_block );
        ( proof, previous_hash )
    };

    let mut blockchain = state.lock().unwrap();
    blockchain.new_transaction( "0", &*node_identifier, 1 );
    let block = blockchain.new_block( proof, Some( &previous_hash ) );
    HttpResponse::Ok().json( MiningRespose {
        message: "New Block Forged".to_string(),
        index: block.index,
        transactions: block.transactions,
        proof,
        previous_hash,
    } )
}

///当前最新链的信息
pub fn chain( state: web::Data<Mutex<Blockchain>>, _req: HttpRequest ) -> HttpResponse {
    let length = state.lock().unwrap().chain.len();
    HttpResponse::Ok().json( Chain {
        chain: state.lock().unwrap().chain.clone(),
        length,
    } )
}

///节点注册
pub fn register_node(
    state: web::Data<Mutex<Blockchain>>,
    req: web::Json<RegisterRequest>,
) -> HttpResponse {
    if req.nodes.is_empty() {
        return HttpResponse::BadRequest().json( MessageResponse {
            message: "Error: Please supply a valid list of nodes".to_string(),
        } );
    }
    let mut blockchain = state.lock().unwrap();
    for node in req.nodes.iter() {
        blockchain.register_node( node )
    }
    HttpResponse::Created().json( RegisterResponse {
        message: "New nodes have been added".to_string(),
        total_nodes: blockchain.nodes.iter().cloned().collect(),
    } )
}

///跟网络上其他节点达成共识，即解决冲突
pub fn resolve_nodes( state: web::Data<Mutex<Blockchain>>, _req: HttpRequest ) -> HttpResponse {
    let mut blockchain = state.lock().unwrap();
    let replaced = blockchain.resolve_conflicts();
    let message = if replaced {
        "Our chain was replaced"
    } else {
        "Our chain is authorative"
    };

    HttpResponse::Ok().json( ResolveResponse {
        message: message.to_string(),
        chain: blockchain.chain.clone(),
    } )
}