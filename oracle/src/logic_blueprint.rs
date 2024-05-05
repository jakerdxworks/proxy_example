use scrypto::prelude::*;
use crate::storage_blueprint::storage::Storage;

#[derive(ScryptoSbor, Clone, Debug)]
 pub struct PriceData {
    pub decimal: Decimal
}

#[blueprint]
mod logic {
    enable_method_auth! {
        roles {
            proxy => updatable_by: [OWNER];
        },
        methods {
            set_storage_component => restrict_to: [OWNER];
            validate_and_update_data => restrict_to: [OWNER];
            get_derived_svalue => restrict_to: [proxy];
        }
    }

    struct Logic {
        storage_component: Option<Global<Storage>>,
    }

    impl Logic {
        pub fn instantiate_and_globalize(
            owner_badge: ResourceAddress,
            proxy_component_address: ComponentAddress,
        ) -> Global<Logic> {
            
            Self {
                storage_component: None,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(owner_badge))))
            .roles(roles!(
                proxy => rule!(require(global_caller(proxy_component_address)));
            ))
            .globalize()
        }

        pub fn set_storage_component(&mut self, storage_component: Global<Storage>) {
            self.storage_component = Some(storage_component);
        }

        pub fn validate_and_update_data(&mut self, data: Decimal) {
            
            let storage_component = 
            self.storage_component.unwrap();
            
            storage_component.update_data(data);
        }

        pub fn get_derived_svalue(&mut self) -> PriceData {
            let storage_component = 
            self.storage_component.expect("Storage component not set");

            storage_component.get_derived_svalue()
            
        }
    }
}