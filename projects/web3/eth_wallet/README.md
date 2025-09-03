# Eth-Wallet: A Sample Trusted Application for Wallet Abstraction and Transaction Signing

This repository provides a reference implementation of an Ethereum wallet as a
Trusted Application (TA) written in Rust. The primary goal is to ensure that
secret credentials (such as private keys) remain securely within the Trusted
Execution Environment (TEE) throughout their entire lifecycle, enhancing
security and privacy for Ethereum-based operations. This reference
implementation can be extended to support additional wallet features or adapted
to other blockchain platforms with similar requirements for secure key
management. The implementation provides basic wallet abstractions, including:

- Key Generation: Securely generating random seeds within the TEE.
- Key Derivation: Deriving keys from seeds within the TEE.
- Key Persistency: Storing cryptographic keys securely in the TEE.
- Transaction Signing: Signing Ethereum transactions without exposing private
  keys to the normal world.
- Key Erase: Erasing keys when they are no longer needed.

## Security Assumptions

This demo assumes the following security foundations:

1. **Trusted Environment**:

   - The device supports OP-TEE as the TEE operating system.
   - Both the TEE OS and the Trusted Application (TA) are considered secure and
     trusted.

2. **Hardware-Specific Security Capabilities**
   - The hardware provides secure storage capabilities to protect cryptographic
     keys.
   - The device includes secure display capabilities (or a Multi-Factor
     Authentication device as alternative) for secure user interface. (MFA
     integration is planned for another demo project)
   - Note that these capabilities depend on specific hardware implementations.
     While this demo provides a default implementation, it should be customized
     to suit the target hardware.

### Important Notes on Security Design

This demo focuses on showcasing core functionalities and may not implement all
security measures required for a production-grade key custodian solution across
the entire key lifecycle. Developers should address the following considerations
when adapting this demo for real-world use cases:

- **Secure User Interface**:  
  In the `create_wallet` function, the mnemonic is returned to the Normal World
  for backup. This approach is inherently risky. For production systems, it is
  strongly recommended to display the mnemonic on a Trusted UI or secure
  display. Additionally, transactions should be confirmed by the user through
  this secure display. As secure display implementations are hardware-specific,
  this demo does not include such functionality.

- **Secure Storage Limitations**:  
  Keys in this demo are stored in an encrypted file on the Normal World File
  System. While this approach ensures basic protection, root access in the
  Normal World could delete this file, leading to key loss. For production
  scenarios, consider more reliable storage solutions like Replay Protected
  Memory Block (RPMB), which is hardware-specific and not included in this demo.

For developers, please note that this demo is intended as a foundational
reference and must be enhanced with hardware-specific adaptations for
production-grade security.

## Structure

- [TA](./ta): The Trusted Application (TA) that performs all secure operations
  related to the wallet. This component runs within the TrustZone TEE, ensuring
  that secret credentials never leave the secure environment.
- [CA](./host): The Client Application (CA) that runs in the normal world and
  communicates with the TA. It is responsible for user interaction and
  non-sensitive operations.
- [Proto](./proto): Contains shared structures and definitions used by both the
  TA and CA to facilitate communication between the two environments.

## Setup

