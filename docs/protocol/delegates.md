# Delegates

When users send transactions inside the Mugraph network, they do it through **Delegators**. They are similar to Payment Networks in the tradicional banking system, but with some critical differences:

- Delegates hold funds in a fully auditable Smart Contract Vault, held in the Layer 1.
- Delegates can not spend user funds without their authorization.
- Delegates are **blind**, meaning they don't know who is transacting.
- Delegates provide **group anonimity** for their users, signing transactions on behalf of them.

An user can, and usually will, hold balance in multiple Delegates at once, and they do not need to have balance in a Delegate to receive payments there.

Essentially, a Delegate has four main roles:

1. **Handle Invoices:** delegates can receive Invoices, or requests to receive a certain payment.
2. **Handle Payments:** invoices are settled by payments, which can be settled by either a Layer 1 transaction or an internal transaction inside the Delegate itself.
3. **Handle Commits:** users can commit the state of the Vault on the blockchain to mint funds inside the delegate.
4. **Sign External Transactions:** delegates can sign external transactions on behalf of users, in order to route payments between them and the outside world.