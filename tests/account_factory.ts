import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { Keypair, PublicKey, SystemProgram } from '@solana/web3.js';
import { expect } from 'chai';

describe('Account Factory Program', () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // Load the program
  const program = anchor.workspace.AccountFactory as Program;
  
  // Setup accounts and variables
  const authority = provider.wallet;
  let factoryStatePda: PublicKey;
  let factoryStateBump: number;
  let feeReceiver = Keypair.generate().publicKey;
  let templateId = "test_template";
  let templatePda: PublicKey;
  let templateBump: number;
  
  before(async () => {
    // Find PDAs
    [factoryStatePda, factoryStateBump] = await PublicKey.findProgramAddressSync(
      [
        Buffer.from('factory_state'),
      ],
      program.programId
    );
    
    [templatePda, templateBump] = await PublicKey.findProgramAddressSync(
      [
        Buffer.from('account_template'),
        Buffer.from(templateId),
      ],
      program.programId
    );
  });

  it('Initialize Factory State', async () => {
    // Initialize factory state
    const creationFee = new anchor.BN(5000); // 5000 lamports fee
    
    const tx = await program.methods
      .initialize({
        creationFee: creationFee,
        feeReceiver: feeReceiver,
      })
      .accounts({
        authority: authority.publicKey,
        factoryState: factoryStatePda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    
    console.log('Initialize transaction signature', tx);
    
    // Fetch the factory state and validate
    const factoryState = await program.account.factoryState.fetch(factoryStatePda);
    expect(factoryState.authority.toString()).to.equal(authority.publicKey.toString());
    expect(factoryState.templateCount.toNumber()).to.equal(0);
    expect(factoryState.accountCount.toString()).to.equal('0');
    expect(factoryState.isPaused).to.be.false;
    expect(factoryState.creationFee.toString()).to.equal(creationFee.toString());
    expect(factoryState.feeReceiver.toString()).to.equal(feeReceiver.toString());
  });

  it('Register Template', async () => {
    // Create template params
    const tokenMints = [Keypair.generate().publicKey, Keypair.generate().publicKey];
    const approvedLibraries = [Keypair.generate().publicKey];
    const description = "Test template for Base Accounts";
    
    const tx = await program.methods
      .registerTemplate({
        templateId: templateId,
        accountType: 0, // Base Account
        description: description,
        autoFundSol: true,
        fundAmountSol: new anchor.BN(10000),
        createTokenAccounts: true,
        tokenMints: tokenMints,
        approveLibraries: true,
        approvedLibraries: approvedLibraries,
        requiredDestination: null,
        expirationSeconds: null,
      })
      .accounts({
        authority: authority.publicKey,
        factoryState: factoryStatePda,
        template: templatePda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    
    console.log('Register template transaction signature', tx);
    
    // Fetch the template and validate
    const template = await program.account.accountTemplate.fetch(templatePda);
    expect(template.templateId).to.equal(templateId);
    expect(template.authority.toString()).to.equal(authority.publicKey.toString());
    expect(template.accountType).to.equal(0);
    expect(template.version).to.equal(1);
    expect(template.isActive).to.be.true;
    expect(template.autoFundSol).to.be.true;
    expect(template.fundAmountSol.toString()).to.equal('10000');
    expect(template.createTokenAccounts).to.be.true;
    expect(template.tokenMints.length).to.equal(2);
    expect(template.approveLibraries).to.be.true;
    expect(template.approvedLibraries.length).to.equal(1);
    expect(template.description).to.equal(description);
    expect(template.usageCount.toString()).to.equal('0');
    
    // Also check that factory state was updated
    const factoryState = await program.account.factoryState.fetch(factoryStatePda);
    expect(factoryState.templateCount.toNumber()).to.equal(1);
  });

  it('Update Template', async () => {
    // Update template params
    const newDescription = "Updated test template description";
    
    const tx = await program.methods
      .updateTemplate({
        templateId: templateId,
        description: newDescription,
        isActive: true,
        autoFundSol: false,
        fundAmountSol: new anchor.BN(20000),
        createTokenAccounts: null,
        tokenMintsToAdd: null,
        tokenMintsToRemove: null,
        approveLibraries: null,
        librariesToApprove: null,
        librariesToRevoke: null,
        requiredDestination: null,
        expirationSeconds: null,
      })
      .accounts({
        authority: authority.publicKey,
        factoryState: factoryStatePda,
        template: templatePda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    
    console.log('Update template transaction signature', tx);
    
    // Fetch the template and validate updates
    const template = await program.account.accountTemplate.fetch(templatePda);
    expect(template.description).to.equal(newDescription);
    expect(template.autoFundSol).to.be.false;
    expect(template.fundAmountSol.toString()).to.equal('20000');
    expect(template.version).to.equal(2); // Version should have incremented
  });

  it('Create From Template', async () => {
    const seed = "test_account";
    const authToken = Keypair.generate().publicKey;
    
    // In a real test, we would include the created account and other necessary
    // accounts to fully validate the creation process. This is just a skeleton.
    console.log('This would test account creation from template.');
    
    /*
    const tx = await program.methods
      .createFromTemplate({
        templateId: templateId,
        seed: seed,
        owner: authority.publicKey,
        authToken: authToken,
        overrideRequiredDestination: null,
        overrideExpirationSeconds: null,
      })
      .accounts({
        payer: authority.publicKey,
        factoryState: factoryStatePda,
        template: templatePda,
        feeReceiver: feeReceiver,
        createdAccount: createdAccountPda,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        baseAccountProgram: baseAccountProgramId,
        storageAccountProgram: storageAccountProgramId,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    
    console.log('Create from template transaction signature', tx);
    
    // Fetch and validate factory state updated
    const factoryState = await program.account.factoryState.fetch(factoryStatePda);
    expect(factoryState.accountCount.toString()).to.equal('1');
    
    // Fetch and validate template updated
    const template = await program.account.accountTemplate.fetch(templatePda);
    expect(template.usageCount.toString()).to.equal('1');
    */
  });

  it('Batch Create Accounts', async () => {
    // In a real test, we would create multiple accounts and validate them
    console.log('This would test batch account creation.');
    
    /*
    const authToken1 = Keypair.generate().publicKey;
    const authToken2 = Keypair.generate().publicKey;
    
    const tx = await program.methods
      .batchCreateAccounts({
        templateId: templateId,
        batchParams: [
          {
            seed: "batch_1",
            owner: authority.publicKey,
            authToken: authToken1,
            overrideRequiredDestination: null,
            overrideExpirationSeconds: null,
          },
          {
            seed: "batch_2",
            owner: authority.publicKey,
            authToken: authToken2,
            overrideRequiredDestination: null,
            overrideExpirationSeconds: null,
          }
        ]
      })
      .accounts({
        payer: authority.publicKey,
        factoryState: factoryStatePda,
        template: templatePda,
        feeReceiver: feeReceiver,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        baseAccountProgram: baseAccountProgramId,
        storageAccountProgram: storageAccountProgramId,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    
    console.log('Batch create accounts transaction signature', tx);
    
    // Fetch and validate factory state updated
    const factoryState = await program.account.factoryState.fetch(factoryStatePda);
    expect(factoryState.accountCount.toString()).to.equal('3'); // 1 from previous test + 2 from batch
    
    // Fetch and validate template updated
    const template = await program.account.accountTemplate.fetch(templatePda);
    expect(template.usageCount.toString()).to.equal('3');
    */
  });
  
  // Additional tests would include:
  // - Testing fee collection
  // - Testing authorization controls
  // - Testing template validation (active/inactive)
  // - Testing error cases (invalid parameters, etc.)
}); 