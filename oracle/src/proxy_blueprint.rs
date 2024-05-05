use scrypto::prelude::*;
use crate::logic_blueprint::logic::Logic;
use crate::logic_blueprint::PriceData;


#[blueprint]
mod proxy {
    enable_method_auth! {
        roles {
            white_listed => updatable_by: [OWNER];
        },
        methods {
            set_oracle_address => restrict_to: [OWNER];
            // Only those with the white list badge can call this method
            call_method => restrict_to: [white_listed];
        }
    }

    struct OracleGenericProxy {
        oracle_logic_component: Option<Global<Logic>>,
    }

    impl OracleGenericProxy {
        pub fn instantiate_and_globalize(
            owner_badge: ResourceAddress,
            // Using `ResourceAddress` instead of `NonFungibleGlobalId` as
            // it allows anyone with a white list NFT badge to have access to the method
            // rather than specific white list badge.
            white_list_badge: ResourceAddress,
        ) -> Global<OracleGenericProxy> {

            Self {
                oracle_logic_component: None,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(owner_badge))))
            .roles(roles!(
                white_listed => rule!(require(white_list_badge));
            ))
            .globalize()
        }

        // Specify Oracle component address
        pub fn set_oracle_address(&mut self, address: Global<Logic>) {
            
            self.oracle_logic_component = Some(address);
        }

        pub fn call_method(&self, method_name: String, args: ScryptoValue) -> PriceData {
            let args = scrypto_encode(&args).unwrap();

            let bytes = ScryptoVmV1Api::object_call(
                self.oracle_logic_component
                    .expect("Component address not set")
                    .handle()
                    .as_node_id(),
                &method_name,
                args,
            );

            let return_value = scrypto_decode(&bytes).unwrap();

            return_value
        }
    }
}