To set up the environment, follow the instructions in the
[Apache Teaclave™ TrustZone SDK README](https://github.com/apache/teaclave-trustzone-sdk/blob/master/README.md).

## Functionalities

- **Create Wallet**: Generate a new Ethereum wallet with a unique ID.
- **Derive Address**: Derive an Ethereum address from a wallet.
- **Sign Transaction**: Sign Ethereum transactions securely within the TEE.
- **Remove Wallet**: Delete a wallet and its associated keys from the TEE.

## Usage

### Build

```
$ cd projects/eth_wallet-rs
$ make
```

### Run

After QEMU boots:

```bash
Welcome to Buildroot, type root or test to login
buildroot login: root
# mkdir shared && mount -t 9p -o trans=virtio host shared
# cd shared/
# ls
be2dc9a0-02b4-4b33-ba21-9964dbdf1573.ta
eth_wallet-rs
# cp be2dc9a0-02b4-4b33-ba21-9964dbdf1573.ta /lib/optee_armtz/
# ./eth_wallet-rs
```

### Command-Line Interface

```bash
A simple Ethereum wallet based on TEE

USAGE:
eth_wallet-rs <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    create-wallet       Create a new wallet
    derive-address      Derive an address from a wallet
    help                Prints this message or the help of the given subcommand(s)
    remove-wallet       Remove a wallet
    sign-transaction    Sign a transaction
    test                Run tests
```

## Example Commands

### Create a Wallet

```bash
# ./eth_wallet-rs create-wallet
```

**CA Output:**

```text
CA: command: CreateWallet
CA: invoke_command success
Wallet ID: aa5798a1-3c89-4708-b316-712aea4f59e2
```

**TA Output:**

```text
[+] TA create
[+] TA open session
[+] TA invoke command
[+] Wallet created: Wallet { id: aa5798a1-3c89-4708-b316-712aea4f59e2, entropy: [...] }
[+] Wallet ID: aa5798a1-3c89-4708-b316-712aea4f59e2
[+] Wallet saved in secure storage
```

### Derive an Address

```bash
# ./eth_wallet-rs derive-address -w aa5798a1-3c89-4708-b316-712aea4f59e2
```

**CA Output:**

```text
CA: command: DeriveAddress
CA: invoke_command success
Address: 0x7ca2b64a29bbf7a77bf8a3187ab09f50413826ea
Public key: 03e1289e07eca6fe47c4825ea52f7cd27e3143ac5d65d5842aa5f59b5eba2d58df
```

**TA Output:**

```text
[+] TA invoke command
[+] Deriving address: secure object loaded
[+] Wallet::derive_pub_key(): pub key: "xpub6FhY8TmVeQ6Yo5ViNX6LK3mM66nMJDe4ZumHmznLNRkK2wEhGoEjaossvKmjgETpFHNGs9CFjUS7HK1un9Djzw9jfsukyNxu53b87abRJUv"
[+] Wallet::derive_pub_key(): non-extended pub key: 03e1289e07eca6fe47c4825ea52f7cd27e3143ac5d65d5842aa5f59b5eba2d58df
[+] Wallet::derive_address(): address: [124, 162, 182, 74, 41, 187, 247, 167, 123, 248, 163, 24, 122, 176, 159, 80, 65, 56, 38, 234]
[+] Deriving address: address: [124, 162, 182, 74, 41, 187, 247, 167, 123, 248, 163, 24, 122, 176, 159, 80, 65, 56, 38, 234]
[+] Deriving address: public key: [3, 225, 40, 158, 7, 236, 166, 254, 71, 196, 130, 94, 165, 47, 124, 210, 126, 49, 67, 172, 93, 101, 213, 132, 42, 165, 245, 155, 94, 186, 45, 88, 223]
```

### Sign a Transaction

```bash
# ./eth_wallet-rs sign-transaction -t 0xc0ffee254729296a45a3885639AC7E10F9d54979 -v 100 -w aa5798a1-3c89-4708-b316-712aea4f59e2
```

**CA Output:**

```text
CA: command: SignTransaction
CA: invoke_command success
Signature: "f86380843b9aca0082520894c0ffee254729296a45a3885639ac7e10f9d5497964802ea0774fc5a364c3d7e3f4e039f8da96b66fb0a5d51cad7524e54a0c9013fb473304a033922ecf964f02c6ebdd7380bc86fe759b65c87dc9e09677d983622e35334931"
```

**TA Output:**

```text
[+] TA invoke command
[+] Sign transaction: secure object loaded
[+] Wallet::derive_prv_key() finished
[+] sign_transaction: signed transaction bytes: [248, 99, 128, 132, 59, 154, 202, 0, 130, 82, 8, 148, 192, 255, 238, 37, 71, 41, 41, 106, 69, 163, 136, 86, 57, 172, 126, 16, 249, 213, 73, 121, 100, 128, 46, 160, 119, 79, 197, 163, 100, 195, 215, 227, 244, 224, 57, 248, 218, 150, 182, 111, 176, 165, 213, 28, 173, 117, 36, 229, 74, 12, 144, 19, 251, 71, 51, 4, 160, 51, 146, 46, 207, 150, 79, 2, 198, 235, 221, 115, 128, 188, 134, 254, 117, 155, 101, 200, 125, 201, 224, 150, 119, 217, 131, 98, 46, 53, 51, 73, 49]
[+] Sign transaction: signature: [248, 99, 128, 132, 59, 154, 202, 0, 130, 82, 8, 148, 192, 255, 238, 37, 71, 41, 41, 106, 69, 163, 136, 86, 57, 172, 126, 16, 249, 213, 73, 121, 100, 128, 46, 160, 119, 79, 197, 163, 100, 195, 215, 227, 244, 224, 57, 248, 218, 150

, 182, 111, 176, 165, 213, 28, 173, 117, 36, 229, 74, 12, 144, 19, 251, 71, 51, 4, 160, 51, 146, 46, 207, 150, 79, 2, 198, 235, 221, 115, 128, 188, 134, 254, 117, 155, 101, 200, 125, 201, 224, 150, 119, 217, 131, 98, 46, 53, 51, 73, 49]
```

### Remove a Wallet

```bash
# ./eth_wallet-rs remove-wallet -w aa5798a1-3c89-4708-b316-712aea4f59e2
```

**CA Output:**

```text
CA: command: RemoveWallet
CA: invoke_command success
Wallet removed
```

**TA Output:**

```text
[+] TA invoke command
[+] Removing wallet: secure object loaded
[+] Wallet removed from secure storage
```
