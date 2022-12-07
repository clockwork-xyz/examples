use {
    crate::*,
    clockwork_sdk::client::{Client, ClientResult},
};

pub fn create_mint(client: &Client) -> ClientResult<()> {
    let mint = client
        .create_token_mint(&client.payer_pubkey(), 9)
        .unwrap()
        .pubkey();

    let subscriber_token_account = client
        .create_associated_token_account(&client.payer, &client.payer_pubkey(), &mint)
        .unwrap();

    client
        .mint_to(&client.payer, &mint, &subscriber_token_account, 100000, 9)
        .unwrap();

    println!("- - - - - - - - - - UPDATE YOUR .ENV FILE - - - - - - - - - -");
    println!("MINT=\"{:?}\"", mint);
    println!(
        "SUBSCRIBER_TOKEN_ACCOUNT=\"{:?}\"",
        subscriber_token_account
    );

    Ok(())
}
