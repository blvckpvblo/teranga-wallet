use bdk::{Wallet, wallet::AddressIndex, wallet::AddressInfo};
use bdk::database::MemoryDatabase;
use bdk::blockchain::{ElectrumBlockchain, Blockchain};
use bdk::SyncOptions;
use bdk::electrum_client::Client;
use bdk::bitcoin::{Network, Address};
use std::str::FromStr;
use bdk::SignOptions;

fn main() -> Result<(), Box<dyn std::error::Error>>  {
    let blockchain = ElectrumBlockchain::from(Client::new("ssl://electrum.blockstream.info:60002")?);
    let wallet = Wallet::new(
        "wpkh([<FINGERPRINT>/84h/1h/0h]<XPRV>/0/*)",
        Some("wpkh([<FINGERPRINT>/84h/1h/0h]<XPRV>/1/*)"),
        Network::Testnet,
        MemoryDatabase::default(),
    )?;

    // Get wallet address    
    let addr: AddressInfo = wallet.get_address(AddressIndex::New)?;
    print!("Address: {}\n", addr);

    // Sync to blockchain
    wallet.sync(&blockchain, SyncOptions::default())?;

    // Get balance
    let balance = wallet.get_balance()?;
    println!("Wallet balance in SAT: {}", balance.get_total());

    // Send BTC Tx
    let faucet_address = Address::from_str("<RECIPIENT ADDRESS>")?;
    let mut tx_builder = wallet.build_tx();
    tx_builder
        .add_recipient(faucet_address.script_pubkey(), balance.get_total() / 2)
        .enable_rbf();
    let (mut psbt, tx_details) = tx_builder.finish()?;
    println!("Transaction details: {:#?}", tx_details);

    // Sign Tx
    let finalized = wallet.sign(&mut psbt, SignOptions::default())?;
    assert!(finalized, "Tx has not been finalized");
    println!("Transaction Signed: {}", finalized);

    // Send Tx to the BTC network
    let raw_transaction = psbt.extract_tx();
    let txid = raw_transaction.txid();
    blockchain.broadcast(&raw_transaction)?;
    println!("Transaction sent! TXID: {txid}.\nExplorer URL: https://blockstream.info/testnet/tx/{txid}", txid = txid);

    Ok(())
}