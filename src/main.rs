use std::{error::Error, thread::sleep, time::Duration};

use clap::Parser;
use mpl_token_metadata::{instruction::create_metadata_accounts, state::Creator};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    account::Account,
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair},
    signer::Signer,
    transaction::Transaction,
};

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let rpc = &RpcClient::new(args.rpc);
    let signer = &read_keypair_file(args.signer)?;

    let mints_missing_metadata = vec![
        "2gJPv2yK4MWEaRDfxmFJo7sGBPQBgMrasMfvC8zVSrzr",
        "4zA73hGR393FKdPTHcwpTXQW1wYXy7qWrNW1w4SibQUg",
        "5TGsXTng16ic9mJrY6QA6t7uqf4X4iwkiHMUbcuJKWay",
        "AdD27SLKumVDF5KwAFZk216T1yaUZTTUqNeTJybThxT5",
        "DFfFTGsrMg99nQ3mSiDBrUVsvZ1pqR7nK55kNQpWaZYK",
    ];

    for mint_address in mints_missing_metadata {
        let mint_address = mint_address.parse().unwrap();
        match find_or_create_metadata_account(rpc, signer, mint_address) {
            Some(_) => eprintln!("{} Okay", mint_address),
            None => eprintln!("{} Fail", mint_address),
        }
    }
    Ok(())
}

fn find_or_create_metadata_account(
    rpc: &RpcClient,
    signer: &Keypair,
    mint_address: Pubkey,
) -> Option<Account> {
    let metadata_address = find_metadata_address(mint_address);
    match rpc.get_account(&metadata_address) {
        Ok(account) => {
            eprintln!("{} has metadata", mint_address);
            Some(account)
        }
        Err(err) => {
            eprintln!("{} probably needs metadata: {:?}", mint_address, err);
            create_metadata_account(rpc, signer, mint_address)
        }
    }
}

fn create_metadata_account(
    rpc: &RpcClient,
    signer: &Keypair,
    mint_address: Pubkey,
) -> Option<Account> {
    let metadata_address = find_metadata_address(mint_address);
    eprintln!(
        "{} creating metadata account: {}",
        mint_address, metadata_address
    );

    let (name, uri) = find_name_and_uri_for(mint_address);
    let creators = Some(vec![Creator {
        address: signer.pubkey(),
        verified: true,
        share: 100,
    }]);

    let instructions = vec![create_metadata_accounts(
        mpl_token_metadata::id(),
        metadata_address,
        mint_address,
        signer.pubkey(),
        signer.pubkey(),
        signer.pubkey(),
        name,
        "SLR".to_owned(),
        uri,
        creators,
        500,
        true,
        true,
    )];

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&signer.pubkey()),
        &[signer],
        rpc.get_latest_blockhash().unwrap(),
    );

    let sim_result = rpc.simulate_transaction(&tx);
    if sim_result.is_err() {
        eprintln!("{} {:?}", mint_address, sim_result);
        return None;
    }

    let result = rpc.send_transaction(&tx);
    eprintln!("{} {:?}", mint_address, result);

    let mut tries = 0;
    loop {
        tries += 1;
        match rpc.get_account(&metadata_address) {
            Ok(account) => {
                break Some(account);
            }
            Err(err) => {
                sleep(Duration::from_millis(500));
                if tries > 5 {
                    eprintln!("{} {:?}", mint_address, err);
                    break None;
                }
            }
        }
    }
}

fn find_name_and_uri_for(mint_address: Pubkey) -> (String, String) {
    let mint_address = mint_address.to_string();
    let (name, uri) = match mint_address.as_str() {
        "2gJPv2yK4MWEaRDfxmFJo7sGBPQBgMrasMfvC8zVSrzr" => (
            "S-DF5 the Hideous",
            "https://arweave.net/x29w7fBl_QoSX6n4hXBrBU1GclCsH_o89dQrF2v3VwQ",
        ),
        "4zA73hGR393FKdPTHcwpTXQW1wYXy7qWrNW1w4SibQUg" => (
            "1-QQ the Happy-Go-Lucky",
            "https://arweave.net/NmnK22045rFFCWPMWpjD5a1bLXTlowUdLluYi3VQXFU",
        ),
        "5TGsXTng16ic9mJrY6QA6t7uqf4X4iwkiHMUbcuJKWay" => (
            "98-4 the Frightened",
            "https://arweave.net/_x1F-vgZQ1bJxyDrIPFWnQ-g9I2DYwFPefnKyAyVJj0",
        ),
        "AdD27SLKumVDF5KwAFZk216T1yaUZTTUqNeTJybThxT5" => (
            "K-19 the Prized",
            "https://arweave.net/VVBYgw_0jNDeqCltpIvQEiOnicdfxDw3iRGoyjrIjdM",
        ),
        "DFfFTGsrMg99nQ3mSiDBrUVsvZ1pqR7nK55kNQpWaZYK" => (
            "X-RU5 the Lost",
            "https://arweave.net/5lolOE2pq4UsTHbTLuqWAi3R39o0ILTv3udK2ZzB6hE",
        ),
        _ => todo!("todo: implemented find_metadata_for {} ", mint_address),
    };

    (name.to_string(), uri.to_string())
}

fn find_metadata_address(mint: Pubkey) -> Pubkey {
    let (address, _bump) = Pubkey::find_program_address(
        &[
            mpl_token_metadata::state::PREFIX.as_bytes(),
            mpl_token_metadata::id().as_ref(),
            mint.as_ref(),
        ],
        &mpl_token_metadata::id(),
    );
    address
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, short, default_value = "https://api.mainnet-beta.solana.com")]
    rpc: String,
    #[clap(long, short)]
    signer: String,
}
