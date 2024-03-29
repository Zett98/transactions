# Simple transaction system 

this is a simple transaction system that can handle 5 different types of transactions described below. 

| type of transaction | client_id : u16 | transaction_id : u32, globally unique | amount: unsigned decimal with scale = 4 |
| :------------------: | :------------------:| :------------------:| :------------------:|
| deposit             | 1             | 2             | 55.1             |
| withdrawal             | 1             | 5          | 55.1             |
| dispute             | 1             | 2          |             |
| resolve             | 1             | 2            |            |
| chargeback             | 1             | 2             |            |

#### Deposit:
simple deposit to the account.
- duplicate deposits are ignored
- frozen accounts can't accept deposits
  
### Withdrawal:
withdrawal from the account. 
 - when `account.available_balance < withdrawal_amount` transaction is ignored
 - duplicate withdrawals are ignored 
 - withdrawals are ignored for frozen accounts.
  
### Dispute:
Clients claim that a transaction referenced by it's `transaction_id` was erroneus.
- as the system can only receive disputes from it's partners it's safe to assume that it's only concerning deposits
- deposit amount of the referenced transaction is put on hold for that particular client account
- dispute is ignored if: 
    - it's referencing a nonexistant transaction
    - transaction is already being disputed 
    - transaction was already resolved/chargedback
- this transaction can create a liability for the `account`. 


### Resolve
Represents resolution to dispute claim, references the `transaction_id` of transaction being disputed.
- on success releases amount that was put on hold for particular client's account. 
- ignored if:
    - transaction is already resolved/chargedback
    - transaction is not disputed 
    - references non-existant transaction

### Chargeback
Represents a client reversing a transaction refenced by `transaction_id`.
- on success subtracts `transaction.amount` from `client.account.held` and freezes the account.
- ignored if:
    - transaction is already resolved/chargedback
    - transaction is not disputed 
    - references non-existant transaction

## Frozen account: 
Frozen account will not accept any `withdrawal`'s or `deposit`'s but still can process disputes.

## Shortcuts: 
 - Didn't provide a persistent storage for the transaction log.
 - errors returned by the ledger are not logged/saved to the storage.
 - because everything is held in memory in this implementation i didn't use async for the ledger internals. 
 - this implementation is not parallelized as we only ready a simple csv file, but can be: 
    - multiple workers that process distinct sets of client id's. 
    - inputs can be partitioned by client_id.


## Choices made for this specific implementation:
- Inputs accepted only via csv input files.
- if we encounter invalid/unknown transaction in the file we just ignore it.
- deposit or withdrawals with `amount <= 0 ` are ignored.
- balances are represented using fixed point numbers as we only do subtraction and additions.
- in-memory ledger uses BtreeMap instead of hashmap to avoid linear behaviour on growth and use a bit less memory. 
- csv file reads and writes are buffered, but in-memory ledger can grow without bounds. 

## How to use: 

```bash
cargo run --release -- <file.csv> > <output-file.csv>
```
