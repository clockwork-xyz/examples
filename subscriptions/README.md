### Prerequisite 

1. Create `.env` file and initialize it with the following content

```
CLIENT_PRIVATE=
SUBSCRIPTION=
SUBSCRIPTION_THREAD=
SUBSCRIPTION_BANK=
SUBSCRIBER=
SUBSCRIBER_TOKEN_ACCOUNT=
MINT=
SUBSCRIPTION_ID=
```

After Running each command, update your .env file accordingly.

### Usage
1. Initialize mint and token accounts for testing purposes.

```
cargo run -- --command create_mint
```

2. Initialize a Subscription with the specified `recurrent_amount`

```
cargo run -- --command create_subscription --recurrent_amount <amount>
```

3. Deactivate a Subscription. 

```
cargo run -- --command deactivate_subscription
```

4. Withdraw from the Subscription if you're the owner.

```
cargo run -- --command withdraw
```

5. Create a Subscriber for a Subscription.

```
cargo run -- --command create_subscriber
```

6. Subscribe to a Subscription.

```
cargo run -- --command subscribe
```

7. Unsubscribe from a Subscription.

```
cargo run -- --command unsubscribe
```
