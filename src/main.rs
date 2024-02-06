use std::collections::HashMap;

use ethers::core::types::{U64,H256,Address,Filter};
use ethers::providers::{Http, Middleware, Provider};
use ethers::prelude::abigen;
use std::sync::Arc;

use eyre::Result;
use std::fmt;
use clap::Parser;

extern crate tokio;

const ASCII_ART: &str = r#"
        ██████   ██████  ██      ███████     ██████  ███████ ████████ ██████  ██ ███████ ██    ██ ███████ ██████  
        ██   ██ ██    ██ ██      ██          ██   ██ ██         ██    ██   ██ ██ ██      ██    ██ ██      ██   ██ 
        ██████  ██    ██ ██      █████       ██████  █████      ██    ██████  ██ █████   ██    ██ █████   ██████  
        ██   ██ ██    ██ ██      ██          ██   ██ ██         ██    ██   ██ ██ ██       ██  ██  ██      ██   ██ 
        ██   ██  ██████  ███████ ███████     ██   ██ ███████    ██    ██   ██ ██ ███████   ████   ███████ ██   ██ 
by @0xTraub              
"#;

struct UserHasRole {
    pub address: Address,
    pub block_num: U64,
    pub tx_hash: H256
}

#[derive(Parser,Default,Debug)]
struct Arguments {
    #[clap(short, long)]
    pub rpc_url: Option<String>,
    
    #[clap(short, long)]
    pub contract: Address,

    #[clap(short, long)]
    pub starting_block: Option<U64>
}

//Custom Debug Formatter for the Struct UserHasRole
impl fmt::Debug for UserHasRole {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "User: {:?} granted role at block: {:?}\nTxHash: {:?}\n", self.address, self.block_num, self.tx_hash)
    }
}

fn display_role_info(role: &H256, admin_role: &H256, users: &Vec<UserHasRole>) {
    let default_admin_role = H256::zero();
    if role.eq(&default_admin_role) {
        println!("--- DEFAULT ADMIN ROLE: {:?}", Address::zero());
    }
    else {
        println!("--- Role: {:?}", role);
    }

    println!("Admin Role: {:?}", admin_role);

    for user in users {
        println!("{:?}", user);
    }
}

async fn get_provider()-> Result<(Provider::<Http>, U64, Address, U64)> {
    let fallback_rpc_url: String = String::from("https://eth.llamarpc.com");
    let fallback_block_range: i32 = 99999;

    //Try to create provider and unwrap otherwise panic
    let args = Arguments::parse();

    let rpc_url = match args.rpc_url {
        Some(url) => url,
        None => fallback_rpc_url.clone()
    };   

    //Create the Provider Object
    let provider: Provider<Http> = Provider::try_from(&rpc_url)?;

    let current_block = provider.get_block_number().await?;
    let block_number;
    //If using the fallback RPC provider, calculate block range
    if rpc_url.eq(&fallback_rpc_url) {
        //Get within the 100k block range
        block_number = current_block - fallback_block_range;
    }

    //Otherwise unwrap the user provided value if exists or make block-range start from zero
    else {
        block_number = match args.starting_block {
            Some(num) => num,
            None => U64::zero()
        };
    }

    Ok((provider, block_number, args.contract, current_block))
}

async fn get_role_admin(provider: Provider<Http>, addr: &Address, role: &H256) -> Result<H256> {

    abigen!(
        AccessControl,
        r#"[
            function getRoleAdmin(bytes32) external view returns (bytes32)
        ]"#,
    );

    let client = Arc::new(provider);
    let contract = AccessControl::new(*addr, client);

    let role_admin = contract.get_role_admin(*role.as_fixed_bytes()).call().await?;
    let _admin: H256 = role_admin.into();

    Ok(_admin)

}

