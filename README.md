# ODESEC
On-chain Database of Emergency Security Event Contact, Whitehat is able to quickly establish a connection with the protocol


## Get ssl Certificate
https://github.com/srvrco/getssl

## Generate the SNARK Proof of the Certificate
Setup environment, Using the [Bonsai](https://dev.risczero.com/api/generating-proofs/remote-proving) proving service.
```bash
export BONSAI_API_KEY=<your-api-key>
export BONSAI_API_URI=<bonsai_url> # now is https://api.bonsai.xyz/
```
Now, you can run the cli to generate the proof.
```bash
RUST_LOG=INFO cargo run -r -- cert -p -c <your-certificate-file>
```

## Generate the zkp of the POC
1. Write your own poc file like [POC template](https://github.com/0xHackedLabs/PoC)
2. Run the cli to generate the proof.
```bash
cargo run -r  -- exploit -r <rpc_url> <your-poc-file>
```
### example
There are two example contracts deployed on the sepolia network
- [MockUSDC](./dapp/packages/hardhat/contracts/mock/MockUSDC.sol) **0x190CaCC70Ba6C8696b6144D67Acf4F5BEE77f713**
- [Victim Contract](./dapp/packages/hardhat/contracts/mock/TargetLoan.sol) **0xD856e309337dea0D14001C2853D23c9a2e384f8D**
```bash
RUST_LOG=INFO cargo run -r  -- exploit -r https://eth-sepolia.g.alchemy.com/v2/PwB1oLC0AVk2wiLTAzskCYGoOGm65bsn -p poc.sol
```
## Running the Frontend Locally
```bash
cd dapp
yarn start
```

## Thanks
- [risc0](https://github.com/risc0/risc0)
- [revm](https://github.com/bluealloy/revm)
- [zeth](https://github.com/risc0/zeth)
- [scaffold-eth-2](https://github.com/scaffold-eth/scaffold-eth-2)