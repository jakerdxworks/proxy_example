use scrypto::prelude::*;
use crate::logic_blueprint::logic::Logic;
use crate::logic_blueprint::PriceData;


#[blueprint]
mod storage {
    // Storage blueprint auth allows logic component to call these methods:
    // * update_data - to store data in storage component
    // * get_derived_svalue - to get data from storage component

    enable_method_auth! {
        roles {
            logic => updatable_by: [OWNER];
        },
        methods {
            update_data => restrict_to: [logic];
            get_derived_svalue => restrict_to: [logic];
        }
    }

    pub struct Storage {
        pub data: Decimal,
    }

    impl Storage {
        pub fn instantiate_and_globalize(
            owner_badge: ResourceAddress,
            proxy_component_address: ComponentAddress,
        ) -> Global<Storage> {

            let logic_component = 
                Logic::instantiate_and_globalize(
                    owner_badge.clone(), 
                    proxy_component_address,
                );
            
            Self {
                data: Decimal::ZERO,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(owner_badge))))
            .roles(roles!(
                logic => rule!(require(global_caller(logic_component.address())));
            ))
            .globalize()
        }

        pub fn update_data(&mut self, data: Decimal) {
            self.data = data;
        }

        pub fn get_derived_svalue(&self) -> PriceData {
            PriceData {
                decimal: self.data
            }
        }
    }
}