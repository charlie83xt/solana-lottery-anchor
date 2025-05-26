import BN from "bn.js";
import assert from "assert";
import * as web3 from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { assert } from "chai";
import { SolanaLottery } from "../target/types/solana_lottery";
import type { SolanaLottery } from "../target/types/solana_lottery";
describe("solana-lottery", () => {
  // Configure the client to use the local cluster
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.SolanaLottery as anchor.Program<SolanaLottery>;
  
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);


  const program = anchor.workspace.SolanaLottery as Program<SolanaLottery>;


  // Accounts
  let lottery: anchor.web3.Keypair;
  let devWallet1: anchor.web3.Keypair;
  let devWallet2: anchor.web3.Keypair;


  it("Initializes a lottery", async () => {
    // Initialize accounts
    lottery = anchor.web3.Keypair.generate();
    devWallet1 = anchor.web3.Keypair.generate();
    devWallet2 = anchor.web3.Keypair.generate();


    // Define parameters
    const ticketPrice = new anchor.BN(1_000_000_000); // 1 SOL in lamports
    const maxParticipants = 5;
    const duration = new anchor.BN(60); // 1-minute duration


    // Initialize the lottery
    await program.methods
      .initializeLottery(ticketPrice, maxParticipants, duration)
      .accounts({
        lottery: lottery.publicKey,
        authority: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([lottery])
      .rpc();


    // Fetch the lottery account
    const lotteryAccount = await program.account.lottery.fetch(lottery.publicKey);


    // Assertions
    assert.equal(lotteryAccount.authority.toBase58(), provider.wallet.publicKey.toBase58());
    assert.equal(lotteryAccount.ticketPrice.toNumber(), ticketPrice.toNumber());
    assert.equal(lotteryAccount.maxParticipants, maxParticipants);
    assert.equal(lotteryAccount.participants.length, 0);
  });


  it("Allows users to buy tickets", async () => {
    const participants = Array.from({ length: 5 }, () => anchor.web3.Keypair.generate());


    for (const participant of participants) {
      // Airdrop SOL to the participant
      const signature = await provider.connection.requestAirdrop(
        participant.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await provider.connection.confirmTransaction(signature);


      // Buy a ticket
      await program.methods
        .buyTicket()
        .accounts({
          lottery: lottery.publicKey,
          buyer: participant.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([participant])
        .rpc();
    }


    // Fetch the lottery account
    const lotteryAccount = await program.account.lottery.fetch(lottery.publicKey);


    // Assertions
    assert.equal(lotteryAccount.participants.length, 5);
    participants.forEach((p, i) => {
      assert.equal(lotteryAccount.participants[i].toBase58(), p.publicKey.toBase58());
    });
  });


  it("Draws a winner and resets the lottery", async () => {
    // Draw a winner
    await program.methods
      .drawWinner()
      .accounts({
        lottery: lottery.publicKey,
        authority: provider.wallet.publicKey,
        devWallet1: devWallet1.publicKey,
        devWallet2: devWallet2.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();


    // Fetch the lottery account
    const lotteryAccount = await program.account.lottery.fetch(lottery.publicKey);


    // Assertions
    assert.ok(lotteryAccount.winner !== null, "Winner should be set");
    console.log("Winner:", lotteryAccount.winner.toBase58());


    // Ensure lottery resets
    assert.equal(lotteryAccount.participants.length, 0, "Participants should be cleared");
    assert.ok(lotteryAccount.endTime.toNumber() > Date.now() / 1000, "End time should be updated");
  });
});






