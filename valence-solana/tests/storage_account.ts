import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { PublicKey, Keypair, SystemProgram } from '@solana/web3.js';
import { expect } from 'chai';

describe('Storage Account Program', () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // Load the program
  const program = anchor.workspace.StorageAccount as Program;
  
  // Setup accounts and variables
  const authority = provider.wallet;
  const authToken = Keypair.generate().publicKey;
  let storageAccountPda: PublicKey;
  let storageAccountBump: number;
  let testLibrary: PublicKey;
  
  const MAX_CAPACITY = 1024 * 10; // 10KB

  before(async () => {
    // Find PDA for storage account
    [storageAccountPda, storageAccountBump] = await PublicKey.findProgramAddressSync(
      [
        Buffer.from('storage_account'),
        authority.publicKey.toBuffer(),
      ],
      program.programId
    );

    // Create a fake library (using a new keypair)
    testLibrary = Keypair.generate().publicKey;
  });

  it('Initialize Storage Account', async () => {
    // Initialize the storage account
    const tx = await program.methods
      .initialize({
        authToken,
        maxCapacity: new anchor.BN(MAX_CAPACITY),
      })
      .accounts({
        authority: authority.publicKey,
        storageAccount: storageAccountPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    
    console.log('Initialize transaction signature', tx);
    
    // Fetch the account and validate
    const storageAccount = await program.account.storageAccount.fetch(storageAccountPda);
    expect(storageAccount.authority.toString()).to.equal(authority.publicKey.toString());
    expect(storageAccount.authToken.toString()).to.equal(authToken.toString());
    expect(storageAccount.itemCount.toNumber()).to.equal(0);
    expect(storageAccount.maxCapacity.toNumber()).to.equal(MAX_CAPACITY);
    expect(storageAccount.currentUsage.toNumber()).to.equal(0);
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
        storageAccount: storageAccountPda,
      })
      .rpc();
    
    console.log('Register library transaction signature', registerTx);
    
    // Fetch the account and validate
    const storageAccount = await program.account.storageAccount.fetch(storageAccountPda);
    expect(storageAccount.approvedLibraries.some(lib => lib.toString() === testLibrary.toString())).to.be.true;
  });

  it('Set and Get Storage Item', async () => {
    const testKey = 'test-key';
    const testValue = Buffer.from('test-value');
    
    // Find PDA for the storage item
    const [storageItemPda] = await PublicKey.findProgramAddressSync(
      [
        Buffer.from('storage_item'),
        storageAccountPda.toBuffer(),
        Buffer.from(testKey),
      ],
      program.programId
    );
    
    // Set the storage item
    const setTx = await program.methods
      .setItem({
        key: testKey,
        valueType: { string: {} }, // Using the String value type
        value: testValue,
      })
      .accounts({
        authority: authority.publicKey,
        storageAccount: storageAccountPda,
        storageItem: storageItemPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    
    console.log('Set item transaction signature', setTx);
    
    // Get the storage item
    const getTx = await program.methods
      .getItem(testKey)
      .accounts({
        authority: authority.publicKey,
        storageAccount: storageAccountPda,
        storageItem: storageItemPda,
      })
      .rpc();
    
    console.log('Get item transaction signature', getTx);
    
    // Fetch the item and validate
    const storageItem = await program.account.storageItem.fetch(storageItemPda);
    expect(storageItem.key).to.equal(testKey);
    expect(storageItem.valueType.string).to.not.be.undefined;
    expect(Buffer.from(storageItem.value)).to.deep.equal(testValue);
    expect(storageItem.version.toNumber()).to.equal(1);
    
    // Fetch the storage account and validate usage
    const storageAccount = await program.account.storageAccount.fetch(storageAccountPda);
    expect(storageAccount.itemCount.toNumber()).to.equal(1);
    expect(storageAccount.currentUsage.toNumber()).to.equal(testValue.length);
  });

  it('Delete Storage Item', async () => {
    const testKey = 'test-key';
    
    // Find PDA for the storage item
    const [storageItemPda] = await PublicKey.findProgramAddressSync(
      [
        Buffer.from('storage_item'),
        storageAccountPda.toBuffer(),
        Buffer.from(testKey),
      ],
      program.programId
    );
    
    // Delete the storage item
    const deleteTx = await program.methods
      .deleteItem(testKey)
      .accounts({
        authority: authority.publicKey,
        storageAccount: storageAccountPda,
        storageItem: storageItemPda,
      })
      .rpc();
    
    console.log('Delete item transaction signature', deleteTx);
    
    // Verify item is deleted (should throw an error when trying to fetch)
    try {
      await program.account.storageItem.fetch(storageItemPda);
      expect.fail('Item should have been deleted');
    } catch (error) {
      // Expected error
    }
    
    // Fetch the storage account and validate usage
    const storageAccount = await program.account.storageAccount.fetch(storageAccountPda);
    expect(storageAccount.itemCount.toNumber()).to.equal(0);
    expect(storageAccount.currentUsage.toNumber()).to.equal(0);
  });

  // Batch operations and passthroughs from Base Account Program tests would go here
}); 