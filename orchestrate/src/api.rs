use crate::vm::{create_vm, Account, Context, Gas, State, VmError};
use cosmwasm_std::{Addr, BlockInfo, Coin, ContractInfo, Empty, Env, MessageInfo, Timestamp};
use cosmwasm_vm::{
    executor::{
        cosmwasm_call, ExecuteInput, ExecuteResult, InstantiateInput, InstantiateResult,
        QueryInput, QueryResult,
    },
    system::{CosmwasmCodeId, CosmwasmContractMeta},
};
use cosmwasm_vm_wasmi::WasmiVM;
use serde::Serialize;

pub fn instantiate_raw(
    vm_state: &mut State,
    sender: &Account,
    code_id: CosmwasmCodeId,
    admin: Option<Account>,
    funds: Vec<Coin>,
    gas: u64,
    message: &[u8],
) -> Result<(Account, InstantiateResult<Empty>), VmError> {
    let code_hash = &vm_state
        .codes
        .get(&code_id)
        .ok_or(VmError::CodeNotFound(code_id))?
        .1;
    let contract_addr = Account::generate(code_hash, message);
    instantiate_with_address_raw(
        &contract_addr,
        vm_state,
        sender,
        code_id,
        admin,
        funds,
        gas,
        message,
    )
}

pub fn instantiate<M: Serialize>(
    vm_state: &mut State,
    sender: &Account,
    code_id: CosmwasmCodeId,
    admin: Option<Account>,
    funds: Vec<Coin>,
    gas: u64,
    message: M,
) -> Result<(Account, InstantiateResult<Empty>), VmError> {
    let message = serde_json::to_vec(&message).map_err(|_| VmError::CannotSerialize)?;
    instantiate_raw(vm_state, sender, code_id, admin, funds, gas, &message)
}

#[allow(clippy::too_many_arguments)]
pub fn instantiate_with_address<M: Serialize>(
    address: &Account,
    vm_state: &mut State,
    sender: &Account,
    code_id: CosmwasmCodeId,
    admin: Option<Account>,
    funds: Vec<Coin>,
    gas: u64,
    message: M,
) -> Result<(Account, InstantiateResult<Empty>), VmError> {
    let message = serde_json::to_vec(&message).map_err(|_| VmError::CannotSerialize)?;
    instantiate_with_address_raw(
        address, vm_state, sender, code_id, admin, funds, gas, &message,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn instantiate_with_address_raw(
    address: &Account,
    vm_state: &mut State,
    sender: &Account,
    code_id: CosmwasmCodeId,
    admin: Option<Account>,
    funds: Vec<Coin>,
    gas: u64,
    message: &[u8],
) -> Result<(Account, InstantiateResult<Empty>), VmError> {
    vm_state.gas = Gas::new(gas);
    if vm_state.contracts.contains_key(address) {
        return Err(VmError::AlreadyInstantiated);
    }
    vm_state.contracts.insert(
        address.clone(),
        CosmwasmContractMeta {
            code_id,
            admin,
            label: String::from("test-label"),
        },
    );
    let mut vm = create_vm(
        vm_state,
        Env {
            block: BlockInfo {
                height: 0xDEADC0DE,
                time: Timestamp::from_seconds(10000),
                chain_id: "abstract-test".into(),
            },
            transaction: None,
            contract: ContractInfo {
                address: address.clone().into(),
            },
        },
        MessageInfo {
            sender: sender.clone().into(),
            funds,
        },
    );

    Ok((
        address.clone(),
        cosmwasm_call::<InstantiateInput<Empty>, WasmiVM<Context>>(&mut vm, message)?,
    ))
}

pub fn execute_raw(
    vm_state: &mut State,
    sender: &Account,
    contract: &Account,
    funds: Vec<Coin>,
    gas: u64,
    message: &[u8],
) -> Result<ExecuteResult<Empty>, VmError> {
    vm_state.gas = Gas::new(gas);
    let mut vm = create_vm(
        vm_state,
        Env {
            block: BlockInfo {
                height: 0xCAFEBABE,
                time: Timestamp::from_seconds(10000),
                chain_id: "abstract-test".into(),
            },
            transaction: None,
            contract: ContractInfo {
                address: contract.clone().into(),
            },
        },
        MessageInfo {
            sender: sender.clone().into(),
            funds,
        },
    );
    cosmwasm_call::<ExecuteInput<Empty>, WasmiVM<Context>>(&mut vm, message)
}

pub fn execute<M: Serialize>(
    vm_state: &mut State,
    sender: &Account,
    contract: &Account,
    funds: Vec<Coin>,
    gas: u64,
    message: M,
) -> Result<ExecuteResult<Empty>, VmError> {
    let message = serde_json::to_vec(&message).map_err(|_| VmError::CannotSerialize)?;
    execute_raw(vm_state, sender, contract, funds, gas, &message)
}

pub fn query_raw(
    vm_state: &mut State,
    contract: &Account,
    message: &[u8],
) -> Result<QueryResult, VmError> {
    let mut vm = create_vm(
        vm_state,
        Env {
            block: BlockInfo {
                height: 0xCAFEBABE,
                time: Timestamp::from_seconds(10000),
                chain_id: "abstract-test".into(),
            },
            transaction: None,
            contract: ContractInfo {
                address: contract.clone().into(),
            },
        },
        MessageInfo {
            sender: Addr::unchecked("MOCK"),
            funds: vec![],
        },
    );
    cosmwasm_call::<QueryInput, WasmiVM<Context>>(&mut vm, message)
}

pub fn query<M: Serialize>(
    vm_state: &mut State,
    contract: &Account,
    message: M,
) -> Result<QueryResult, VmError> {
    let message = serde_json::to_vec(&message).map_err(|_| VmError::CannotSerialize)?;
    query_raw(vm_state, contract, &message)
}
