use sails_rs::{calls::*, gtest::calls::*};

use nexus_vft_client::traits::*;
use gstd::collections::HashSet;

const ADMIN_ID: u64 = 42;
const NEW_ADMIN_ID: u64 = 43;
const NON_ADMIN_ID: u64 = 44;

const TOKEN_OWNER_ID_A: u64 = 45;
const TOKEN_OWNER_ID_B: u64 = 45;

const ACTOR_ID: u64 = 42;

use nexus_vft_client::{
    traits::{NexusVftFactory, NexusVft},
    NexusVftFactory as Factory, NexusVft as VftClient,
};

#[tokio::test]
async fn nexus_vft_works() {
    let remoting = GTestRemoting::new(ACTOR_ID.into());
    remoting.system().init_logger();

    // Submit program code into the system
    let program_code_id = remoting.system().submit_code(nexus_vft::WASM_BINARY);

    let program_factory = nexus_vft_client::NexusVftFactory::new(remoting.clone());

    let program_id = program_factory
        .initialize("TokenName".to_string(), "TokenSymbol".to_string(), 18, [(TOKEN_OWNER_ID_A.into(), 500000_000000000000000000u128.into()), (TOKEN_OWNER_ID_B.into(), 500000_000000000000000000u128.into())].to_vec()) // Call program's constructor (see src/lib.rs:27)
        .send_recv(program_code_id, b"salt")
        .await
        .unwrap();

    let mut service_client = nexus_vft_client::NexusVft::new(remoting.clone());

    // Test the `balance_of` method
    let balance = service_client
        .balance_of(TOKEN_OWNER_ID_A.into()) // Get the balance of ACTOR_ID
        .recv(program_id)
        .await
        .unwrap();

    assert_eq!(balance, 500000_000000000000000000u128.into());

    let balance = service_client
        .balance_of(TOKEN_OWNER_ID_B.into()) // Get the balance of ACTOR_ID
        .recv(program_id)
        .await
        .unwrap();

    assert_eq!(balance, 500000_000000000000000000u128.into());


    // Test the `mint` method
    let mint_result = service_client
        .mint(ACTOR_ID.into(), 100.into()) // Mint 100 tokens to ACTOR_ID
        .send_recv(program_id)
        .await
        .unwrap();

    assert!(mint_result);

    // Optionally, check the balance to verify minting worked
    let balance = service_client
        .balance_of(ACTOR_ID.into()) // Get the balance of ACTOR_ID
        .recv(program_id)
        .await
        .unwrap();

    assert_eq!(balance, 100.into());

    let balance_total = service_client
        .total_supply() // Get the total supply
        .recv(program_id)
        .await
        .unwrap();

    assert_eq!(balance_total, 500000_000000000000000100u128.into());
}


#[tokio::test]
async fn admin_minting_works() {
    let remoting = GTestRemoting::new(ADMIN_ID.into());
    remoting.system().init_logger();

    // Submit program code into the system
    let program_code_id = remoting.system().submit_code(nexus_vft::WASM_BINARY);

    let program_factory = nexus_vft_client::NexusVftFactory::new(remoting.clone());

    // Initialize with ADMIN_ID as the first admin
    let program_id = program_factory
        .initialize("TokenName".to_string(), "TokenSymbol".to_string(), 18, [].to_vec()) // Call program's constructor
        .send_recv(program_code_id, b"salt")
        .await
        .unwrap();

    let mut service_client = nexus_vft_client::NexusVft::new(remoting.clone());


    // Test the `mint` function as the initial admin
    let mint_result = service_client
        .mint(ADMIN_ID.into(), 100.into()) // Mint 100 tokens to ADMIN_ID
        .send_recv(program_id)
        .await
        .unwrap();
    assert!(mint_result);

    let balance = service_client
        .balance_of(ADMIN_ID.into()) // Get the balance of ADMIN_ID
        .recv(program_id)
        .await
        .unwrap();
    assert_eq!(balance, 100.into());

    let is_admin = service_client
        .is_admin(ADMIN_ID.into())
        .recv(program_id)
        .await
        .unwrap();
    assert!(is_admin);

    // Mitt of NON_ADMIN_ID should fail
    let is_admin = service_client
        .is_admin(NON_ADMIN_ID.into())
        .recv(program_id)
        .await
        .unwrap();
    assert!(!is_admin);

}


#[tokio::test]
async fn init_vft() {
    let remoting = GTestRemoting::new(ADMIN_ID.into());
    remoting.system().init_logger();

    // Submit program code into the system
    let program_code_id = remoting.system().submit_code(nexus_vft::WASM_BINARY);

    let program_factory = nexus_vft_client::NexusVftFactory::new(remoting.clone());

    // Initialize with ADMIN_ID as the first admin
    let program_id = program_factory
        .new()
        .send_recv(program_code_id, b"salt")
        .await
        .unwrap();

    let mut service_client = nexus_vft_client::NexusVft::new(remoting.clone());

    let is_admin = service_client
        .is_admin(ADMIN_ID.into())
        .recv(program_id)
        .await
        .unwrap();
    assert!(is_admin);

    let mint_result = service_client
        .init_vft("TokenName".to_string(), "TokenSymbol".to_string(), 18, [(TOKEN_OWNER_ID_A.into(), 500000_000000000000000000u128.into()), (TOKEN_OWNER_ID_B.into(), 500000_000000000000000000u128.into())].to_vec())
        .send_recv(program_id)
        .await
        .unwrap();
    assert!(mint_result);

    // Test the `balance_of` method
    let balance = service_client
        .balance_of(TOKEN_OWNER_ID_A.into()) // Get the balance of ACTOR_ID
        .recv(program_id)
        .await
        .unwrap();

    assert_eq!(balance, 500000_000000000000000000u128.into());

    let balance = service_client
        .balance_of(TOKEN_OWNER_ID_B.into()) // Get the balance of ACTOR_ID
        .recv(program_id)
        .await
        .unwrap();

    assert_eq!(balance, 500000_000000000000000000u128.into());

    let mint_result = service_client
        .init_vft("TokenName".to_string(), "TokenSymbol".to_string(), 18, [(TOKEN_OWNER_ID_A.into(), 500000_000000000000000000u128.into()), (TOKEN_OWNER_ID_B.into(), 500000_000000000000000000u128.into())].to_vec())
        .send_recv(program_id)
        .await
        .unwrap();
    assert!(mint_result);

}