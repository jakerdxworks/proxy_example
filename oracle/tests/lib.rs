use radix_engine_interface::prelude::*;
use scrypto::{info, this_package};
use scrypto_test::prelude::*;
use scrypto_unit::*;
use transaction::manifest::decompiler::ManifestObjectNames;


#[test]
fn instantiate() {
    let mut env = TestEnvironment::new();
    env.instantiate_dapp();
}

#[test]
fn dapp_can_call_allowed_logic_methods_through_proxy() {
    let mut env = TestEnvironment::new();
    env.instantiate_dapp();
    let manifest = ManifestBuilder::new()
        
        .call_method(
            env.dapp_component_address.unwrap(),
            "call_method",
            manifest_args!(
                String::from("get_derived_svalue"),
                // Empty argument
                ()
            )
        );

    let receipt = env.execute_manifest(
        manifest.object_names(),
        manifest.build(),
        "dapp_can_call_allowed_logic_methods_through_proxy",
    );

    println!("{}", receipt.display(&AddressBech32Encoder::for_simulator()));

    receipt.expect_commit(true);
}

#[test]
fn dapp_cant_call_permissioned_logic_methods_through_proxy() {
    let mut env = TestEnvironment::new();
    env.instantiate_dapp();
    let manifest = ManifestBuilder::new()
        
        .call_method(
            env.dapp_component_address.unwrap(),
            "call_method",
            manifest_args!(
                String::from("validate_and_update_data"),
                dec!(1)
            )
        );

    let receipt = env.execute_manifest(
        manifest.object_names(),
        manifest.build(),
        "dapp_cant_call_permissioned_logic_methods_through_proxy",
    );

    println!("{}", receipt.display(&AddressBech32Encoder::for_simulator()));

    receipt.expect_commit(false);
}

#[test]
fn cant_call_logic_method_directly() {
    let mut env = TestEnvironment::new();
    env.instantiate_dapp();

    // get_derived_svalue
    let manifest = ManifestBuilder::new()   
        .call_method(
            env.logic_component_address,
            "get_derived_svalue",
            manifest_args!()
        );

    let receipt = env.execute_manifest(
        manifest.object_names(),
        manifest.build(),
        "cant_call_logic_methods_directly",
    );

    println!("{}", receipt.display(&AddressBech32Encoder::for_simulator()));

    receipt.expect_commit(false);
}

#[test]
fn cant_call_proxy_method_without_badge() {
    let mut env = TestEnvironment::new();
    env.instantiate_dapp();
    let manifest = ManifestBuilder::new()
        .call_method(
            env.proxy_component_address,
            "call_method",
            manifest_args!(
                String::from("get_derived_svalue"),
                ()
            )
        );

    let receipt = env.execute_manifest(
        manifest.object_names(),
        manifest.build(),
        "cant_call_proxy_method_without_badge",
    );

    println!("{}", receipt.display(&AddressBech32Encoder::for_simulator()));
    receipt.expect_commit(false);
}

#[test]
fn logic_component_can_call_permissioned_storage_methods_as_owner() {
    let mut env = TestEnvironment::new();
    env.instantiate_dapp();
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_amount(
            env.account.account_component,
            env.owner_badge,
            dec!(1)
        )
        .call_method(
            env.logic_component_address,
            "validate_and_update_data",
            manifest_args!(dec!(1))
        );

    let receipt = env.execute_manifest(
        manifest.object_names(),
        manifest.build(),
        "logic_component_can_call_permissioned_storage_methods_as_owner",
    );

    println!("{}", receipt.display(&AddressBech32Encoder::for_simulator()));

    receipt.expect_commit(true);
}

#[test]
fn logic_component_cant_call_permissioned_storage_methods_without_owner() {
    let mut env = TestEnvironment::new();
    env.instantiate_dapp();
    let manifest = ManifestBuilder::new()
        .call_method(
            env.logic_component_address,
            "validate_and_update_data",
            manifest_args!(dec!(1))
        );

    let receipt = env.execute_manifest(
        manifest.object_names(),
        manifest.build(),
        "logic_component_cant_call_permissioned_storage_methods_without_owner",
    );

    println!("{}", receipt.display(&AddressBech32Encoder::for_simulator()));

    receipt.expect_commit(false);
}

#[test]
fn nobody_can_call_permissioned_storage_methods_directly() {
    let mut env = TestEnvironment::new();
    env.instantiate_dapp();
    let manifest = ManifestBuilder::new()
        .call_method(
            env.storage_component_address,
            "update_data",
            manifest_args!(dec!(1))
        );

    let receipt = env.execute_manifest(
        manifest.object_names(),
        manifest.build(),
        "nobody_can_call_permissioned_storage_methods_directly",
    );

    println!("{}", receipt.display(&AddressBech32Encoder::for_simulator()));

    receipt.expect_commit(false);
}


pub struct Account {
    pub public_key: Secp256k1PublicKey,
    pub account_component: ComponentAddress,
}

pub struct TestEnvironment {
    pub test_runner: DefaultTestRunner,
    pub account: Account,
    pub owner_badge: ResourceAddress,
    pub white_list_badge: ResourceAddress,
    pub oracle_package_address: PackageAddress,
    pub storage_component_address: ComponentAddress,
    pub logic_component_address: ComponentAddress,
    pub proxy_component_address: ComponentAddress,
    pub dapp_package_address: Option<PackageAddress>,
    pub dapp_component_address: Option<ComponentAddress>,
}

