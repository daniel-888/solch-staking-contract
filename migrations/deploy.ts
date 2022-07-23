// Migrations are an early feature. Currently, they're nothing more than this
// single deploy script that's invoked from the CLI, injecting a provider
// configured from the workspace's Anchor.toml.

import * as anchor from '@project-serum/anchor';
import { AccountLayout, TOKEN_PROGRAM_ID, Token } from '@solana/spl-token';
import {IDL} from '../target/types/solch_staking_contract';
import { Commitment, ConnectionConfig} from '@solana/web3.js'
const { SystemProgram, Keypair, PublicKey, Connection } = anchor.web3;
const token_mint = 'EKSM2sjtptnvkqq79kwfAaSfVudNAtFYZSBdPe5jeRSt';
const PROGRAM_ID = '6Pf6bCr94Y8UFwDWneMbyWNUDtv9LRowVwGR9DzKUACD';
const CLUSTER_API = 'https://misty-green-dream.solana-mainnet.quiknode.pro/324b64b0ef2638385e0facb9a7cde25ed22f91f9/';
module.exports = async function (provider) {
  // Configure client to use the provider.

  const connection = new Connection(CLUSTER_API, {
    skipPreflight: true,
    preflightCommitment: 'confirmed' as Commitment 
  } as ConnectionConfig);
  
  const runProvider =  new anchor.Provider(connection,  provider.wallet, {
      skipPreflight: true,
      preflightCommitment: 'confirmed' as Commitment 
  } as ConnectionConfig)


  const program = new anchor.Program(IDL, new PublicKey(PROGRAM_ID), runProvider);
  let [vaultPDA, _nonce] = await anchor.web3.PublicKey.findProgramAddress(
    [Buffer.from('rewards vault')],
    program.programId
  );

  const aTokenAccount = new Keypair();
  const aTokenAccountRent = await connection.getMinimumBalanceForRentExemption(
    AccountLayout.span
  )



  const tx = await program.rpc.createVault(
     _nonce, {
      accounts: {
        vault: vaultPDA,
        admin: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId
      },
      signers: [aTokenAccount],
      instructions: [
        SystemProgram.createAccount({
          fromPubkey: provider.wallet.publicKey,
          newAccountPubkey: aTokenAccount.publicKey,
          lamports: aTokenAccountRent,
          space: AccountLayout.span,
          programId: TOKEN_PROGRAM_ID
        }),
        Token.createInitAccountInstruction(
          TOKEN_PROGRAM_ID,
          new PublicKey(token_mint),
          aTokenAccount.publicKey,
          vaultPDA
        )
      ]
    } 
  );
  
  console.log('vaultPda', vaultPDA.toString(), 'nonce', _nonce);
  console.log('tokenAccount', aTokenAccount.publicKey.toString());

  let [poolData, nonce] = await anchor.web3.PublicKey.findProgramAddress(
    [Buffer.from('pool data')],
    program.programId
  );
  console.log('poolData', poolData);
  const tx_data = await program.rpc.createDataAccount(nonce, {
    accounts: {
      data: poolData,
      admin: provider.wallet.publicKey,
      systemProgram: SystemProgram.programId
    }
  });
  console.log(tx_data);
}