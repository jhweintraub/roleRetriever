# Role Retriever

Has this ever happened to you? You're writing tests for your smart contract and realize that you don't have permissions on someone else's contracts to configure the test. So you need to find an address has the specific role you're looking for? But you find out that there's no easy way to find an address that has those permissions, so you spend an hour searching through event logs for the `RoleGranted` event emission, only to find that its not for the role you wanted, or was later revoked? Well fear no more. This quick and dirty Rust program can solve all your problems.

Role Retriever does all the log searching for you and returns all the Access Control information for you, so that you can get back to writing and configuring your tests quicker.

# Usage

# Installation
