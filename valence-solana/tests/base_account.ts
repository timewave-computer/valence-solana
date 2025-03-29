import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { PublicKey, Keypair, SystemProgram } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID, getAssociatedTokenAddress, createMint, createAssociatedTokenAccount } from '@solana/spl-token';
import { expect } from 'chai';

describe('Base Account Program', () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // Load the program
  const program = anchor.workspace.BaseAccount as Program;
  
  // Setup accounts and variables
  const authority = provider.wallet;
  const authToken = Keypair.generate().publicKey;
  let baseAccountPda: PublicKey;
  let baseAccountBump: number;
  let testLibrary: PublicKey;
  let testMint: PublicKey;
  let tokenAccount: PublicKey;

  before(async () => {
    // Find PDA for base account
    [baseAccountPda, baseAccountBump] = await PublicKey.findProgramAddressSync(
      [
        Buffer.from('base_account'),
        authority.publicKey.toBuffer(),
      ],
      program.programId
    );

    // Create a fake library (using a new keypair)
    testLibrary = Keypair.generate().publicKey;
  });

  it('Initialize Base Account', async () => {
    // Initialize the base account
    const tx = await program.methods
      .initialize({ authToken })
      .accounts({
        authority: authority.publicKey,
        baseAccount: baseAccountPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    
    console.log('Initialize transaction signature', tx);
    
    // Fetch the account and validate
    const baseAccount = await program.account.baseAccount.fetch(baseAccountPda);
    expect(baseAccount.authority.toString()).to.equal(authority.publicKey.toString());
    expect(baseAccount.authToken.toString()).to.equal(authToken.toString());
    expect(baseAccount.instructionCount.toNumber()).to.equal(0);
    expect(baseAccount.tokenAccountCount.toNumber()).to.equal(0);
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
        baseAccount: baseAccountPda,
      })
      .rpc();
    
    console.log('Register library transaction signature', registerTx);
    
    // Fetch the account and validate
    const baseAccount = await program.account.baseAccount.fetch(baseAccountPda);
    expect(baseAccount.approvedLibraries.has(testLibrary.toString())).to.be.true;
  });

  it('Create Token Account', async () => {
    // Create a test mint
    const mintAuthority = Keypair.generate();
    
    // Airdrop some SOL to the mint authority for the transaction fees
    const airdropSig = await provider.connection.requestAirdrop(
      mintAuthority.publicKey,
      anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdropSig);
    
    // Create the mint
    testMint = await createMint(
      provider.connection,
      mintAuthority,
      mintAuthority.publicKey,
      null,
      9 // 9 decimals
    );
    
    // Find the associated token account for the base account PDA
    tokenAccount = await getAssociatedTokenAddress(
      testMint,
      baseAccountPda,
      true // allowOwnerOffCurve
    );
    
    // Create the token account
    const createTx = await program.methods
      .createTokenAccount(testMint)
      .accounts({
        authority: authority.publicKey,
        baseAccount: baseAccountPda,
        mint: testMint,
        tokenAccount: tokenAccount,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();
    
    console.log('Create token account transaction signature', createTx);
    
    // Fetch the base account and validate
    const baseAccount = await program.account.baseAccount.fetch(baseAccountPda);
    expect(baseAccount.tokenAccountCount.toNumber()).to.equal(1);
  });

  // Note: Execute Instruction and Transfer Tokens tests would require more setup
  // and would depend on specific implementation details and test environment,
  // so they are omitted for simplicity.
}); 