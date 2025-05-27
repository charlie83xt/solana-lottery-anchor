import BN from "bn.js";
import * as web3 from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import { Connection, PublicKey, clusterApiUrl } from '@solana/web3.js';
import { Program } from "@coral-xyz/anchor"
import type { SolanaLottery } from "../target/types/solana_lottery";

// Configure the client to use the local cluster
anchor.setProvider(anchor.AnchorProvider.env());

const program = anchor.workspace.SolanaLottery as anchor.Program<SolanaLottery>;

const PROGRAM_ID = new PublicKey("7VD5huPrnENoik7jMZijXnnnVrKayBY3rwk8BLULh5oQ");

// Client
console.log("My address:", program.provider.publicKey.toString());
// const balance = await program.provider.connection.getBalance(program.provider.publicKey);
// console.log(`My balance: ${balance / web3.LAMPORTS_PER_SOL} SOL`);

// const lotteryPubkey = new PublicKey("66tYyDnMHZLWYdXkR7hsYrxB4wb35FQtqNWhUvDbTHoa");

// Derive the Global State Pda
const [pda] = await PublicKey.findProgramAddressSync(
  [Buffer.from("global_state_v3")],
  PROGRAM_ID
);
console.log("Global State PDA:", pda.toBase58());
// const lotteryBuffer = new anchor.BN(4).toArrayLike(Buffer, 'le', 8);
// const [lotteryPda] = await PublicKey.findProgramAddress(
//   [Buffer.from("lottery"), lotteryBuffer],
//   PROGRAM_ID
// );
// console.log("Client derived PDA:", lotteryPda.toBase58())
// async getGlobalStatePDA(): Promise<void> {

// Fetch the globalstate account
const globalState = await program.account.globalState.fetch(pda);
console.log("Fetched Global State:", globalState);

// const currentNumber = await 
// const fetchedGlobalState = await program.account.fetch(globalState)