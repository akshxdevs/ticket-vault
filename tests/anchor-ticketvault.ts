import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AnchorTicketvault } from "../target/types/anchor_ticketvault";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { expect } from "chai";

describe("anchor-ticketvault", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace
    .anchorTicketvault as Program<AnchorTicketvault>;
  const user = provider.wallet as anchor.Wallet;
  const creator = provider.wallet as anchor.Wallet;
  let mint: anchor.web3.PublicKey;
  let userAta: anchor.web3.PublicKey;
  let vaultAta: anchor.web3.PublicKey;
  let eventAta: anchor.web3.PublicKey;
  let creatorAta: anchor.web3.PublicKey;
  let event: anchor.web3.PublicKey;
  let vault: anchor.web3.PublicKey;
  let ticket: anchor.web3.PublicKey;
  let amount = new anchor.BN(1_000_000_000);
  const ticketFee = new anchor.BN(1_000_000_000);
  it("Is initialized!", async () => {
    mint = await createMint(
      provider.connection,
      creator.payer,
      creator.publicKey,
      null,
      6
    );
    
    [event] = await PublicKey.findProgramAddress(
      [Buffer.from("event"), creator.publicKey.toBuffer()],
      program.programId
    );
    [vault] = await PublicKey.findProgramAddress(
      [Buffer.from("vault"),user.publicKey.toBuffer()],
      program.programId
    );
    [ticket] = await PublicKey.findProgramAddress(
      [Buffer.from("ticket"), event.toBuffer(), user.publicKey.toBuffer()],
      program.programId
    );
    eventAta = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        creator.payer,
        mint,
        event,
        true
      )
    ).address;
    vaultAta = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        user.payer,
        mint,
        vault,
        true
      )
    ).address;
    creatorAta = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        creator.payer,
        mint,
        creator.publicKey
      )
    ).address;

    userAta = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        user.payer,
        mint,
        user.publicKey
      )
    ).address;

    await mintTo(
      provider.connection,
      user.payer,
      mint,
      userAta,
      user.publicKey,
      100_000_000_000 
    );
    const eventDetails = "Coolie Movie Audio launch";
    const tx = await program.methods
      .initializeEvent(10, eventDetails, ticketFee, amount)
      .accounts({
        event: event,
        creator: creator.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log("Your transaction signature", tx);
  });
  it("Enroll Event",async()=>{
    const preUser = await provider.connection.getTokenAccountBalance(userAta);
    const preVault = await provider.connection.getTokenAccountBalance(vaultAta);

    const tx = await program.methods.encrollEvent().accounts({
      ticket: ticket,
      event: event,
      vault: vault,
      vaultAta: vaultAta,
      userAta: userAta,
      creatorAta: creatorAta,
      creator: creator.publicKey,
      user: user.publicKey,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID      
    }).signers([user.payer]).rpc();
    
    const postUser = await provider.connection.getTokenAccountBalance(userAta);
    const postVault = await provider.connection.getTokenAccountBalance(vaultAta);
    const ticketDetails = await program.account.ticket.fetch(ticket);
    console.log("Ticket Details: ",ticketDetails);
    console.log("User Balance (pre->post): ",preUser.value.amount,"->",postUser.value.amount,`(${Number(preUser.value.amount)/1e9} SOL -> ${Number(postUser.value.amount)/1e9} SOL)`);
    console.log("Vault Balance (pre->post): ",preVault.value.amount,"->",postVault.value.amount,`(${Number(preVault.value.amount)/1e9} SOL -> ${Number(postVault.value.amount)/1e9} SOL)`);

    const userDelta = BigInt(preUser.value.amount) - BigInt(postUser.value.amount);
    const vaultDelta = BigInt(postVault.value.amount) - BigInt(preVault.value.amount);

    console.log("User Delta:", userDelta.toString(), `(${Number(userDelta)/1e9} SOL)`);
    console.log("Vault Delta:", vaultDelta.toString(), `(${Number(vaultDelta)/1e9} SOL)`);

    expect(userDelta.toString()).to.equal(ticketFee.toString());
    expect(vaultDelta.toString()).to.equal(ticketFee.toString());

    console.log("Your transaction signature", tx);
  });
  
  it("Ticket Claim",async()=>{
    const tx = await program.methods.claimTicket().accounts({
      event: event,
      ticket: ticket,
      user: user.publicKey,
      systemProgram: SystemProgram.programId,
    }).signers([user.payer]).rpc();
    const ticketDetails = await program.account.ticket.fetch(ticket);
    console.log("Ticket Details: ",ticketDetails);
    console.log("Your transaction signature", tx);
    expect(ticketDetails.claimed.valueOf()).to.equal(true);
  });
});
