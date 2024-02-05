use std::env;
use std::collections::HashMap;
use ethers::prelude::*;
use eyre::Result;

extern crate tokio;

struct UserHasRole {
    pub address: Address,
    pub block_num: U64
}

//TODO: Implement Role with Debug

#[tokio::main]
async fn main() -> Result<()> {
    let mut roles: Vec<H256> = Vec::new();
    let mut role_users: HashMap<H256, Vec<UserHasRole>> = HashMap::new();

    let contract_address: &str = "0x3C2982CA260e870eee70c423818010DfeF212659";
    let add_role_event_name = String::from("RoleGranted(bytes32,address,address)");
    let remove_role_event_name = String::from("RoleRevoked(bytes32,address,address)");

    //Event Sigs for add and remove
    let role_added_sig: H256 = "0x2f8788117e7eff1d82e926ec794901d17c78024a50270940304540a733656f0d".parse::<H256>()?;
    let role_removed_sig: H256 = "0x04f4ba83d654385553482c5bc933c544b42dcbf063cbb948f438e89a646b4ed5".parse::<H256>()?;

    //get the rpc url from the env or panic  
    // let rpc_url = env::var("ETH_RPC_URL")?;
    let rpc_url = String::from("https://eth-mainnet.alchemyapi.io/v2/zOVFUzSEld1v_MuTOqGPYkTYttwBUrmF");

    //Try to create provider and unwrap otherwise panic
    let provider = Provider::try_from(&rpc_url)?;

    //Call the Address struct's funcstion "parse" and parse the string slice
    let address = contract_address.parse::<Address>().unwrap();
    let add_role_filter = Filter::new()
        .address(address)
        .event(&add_role_event_name)
        .from_block(0);
    
    let remove_role_filter = Filter::new()
        .address(address)
        .event(&remove_role_event_name)
        .from_block(0);

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
        if log.topics.get(0).unwrap().eq(&role_added_sig) {
            let role_name = &log.topics.get(1).unwrap();
            println!("Role Name: {:?}", role_name);

            if roles.contains(role_name) {
                println!("role already exists");

                let user_granted_role = UserHasRole {
                    address: log.address,
                    block_num: log.block_number.unwrap()
                };
        
                //Get a mutable reference to the hashmap list and add the new user to the vector
                role_users.get_mut(role_name).unwrap()
                    .push(user_granted_role);
            }
        
            else {
                println!("new role");
                roles.push(**role_name);//Push the list of roles onto the vec by dereferencing the borrows
                
                //Create a new vector so that we can push to it going forward
                role_users.insert(**role_name, Vec::new());
            }
        }

        else if log.topics.get(0).unwrap().eq(&role_removed_sig) {
            let role_name = log.topics.get(0).unwrap();
            let addr_long = log.topics.get(2).unwrap();
            let user = Address::from(*addr_long);

            let mut addr_vector = role_users.get_mut(&role_name)
                .unwrap().retain(|a| a.address.ne(&user));
           
            // println!("Removing Role: {:?}", )
        }
        
    }

    // println!("{:?}", logs);

    Ok(())

}
