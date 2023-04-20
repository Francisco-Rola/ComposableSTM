extern crate alloc;
use alloc::collections::BTreeMap;

use cosmwasm_vm::executor::{cosmwasm_call, ExecuteCall, InstantiateCall};
use cosmwasm_vm::system::{cosmwasm_system_entrypoint_hook, CosmwasmContractMeta, SystemError};
use cosmwasm_vm_wasmi::{
    create_simple_vm, instrument_contract, BankAccount, Gas, OwnedWasmiVM, SimpleVMError,
    SimpleWasmiVM, SimpleWasmiVMExtension, WasmiVMError,
};

use cosmwasm_std::ContractResult;

#[test]
fn test_hook() {
    let code = instrument_contract(include_bytes!("../../fixtures/cw20_base.wasm"));
    let sender = BankAccount::new(100);
    let address = BankAccount::new(10_000);
    let funds = vec![];
    let mut extension = SimpleWasmiVMExtension {
        storage: BTreeMap::default(),
        codes: BTreeMap::from([(0x1337, code)]),
        contracts: BTreeMap::from([(
            address,
            CosmwasmContractMeta {
                code_id: 0x1337,
                admin: None,
                label: String::new(),
            },
        )]),
        next_account_id: BankAccount::new(10_001),
        transaction_depth: 0,
        gas: Gas::new(100_000_000),
        ..Default::default()
    };
    let mut vm = create_simple_vm(sender, address, funds, &mut extension).unwrap();
    let _ = cosmwasm_system_entrypoint_hook::<InstantiateCall, OwnedWasmiVM<SimpleWasmiVM>>(
        &mut vm,
        format!(
            r#"{{
                  "name": "Picasso",
                  "symbol": "PICA",
                  "decimals": 12,
                  "initial_balances": [],
                  "mint": {{
                    "minter": "{}",
                    "cap": null
                  }},
                  "marketing": null
                }}"#,
            sender.credit()
        )
        .as_bytes(),
        |vm, msg| {
            cosmwasm_call::<InstantiateCall, OwnedWasmiVM<SimpleWasmiVM>>(vm, msg).map(Into::into)
        },
    )
    .unwrap();
    let r = cosmwasm_system_entrypoint_hook::<ExecuteCall, OwnedWasmiVM<SimpleWasmiVM>>(
        &mut vm,
        r#"{
              "mint": {
                "recipient": "10001",
                "amount": "5555"
              }
            }"#
        .as_bytes(),
        |_, _| Ok(ContractResult::Err("Bro".into())),
    );
    assert_eq!(
        r,
        Err(SimpleVMError::VMError(WasmiVMError::SystemError(
            SystemError::ContractExecutionFailure("Bro".into())
        )))
    );
}
