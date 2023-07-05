import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ClockworkProvider } from "@clockwork-xyz/sdk";
import { AutomatedVault } from "../target/types/automated_vault";
import { BN } from 'bn.js';
import { assert } from "chai";
const RECURRING_TRANSFER = 1;

describe("Automated_Vault", async() => {
  let threadId =  "vault1";
  let balance =  80;
  let target = 100;
  let label =  "Get a Madlad";
  let amount =  140;

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const wallet = provider.wallet;
  const program = anchor.workspace.AutomatedVault as Program<AutomatedVault>;
  const clockworkProvider = ClockworkProvider.fromAnchorProvider(provider);

  const [vault] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("vault"),Buffer.from(threadId)],
    program.programId
  );

  const [threadAuthority] = anchor.web3.PublicKey.findProgramAddressSync(
    [anchor.utils.bytes.utf8.encode("authority")],
    program.programId
  );
  const [threadAddress,threadBump] = clockworkProvider.getThreadPDA(
    threadAuthority,
    threadId
  );

  const[recurringTransfer_PDA] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("recurringTransfer"), Buffer.from(threadId)],
    program.programId
  );

  console.log("Vault:", vault.toBase58());
  console.log("Thread Authority:", threadAuthority.toBase58());
  console.log("Thread Address:", threadAddress.toBase58());
  console.log("Clockwork Program:",clockworkProvider.threadProgram.programId.toBase58());
  console.log("System Program: ", anchor.web3.SystemProgram.programId.toBase58());

  it("Vault Initialization!", async() => {
    await program.methods
    .initializeVault(
      Buffer.from(threadId),
      new BN(balance),
      new BN(target),
      label,
    ) 
    .accounts({
        owner: wallet.publicKey,
        vault: vault,
        clockworkProgram: clockworkProvider.threadProgram.programId,
        systemProgram: anchor.web3.SystemProgram.programId,
        thread: threadAddress,
        threadAuthority: threadAuthority,
    })
      .rpc();
  });

  it("Withdraw from Vault",  async() => {
    await program.methods
      .withdraw(Buffer.from(threadId), new BN(amount))
      .accounts({
        owner: wallet.publicKey,
        vault: vault,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
  });

  it("Close Vault", async() => {
    await program.methods
      .closeVault(Buffer.from(threadId))
      .accounts({
        owner: wallet.publicKey,
        vault: vault,
        thread: threadAddress,
        threadAuthority: threadAuthority,
        clockworkProgram: clockworkProvider.threadProgram.programId,
      })
      .rpc();
  });
});