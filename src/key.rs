
use crate::encryption::key::StoreKey;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Key{
    pub hex:Vec<u8>,
}

impl Key{
    //Retrieves program key, if key does not yet exist one is created

    pub fn retrieve_key() -> Self{
        
        let encryption_key = match StoreKey::retrieve_key() {
            Some(key) => {
                Key{
                    hex:StoreKey::decrypt_data(&key)
                }
            },
            None =>{
                let new_key = StoreKey::make_key();
                let encrypted_key = StoreKey::encrypt_data(&new_key);
                Key{
                    hex:StoreKey::decrypt_data(&encrypted_key)
                }

            },
        };
        encryption_key
    }

    
}