#[tokio::main]
async fn main() -> Result<()> {
    println!("{}", ASCII_ART); 
    let mut roles: Vec<H256> = Vec::new();
    let mut role_users: HashMap<H256, Vec<UserHasRole>> = HashMap::new();
    let mut role_admins: HashMap<H256, H256> = HashMap::new();

    // let contract_address: &str = "0x853d955aCEf822Db058eb8505911ED77F175b99e";
    let add_role_event_name = String::from("RoleGranted(bytes32,address,address)");
    let remove_role_event_name = String::from("RoleRevoked(bytes32,address,address)");

    //Event Sigs for add and remove
    let role_added_sig: H256 = "0x2f8788117e7eff1d82e926ec794901d17c78024a50270940304540a733656f0d".parse::<H256>()?;
    let role_removed_sig: H256 = "0x04f4ba83d654385553482c5bc933c544b42dcbf063cbb948f438e89a646b4ed5".parse::<H256>()?;

    let (provider, block_num, address, current_block) = get_provider().await?;

    let chain_id = &provider.get_chainid().await?;
    let chain_name: String = match chain_id.as_u64() {
        1 => "Ethereum".to_string(),
        10 => "Optimism".to_string(),
        137 => "Polygon".to_string(),
        42161 => "Arbitrum".to_string(),
        43114 => "Avalanche".to_string(),
        _ => chain_id.to_string()
    };

    println!("SCANNING Contract {:?} on chain {:?} starting at block {:?}\n", address, chain_name, block_num);

    //Call the Address struct's funcstion "parse" and parse the string slice
    let add_role_filter = Filter::new()
        .address(address)
        .event(&add_role_event_name)
        .from_block(block_num);
    
    let remove_role_filter = Filter::new()
        .address(address)
        .event(&remove_role_event_name)
        .from_block(block_num);

    let mut logs = provider.get_logs(&add_role_filter).await?;
    let mut remove_role_logs = provider.get_logs(&remove_role_filter).await?;

    //Add all logs to the same vector of logs
    logs.append(&mut remove_role_logs);

    //Sort based on Block Number
    logs.sort_by(|a, b| a.block_number.unwrap().as_u64().cmp(&b.block_number.unwrap().as_u64()));

    //Pass the reference into example_log
    for log in logs {
        //If the event sig matches the role added signature
        //Because H256 implements EQ you can use the eq function which returns a bool
        if log.topics[0].eq(&role_added_sig) {
            let role_name = log.topics.get(1).unwrap();
            // println!("Role Name: {:?}", role_name);

            if roles.contains(role_name) {
                // println!("role already exists");

                let user_granted_role = UserHasRole {
                    address: Address::from(log.topics[2]),
                    block_num: log.block_number.unwrap(),
                    tx_hash: log.transaction_hash.unwrap()
                };
        
                //Get a mutable reference to the hashmap list and add the new user to the vector
                role_users.get_mut(role_name).unwrap()
                    .push(user_granted_role);
            }
        
            else {
                    roles.push(*role_name);//Push the list of roles onto the vec by dereferencing the borrows
                
                //Create a new vector so that we can push to it going forward
                role_users.insert(*role_name, Vec::new());

                let role_admin: H256 = get_role_admin(provider.clone(), &address, role_name).await?;
                role_admins.insert(*role_name, role_admin);
            }
        }

        else if log.topics.get(0).unwrap().eq(&role_removed_sig) {
            let role_name = log.topics.get(0).unwrap();
            let addr_long = log.topics.get(2).unwrap();
            let user = Address::from(*addr_long);

            role_users.get_mut(&role_name)
                .unwrap().retain(|a| a.address.ne(&user));
        }
    }

    for role in &roles {
        display_role_info(&role, role_admins.get(&role).unwrap(), role_users.get(&role).unwrap())
    }

    //If no events were found in the last n-blocks then print and exit.
    if roles.len() == 0 {
        println!("No Roles Granted on contract{:?} between blocks {:?} and {:?}", address, block_num, current_block);
        println!("Either expand range of blocks to check or try a different contract");
    }

    Ok(())
}