impl TestEnvironment {
    pub fn new() -> Self {

        // Setting up testing environment without trace to speed up testing a bit.
        let mut test_runner = 
            TestRunnerBuilder::new().build();

        // Creating account
        let (public_key, _private_key, account) = 
            test_runner.new_allocated_account();

        let account = Account {
            public_key,
            account_component: account,
        };

        // Compiling and publishing oracle package
        let oracle_package_address = 
            test_runner.compile_and_publish(this_package!());

        // Creating white list badge from manifest
        let white_list_badge = test_runner.create_fungible_resource(
            dec!(1), 
            0u8, 
            account.account_component
        );

        // Creating owner badge
        let owner_badge = test_runner.create_fungible_resource(
            dec!(1), 
            0u8, 
            account.account_component
        );

        // ----------------------------------- PROXY COMPONENT ----------------------------------- //
        // Instantiating Proxy component first and passing in owner badge and white list badge
        let manifest = ManifestBuilder::new()
            .call_function(
                oracle_package_address,
                "OracleGenericProxy",
                "instantiate_and_globalize",
                manifest_args!(
                    owner_badge,
                    white_list_badge,
                ),
            )
            .build();

        let receipt = test_runner.execute_manifest_ignoring_fee(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&account.public_key)],
        );

        let proxy_component_address = 
            receipt.expect_commit(true).new_component_addresses()[0];
        // ----------------------------------- PROXY COMPONENT ----------------------------------- //



        // ----------------------------------- STORAGE COMPONENT & LOGIC COMPONENT ----------------------------------- //
        // Instantiating storage and logic component.
        // Storage component instantiates the logic component within its instantiation function.
        let manifest = ManifestBuilder::new()
            .call_function(
                oracle_package_address,
                "Storage",
                "instantiate_and_globalize",
                manifest_args!(
                    owner_badge,
                    proxy_component_address
                ),
            )
            .build();

            let receipt = test_runner.execute_manifest_ignoring_fee(
                manifest,
                vec![NonFungibleGlobalId::from_public_key(&account.public_key)],
            );

            let storage_component_address = 
                receipt.expect_commit(true).new_component_addresses()[1];
            let logic_component_address = 
                receipt.expect_commit(true).new_component_addresses()[0];
            // ----------------------------------- STORAGE COMPONENT & LOGIC COMPONENT ----------------------------------- //

            // Setting storage component in logic component
            // Setting logic component in proxy component
            let manifest = ManifestBuilder::new()
                .create_proof_from_account_of_amount(
                    account.account_component, 
                    owner_badge, 
                    dec!(1)
                )
                .call_method(
                    logic_component_address,
                    "set_storage_component",
                    manifest_args!(
                        storage_component_address
                    )
                )
                .call_method(
                    proxy_component_address,
                    "set_oracle_address",
                    manifest_args!(
                        logic_component_address
                    )
                )
                .build();

            let receipt = test_runner.execute_manifest_ignoring_fee(
                manifest,
                vec![NonFungibleGlobalId::from_public_key(&account.public_key)],
            );

            info!("Receipt: {:#?}", receipt.display(&AddressBech32Encoder::for_simulator()));

            receipt.expect_commit(true);

        Self {
            test_runner,
            account,
            owner_badge,
            white_list_badge,
            oracle_package_address,
            storage_component_address,
            logic_component_address,
            proxy_component_address,
            dapp_package_address: None,
            dapp_component_address: None,
        }
    }

    // Compiling, publishing, and instantiating dApp component separately
    pub fn instantiate_dapp(&mut self) {
        let dapp_package_address = 
            self.test_runner.compile_and_publish("../dapp");

        // Instantiating dApp component and passing in a white list badge
        // to the dApp to allow access to proxy component methods.
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(
                self.account.account_component, 
                self.white_list_badge, 
                dec!(1)
            )
            .take_from_worktop(
                self.white_list_badge, 
                dec!(1), 
                "white_list_badge_bucket"
            )
            .call_function_with_name_lookup(
                dapp_package_address,
                "Hello",
                "instantiate_hello",
                |lookup| {
                    manifest_args!(
                        lookup.bucket("white_list_badge_bucket"),
                    )
                }
            )
            .build();

        let receipt = self.test_runner.execute_manifest_ignoring_fee(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&self.account.public_key)],
        );

        let dapp_component_address = 
            receipt.expect_commit(true).new_component_addresses()[0];

        self.dapp_package_address = Some(dapp_package_address);
        self.dapp_component_address = Some(dapp_component_address);
    }

    pub fn execute_manifest(
        &mut self,
        object_manifest: ManifestObjectNames,
        built_manifest: TransactionManifestV1,
        name: &str,
    ) -> TransactionReceiptV1 {
        dump_manifest_to_file_system(
            object_manifest,
            &built_manifest,
            "./transaction_manifest",
            Some(name),
            &NetworkDefinition::stokenet(),
        )
        .ok();

        let receipt = self.test_runner.execute_manifest_ignoring_fee(
            built_manifest,
            vec![NonFungibleGlobalId::from_public_key(
                &self.account.public_key,
            )],
        );

        return receipt;
    }
}

