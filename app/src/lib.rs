#![no_std]

use sails_rs::prelude::*;
use vft_service::{Service as VftService, Storage};
use gstd::{ActorId, collections::HashSet, msg, prog, debug, exec};
use gstd::collections::HashMap;

#[derive(Default)]
pub struct NexusVftStorage {
    is_initialized: bool,
    admins: HashSet<ActorId>,
}

static mut NEXUS_VFT_STORAGE: Option<NexusVftStorage> = None;


#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, TypeInfo)]
pub enum Event {
    AdminAdded { admin: ActorId },
    AdminRemoved { admin: ActorId },
    InitVft { name: String, symbol: String, decimals: u8, initial_balances: Vec<(ActorId, U256)> },
}

#[derive(Clone)]
pub struct NexusVftService {
    vft: VftService,
}

impl NexusVftService {

    pub fn new() -> Self {
        let mut admins = HashSet::new();
        admins.insert(msg::source()); //

        unsafe {
            NEXUS_VFT_STORAGE = Some(NexusVftStorage {
                is_initialized: false,
                admins,
            });
        }

        Self {
            vft: VftService::new(),
        }
    }

    pub fn seed(name: String, symbol: String, decimals: u8, initial_balances: Vec<(ActorId, U256)>) -> Self {
        let mut admins = HashSet::new();
        admins.insert(msg::source());

        let mut _self = Self::new();
        _self._init_vft(name, symbol, decimals, initial_balances);

        unsafe {
            NEXUS_VFT_STORAGE = Some(NexusVftStorage {
                is_initialized: true,
                admins,
            });
        }

        _self
    }


    fn _init_vft(&mut self, name: String, symbol: String, decimals: u8, initial_balances: Vec<(ActorId, U256)>) -> bool {
        if !self.is_admin(msg::source()) {
            panic!("Only an admin can mint tokens");
        }
        let _initial_balances: HashMap<ActorId, sails_rs::U256> = initial_balances.clone().into_iter().collect();
        // Initialize VftService
        let vft_service = VftService::seed(name.clone(), symbol.clone(), decimals.clone());
        // Set initial balances
        let balances = Storage::balances();
        let total_supply = Storage::total_supply();
        for (actor, amount) in _initial_balances {
            let balance = balances.entry(actor).or_insert(U256::zero());
            *balance += amount;
            *total_supply += amount;
        }
        self.vft = vft_service;

        true
    }

    pub fn get_mut(&mut self) -> &'static mut NexusVftStorage {
        unsafe {
            NEXUS_VFT_STORAGE
                .as_mut()
                .expect("Nexus VFT is not initialized")
        }
    }

    pub fn get(&self) -> &'static NexusVftStorage {
        unsafe {
            NEXUS_VFT_STORAGE
                .as_ref()
                .expect("Nexus VFT is not initialized")
        }
    }
}

#[sails_rs::service(events = Event)]
impl NexusVftService {

    pub fn is_admin(&self, actor: ActorId) -> bool {
        let result = self.get().admins.contains(&actor);
        result
    }

    // Mint function
    pub fn mint(&mut self, to: ActorId, amount: U256) -> bool {
        if !self.is_admin(msg::source()) {
            panic!("Only an admin can mint tokens");
        }
        let balances = Storage::balances();
        let total_supply = Storage::total_supply();
        let balance = balances.entry(to).or_insert(U256::zero());
        *balance += amount;
        *total_supply += amount;
        true
    }

    pub fn create_vft(&mut self) -> ActorId {
        let code_id: CodeId = msg::load().expect("Unable to load");
        let (init_message_id, new_program_id) =
            prog::create_program_bytes(code_id, "salt".as_bytes(), b"NEW", 0)
                .expect("Unable to create a program");
        new_program_id
    }

    pub fn init_vft(&mut self, name: String, symbol: String, decimals: u8, initial_balances: Vec<(ActorId, U256)>) -> bool {

        if !self.is_admin(msg::source()) {
            return false;
        }
        if self.get().is_initialized {
            return false;
        }

        if self._init_vft(name.clone(), symbol.clone(), decimals.clone(), initial_balances.clone()) {
            let _ = self.notify_on(Event::InitVft {
                name,
                symbol,
                decimals,
                initial_balances,
            });

            self.get_mut().is_initialized = true;

            true
        } else {
            false
        }
    }

    pub fn add_admin(&mut self, new_admin: ActorId) {
        if !self.is_admin(msg::source()) {
            panic!("Only an admin can add another admin");
        }
        self.get_mut().admins.insert(new_admin);
    }

    pub fn remove_admin(&mut self, admin: ActorId) {
        if !self.is_admin(msg::source()) {
            panic!("Only an admin can remove an admin");
        }
        self.get_mut().admins.remove(&admin);

    }

    // For vft service
    pub fn balance_of(&self, account: ActorId) -> U256 {
        self.vft.balance_of(account)
    }

    pub fn balances(&self) -> Vec<(ActorId, U256)> {
        Storage::balances().iter().map(|(k, v)| (*k, *v)).collect()
    }

    pub fn approve(&mut self, spender: ActorId, value: U256) -> bool {
        self.vft.approve(spender, value)
    }

    pub fn transfer(&mut self, to: ActorId, value: U256) -> bool {
        self.vft.transfer(to, value)
    }

    pub fn transfer_from(&mut self, from: ActorId, to: ActorId, value: U256) -> bool {
        self.vft.transfer_from(from, to, value)
    }

    pub fn allowance(&self, owner: ActorId, spender: ActorId) -> U256 {
        self.vft.allowance(owner, spender)
    }

    pub fn decimals(&self) -> &'static u8 {
        self.vft.decimals()
    }

    pub fn name(&self) -> &'static str {
        self.vft.name()
    }

    pub fn symbol(&self) -> &'static str {
        self.vft.symbol()
    }

    pub fn total_supply(&self) -> &'static U256 {
        self.vft.total_supply()
    }
}

pub struct NexusVftProgram(());

#[sails_rs::program]
impl NexusVftProgram {
    // Program's constructor
    pub fn initialize(name: String, symbol: String, decimals: u8, initial_balances: Vec<(ActorId, U256)>) -> Self {
        NexusVftService::seed(name, symbol, decimals, initial_balances);
        Self(())
    }

    pub fn new() -> Self {
        NexusVftService::new();
        Self(())
    }

    // Exposed service
    pub fn nexus_vft(&self) -> NexusVftService {
        NexusVftService::new()
    }
}
