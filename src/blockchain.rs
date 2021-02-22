use crate::api::Chain;
use chrono::{ DateTime, Utc };
use crypto_hash::{hex_digest, Algorithm};
use serde::{ Deserialize, Serialize };
use std::collections::HashSet;
use urlparse::urlparse;
use reqwest;

#[derive( Clone, Hash, Serialize, Deserialize, Debug )]
pub struct Transaction {
    sender: String,                             // 发送者
    recipient: String,                          // 接受者
    amount: i64,                                // 交易数量
}


#[derive( Clone, Hash, Serialize, Deserialize, Debug )]
pub struct Block {
    pub index: u64,                             // 区块高度
    timestamp: DateTime<Utc>,                   // 时间戳
    pub transactions: Vec<Transaction>,         // 交易
    pub proof: u64,                             // 证明
    pub previous_hash: String,                  // 上一个区块的哈希地址
}


#[derive( Default, Debug )]
pub struct Blockchain {
    pub chain: Vec<Block>,                      // 区块链账本
    current_transactions: Vec<Transaction>,     // 交易集合
    pub nodes: HashSet<String>,                     // 节点集合
}

impl Blockchain {
    pub fn new() -> Blockchain {
        let mut blockchain = Blockchain {
            chain: vec![],
            current_transactions: vec![],
            nodes: HashSet::new(),
        };

        blockchain.new_block( 100, Some( "1" ) );
        blockchain
    }

    /// Create a new Block in the Blockchain
    ///
    /// :param proof: The proof given by the Proof of Work algorithm
    /// :param previous_hash: (Optional) hash of previous Block
    /// :return: New Bloc
    /// 创建新区块
    pub fn new_block( &mut self, proof: u64, previous_hash: Option<&str> ) -> Block {
        let block = Block {
            index: ( self.chain.len() + 1 ) as u64,
            timestamp: Utc::now(),
            transactions: self.current_transactions.drain( 0 .. ).collect(),
            proof,
            previous_hash: previous_hash.unwrap_or( "0" ).to_string(),
        };

        self.chain.push( block.clone() );
        block
    }

    /// Returns the last Block in the chain
    /// 返回最后一个区块
    pub fn last_block( &self ) -> Option<&Block> {
        self.chain.last()
    }

    /// Simple Proof of Work Algorithm:
    /// - Find a number p' such that hash(pp') contains 4 leading zeroes,
    ///   where p is the previous proof, and p' is the new proof
    /// POW工作量证明共识机制算法
    pub fn proof_of_work( last_block: &Block ) -> u64 {
        let mut proof = 0;
        let last_proof = last_block.proof;
        let last_hash = &last_block.previous_hash;

        while !Self::valid_proof( last_proof, proof, last_hash ) {
            proof += 1;
        }
        proof
    }

    /// Creates a SHA-256 hash of a Block
    ///
    /// :param block: Block
    /// :return hash for the block
    /// 创建一个区块 的哈希值，基SHA-256算法
    pub fn hash( block: &Block ) -> String {
        let serialized = serde_json::to_string( &block ).unwrap();
        hex_digest( Algorithm::SHA256, serialized.as_bytes() )
    }

    /// Validates the Proof: Does hash(last_proof, proof, last_hash) containt 4 leading zeroes
    /// 验证工作证明数字
    fn valid_proof( last_proof: u64, proof: u64, last_hash: &String ) -> bool {
        let guess = format!( "{}{}{}", last_proof, proof, last_hash );
        let guess_hash = hex_digest( Algorithm::SHA256, guess.as_bytes() );
        guess_hash.ends_with( "000" ) //困难度为3
    }

    /// Creates a new transaction to go into the next mined Block
    ///
    /// :param sender: Address of the Å›ender
    /// :param recipient: Address of the recipient
    /// :param amount: Amount
    /// :return: The index of the Block that will hold this transaction
    /// 发起一个新交易，将写入下一个区块
    pub fn new_transaction( &mut self, sender: &str, recipient: &str, amount: i64 ) -> u64 {
        self.current_transactions.push( Transaction {
            sender: sender.to_string(),
            recipient: recipient.to_string(),
            amount,
        } );
        self.last_block().unwrap().index + 1
    }

    /// Add a new node to the list of nodes
    ///
    /// :param address: Address of the node. Eg. 'http://192.168.0.5:5000'
    ///
    /// 节点注册，即新节点加入区块链网络,注册地址参数为节点服务器地址，如：'http://192.168.0.5:5000‘
    pub fn register_node( &mut self, address: &str ) {
        let parsed_url = urlparse( address );
        self.nodes.insert( parsed_url.netloc );
    }

    /// Determine if a given blockchain is valid
    /// 链的验证
    fn valid_chain( &self, chain: &[Block] ) -> bool {
        let mut last_block = &chain[0];
        let mut current_index: usize = 1;
        while current_index < chain.len() {
            let block = &chain[current_index];
            println!( "{:?}", last_block );
            println!( "{:?}", block );
            println!( "-----------" );
            if block.previous_hash != Blockchain::hash( last_block ) {
                return false;
            }
            if !Blockchain::valid_proof( last_block.proof, block.proof, &last_block.previous_hash ) {
                return false;
            }

            last_block = block;
            current_index += 1;
        }
        true
    }

    /// This is our Consensus Algorithm, it resolves conflicts
    /// by replacing our chain with the longest one in the network.
    ///
    /// :return True if our chain was replaced and false otherwise
    /// 最长链原则处理逻辑，即共识机制为（POw+最长链原则）
    pub fn resolve_conflicts( &mut self ) -> bool {
        let mut max_length = self.chain.len();
        let mut new_chain: Option<Vec<Block>> = None;

        // Grab and verify the chains from all the nodes in our network
        for node in &self.nodes {
            let mut response = reqwest::get( &format!( "http://{}/chain", node ) ).unwrap();
            if response.status().is_success() {
                let node_chain: Chain = response.json().unwrap();
                if node_chain.length > max_length && self.valid_chain( &node_chain.chain ) {
                    max_length = node_chain.length;
                    new_chain = Some( node_chain.chain );
                }
            }
        }
        // Replace our chain if we discovered a new, valid chain longer than ours
        match new_chain {
            Some( x ) => {
                self.chain = x;
                true
            }
            None => false,
        }
    }
}
