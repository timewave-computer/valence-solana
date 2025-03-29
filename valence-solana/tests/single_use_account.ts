import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { PublicKey, Keypair, SystemProgram } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID, getAssociatedTokenAddress, createMint, mintTo } from '@solana/spl-token';
import { expect } from 'chai';

describe('Single-Use Account Program', () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // Load the program
  const program = anchor.workspace.SingleUseAccount as Program;
  
  // Setup accounts and variables
  const authority = provider.wallet;
  const authToken = Keypair.generate().publicKey;
  let singleUseAccountPda: PublicKey;
  let singleUseAccountBump: number;
  let testLibrary: PublicKey;
  let testDestination: PublicKey;
  
  // For testing expiration functionality
  const ONE_DAY_IN_SECONDS = 86400;
  const now = Math.floor(Date.now() / 1000);
  const futureExpiration = now + ONE_DAY_IN_SECONDS;

  before(async () => {
    // Find PDA for single-use account
    [singleUseAccountPda, singleUseAccountBump] = await PublicKey.findProgramAddressSync(
      [
        Buffer.from('single_use_account'),
        authority.publicKey.toBuffer(),
      ],
      program.programId
    );

    // Create a fake library (using a new keypair)
    testLibrary = Keypair.generate().publicKey;
    
    // Create a test destination
    testDestination = Keypair.generate().publicKey;
  });

  it('Initialize Single-Use Account', async () => {
    // Initialize the single-use account
    const tx = await program.methods
      .initialize({
        authToken,
        requiredDestination: testDestination,
        expirationTime: new anchor.BN(futureExpiration),
      })
      .accounts({
        authority: authority.publicKey,
        singleUseAccount: singleUseAccountPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    
    console.log('Initialize transaction signature', tx);
    
    // Fetch the account and validate
    const singleUseAccount = await program.account.singleUseAccount.fetch(singleUseAccountPda);
    expect(singleUseAccount.authority.toString()).to.equal(authority.publicKey.toString());
    expect(singleUseAccount.authToken.toString()).to.equal(authToken.toString());
    expect(singleUseAccount.wasUsed).to.be.false;
    expect(singleUseAccount.requiredDestination.toString()).to.equal(testDestination.toString());
    expect(singleUseAccount.expirationTime.toNumber()).to.equal(futureExpiration);
  });

  it('Register and Approve Library', async () => {
    // Register the library
    const registerTx = await program.methods
      .registerLibrary({
        library: testLibrary,
        autoApprove: true,
      })
      .accounts({
        authority: authority.publicKey,
        singleUseAccount: singleUseAccountPda,
      })
      .rpc();
    
    console.log('Register library transaction signature', registerTx);
    
    // Fetch the account and validate
    const singleUseAccount = await program.account.singleUseAccount.fetch(singleUseAccountPda);
    expect(singleUseAccount.approvedLibraries.some(lib => lib.toString() === testLibrary.toString())).to.be.true;
  });

  it('Cannot use account after it has been used', async () => {
    // First, mark the account as used (in a real scenario, this would happen through execute())
    // For this test, we would need to create a mock execute transaction
    console.log('This test would verify that once an account is used, it cannot be used again');
    
    // In a real test, we would:
    // 1. Execute an operation that marks the account as used
    // 2. Attempt to execute another operation and expect it to fail
    // 3. Validate the error is AccountAlreadyUsed
  });

  it('Cannot recover funds before expiration', async () => {
    // This test would verify that emergency recovery fails if the account hasn't expired
    console.log('This test would verify that emergency recovery fails before expiration');
    
    // In a real test, we would:
    // 1. Attempt to execute emergency_recover
    // 2. Expect it to fail with AccountNotExpired error
  });

  // Additional tests would include:
  // - Testing execution with different libraries
  // - Testing the destination validation
  // - Testing emergency recovery after expiration
  // - Testing token transfers to the required destination
}); 