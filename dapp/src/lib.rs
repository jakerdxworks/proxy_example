use scrypto::prelude::*;

#[derive(ScryptoSbor, Clone, Debug)]
 pub struct PriceData {
    decimal: Decimal
}

#[blueprint]
mod hello {

    extern_blueprint! {
        "package_sim1phhyaadjcggz9vs26vp5rl52pvsa0mppqkfkt9ld7rqdndxpzcl9j8",
        OracleProxy {
            fn call_method(&mut self, method_name: String, args: ScryptoValue) -> PriceData;
        }
    }

    const ORACLEPROXY: Global<OracleProxy> = global_component! (
        OracleProxy,
        "component_sim1cz04880nauk42t5ckne2l2a4u5dpl3p9vuuac3vsg9prmk0eg9qj9x"
    );

    struct Hello {
        white_list_vault: Vault,
    }

    impl Hello {
        pub fn instantiate_hello(
            bucket: Bucket
        ) -> Global<Hello> {
            // Instantiate a Hello component, populating its vault with our supply of 1000 HelloToken
            Self {
                white_list_vault: Vault::with_bucket(bucket),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn call_method(&mut self, method: String, args: ScryptoValue) -> PriceData {

            let price_data = self.white_list_vault.as_fungible().authorize_with_amount(dec!(1),
            || {
                ORACLEPROXY.call_method(
                    method,
                    args
                )
            });

            price_data
        }
    }
}
