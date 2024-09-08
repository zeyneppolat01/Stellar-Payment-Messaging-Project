use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

use soroban_sdk::{contractimpl, Env, Symbol, Address};

pub struct PaymentContract;

#[derive(Serialize, Deserialize, Debug)]
struct TransactionRecord {
    from: Address,
    to: Address,
    amount: i128,
    message: String,
}

#[contractimpl]
impl PaymentContract {

    pub fn send_payment(env: Env, from: Address, to: Address, amount: i128, message: Symbol) {

        let key = Symbol::new(&env, "last_payment_message");
        env.storage().persistent().set(&key, &message);


        let sender_secret = from.to_string();
        let receiver_public = to.to_string();
        let client = Client::new();

        let transaction = json!({
            "source": &*sender_secret,
            "operations": [{
                "type": "payment",
                "destination": &*receiver_public,
                "asset": {
                    "type": "native"
                },
                "amount": amount.to_string()
            }],
            "memo": {
                "type": "text",
                "text": message.to_string()
            }
        });

        let response = client
            .post("https://horizon-testnet.stellar.org/transactions")
            .json(&transaction)
            .send();

        match response {
            Ok(resp) => println!("Transaction Response: {:?}", resp.text().unwrap()),
            Err(e) => panic!("Transaction failed: {:?}", e),
        }
    }


    pub fn send_payment_multiple(env: Env, from: Address, to_list: Vec<Address>, amount: i128, message: Symbol) {
        for to in to_list {
            PaymentContract::send_payment(env.clone(), from.clone(), to, amount, message.clone());
        }
    }


    pub fn record_transaction(env: Env, from: Address, to: Address, amount: i128, message: Symbol) {
        let key = Symbol::new(&env, "transaction_history");


        let history_json: String = env.storage().persistent().get(&key).unwrap_or_else(|| String::new());
        let mut history: Vec<TransactionRecord> = if history_json.is_empty() {
            Vec::new()
        } else {
            serde_json::from_str(&history_json).unwrap_or_default()
        };


        let record = TransactionRecord {
            from,
            to,
            amount,
            message: message.to_string(),
        };


        history.push(record);


        let updated_history_json = serde_json::to_string(&history).unwrap();
        env.storage().persistent().set(&key, &updated_history_json);
    }

    pub fn get_transaction_history(env: Env) -> Vec<TransactionRecord> {
        let key = Symbol::new(&env, "transaction_history");


        let history_json: String = env.storage().persistent().get(&key).unwrap_or_else(|| String::new());


        if history_json.is_empty() {
            Vec::new()
        } else {
            serde_json::from_str(&history_json).unwrap_or_default()
        }
    }


    pub fn get_last_message(env: Env) -> Symbol {
        let key = Symbol::new(&env, "last_payment_message");
        env.storage()
            .persistent()
            .get::<Symbol>(&key)
            .unwrap_or_else(|| Symbol::new(&env, "No message"))
    }
}

#[derive(Deserialize, Debug)]
struct Balance {
    asset_type: String,
    balance: String,
}

#[derive(Deserialize, Debug)]
struct Account {
    balances: Vec<Balance>,
}

fn get_account_balance(public_key: &str) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("https://horizon-testnet.stellar.org/accounts/{}", public_key);
    let client = Client::new();
    let response: Account = client.get(&url).send()?.json()?;

    for balance in response.balances {
        println!("Asset Type: {}, Balance: {}", balance.asset_type, balance.balance);
    }

    Ok(())
}



fn main() {
    let public_key = "GA...";
    get_account_balance(public_key).unwrap();

    let sender_secret = "SB...";
    let receiver_public = "GA...";
    let amount = 10_i128;
    let memo = "Thanks!";

    let env = Env::default();
    let from_address = Address::from_account_id(&env, sender_secret);
    let to_address = Address::from_account_id(&env, receiver_public);
    let message = Symbol::new(&env, memo);




    PaymentContract::send_payment(env.clone(), from_address.clone(), to_address.clone(), amount, message.clone());


    let multiple_recipients = vec![to_address.clone(), Address::from_account_id(&env, "GA...2")];
    PaymentContract::send_payment_multiple(env.clone(), from_address.clone(), multiple_recipients, amount, message.clone());


    let last_message = PaymentContract::get_last_message(env.clone());
    println!("Son mesaj: {:?}", last_message);


    PaymentContract::record_transaction(env.clone(), from_address.clone(), to_address.clone(), amount, message.clone());


    let history = PaymentContract::get_transaction_history(env.clone());


    for record in history {
        println!("From: {:?}, To: {:?}, Amount: {}, Message: {}", record.from, record.to, record.amount, record.message);
    }
}
