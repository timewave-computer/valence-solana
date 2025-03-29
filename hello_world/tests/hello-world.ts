import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { expect } from "chai";
import { HelloWorld } from "../target/types/hello_world";

describe("hello-world", () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.HelloWorld as Program<HelloWorld>;
  const wallet = provider.wallet;

  // Create a keypair for our greeting account
  const greetingAccount = anchor.web3.Keypair.generate();

  it("Initializes with a greeting", async () => {
    // Initial greeting
    const greeting = "Hello, Solana!";

    // Initialize the greeting account
    const tx = await program.methods
      .initialize(greeting)
      .accounts({
        greetingAccount: greetingAccount.publicKey,
        user: wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([greetingAccount])
      .rpc();

    console.log("Transaction signature:", tx);

    // Fetch the created account
    const account = await program.account.greetingAccount.fetch(
      greetingAccount.publicKey
    );

    // Check if the greeting is correct
    expect(account.greeting).to.equal(greeting);
    expect(account.counter.toNumber()).to.equal(0);
  });

  it("Updates the greeting", async () => {
    // New greeting
    const newGreeting = "Hello, Anchor!";

    // Update the greeting
    const tx = await program.methods
      .updateGreeting(newGreeting)
      .accounts({
        greetingAccount: greetingAccount.publicKey,
        user: wallet.publicKey,
      })
      .rpc();

    console.log("Transaction signature:", tx);

    // Fetch the updated account
    const account = await program.account.greetingAccount.fetch(
      greetingAccount.publicKey
    );

    // Check if the greeting was updated
    expect(account.greeting).to.equal(newGreeting);
    expect(account.counter.toNumber()).to.equal(1);
  });

  it("Says hello and increments counter", async () => {
    // Fetch the account before saying hello
    const accountBefore = await program.account.greetingAccount.fetch(
      greetingAccount.publicKey
    );
    const counterBefore = accountBefore.counter.toNumber();

    // Say hello
    const tx = await program.methods
      .sayHello()
      .accounts({
        greetingAccount: greetingAccount.publicKey,
      })
      .rpc();

    console.log("Transaction signature:", tx);

    // Fetch the account after saying hello
    const accountAfter = await program.account.greetingAccount.fetch(
      greetingAccount.publicKey
    );

    // Check if the counter was incremented
    expect(accountAfter.counter.toNumber()).to.equal(counterBefore + 1);
  });
}); 