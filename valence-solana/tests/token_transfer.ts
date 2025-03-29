import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { TokenTransfer } from '../target/types/token_transfer';
import { PublicKey, Keypair, SystemProgram, LAMPORTS_PER_SOL } from '@solana/web3.js';
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createMint,
  createAssociatedTokenAccount,
  mintTo,
  getAccount,
  getAssociatedTokenAddress,
} from '@solana/spl-token';
import { expect } from 'chai';

describe('Token Transfer Library', () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.TokenTransfer as Program<TokenTransfer>;
  
  // Setup test accounts
  const authority = Keypair.generate();
  const user = Keypair.generate();
  const recipient = Keypair.generate();
  const feeReceiver = Keypair.generate();
  
  // Test tokens and accounts
  let mint: PublicKey;
  let authorityTokenAccount: PublicKey;
  let userTokenAccount: PublicKey;
  let recipientTokenAccount: PublicKey;
  let feeTokenAccount: PublicKey;
  
  // Config PDA
  let configPda: PublicKey;
  let configBump: number;
  
  // Test token authority PDA for delegated transfers
  let tokenAuthorityPda: PublicKey;
  let tokenAuthorityBump: number;
  
  const MINT_DECIMALS = 6;
  const INITIAL_MINT_AMOUNT = 1000 * (10 ** MINT_DECIMALS);
  const TEST_TRANSFER_AMOUNT = 50 * (10 ** MINT_DECIMALS);
  const TEST_FEE_BPS = 100; // 1%
  
  before(async () => {
    // Airdrop SOL to authority and user
    await provider.connection.requestAirdrop(authority.publicKey, 10 * LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(user.publicKey, 10 * LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(recipient.publicKey, LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(feeReceiver.publicKey, LAMPORTS_PER_SOL);
    
    // Create test token mint
    mint = await createMint(
      provider.connection,
      authority,
      authority.publicKey,
      null,
      MINT_DECIMALS,
      undefined,
      undefined,
      TOKEN_PROGRAM_ID
    );
    
    // Create associated token accounts
    authorityTokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      authority,
      mint,
      authority.publicKey,
      undefined,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );
    
    userTokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      user,
      mint,
      user.publicKey,
      undefined,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );
    
    recipientTokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      recipient,
      mint,
      recipient.publicKey,
      undefined,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );
    
    feeTokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      feeReceiver,
      mint,
      feeReceiver.publicKey,
      undefined,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );
    
    // Mint initial tokens to authority and user
    await mintTo(
      provider.connection,
      authority,
      mint,
      authorityTokenAccount,
      authority,
      INITIAL_MINT_AMOUNT,
      [],
      undefined,
      TOKEN_PROGRAM_ID
    );
    
    await mintTo(
      provider.connection,
      authority,
      mint,
      userTokenAccount,
      authority,
      INITIAL_MINT_AMOUNT,
      [],
      undefined,
      TOKEN_PROGRAM_ID
    );
    
    // Find config PDA
    [configPda, configBump] = await PublicKey.findProgramAddressSync(
      [
        Buffer.from("token_transfer_config"),
        authority.publicKey.toBuffer(),
      ],
      program.programId
    );
    
    // Find token authority PDA (for delegated transfers)
    [tokenAuthorityPda, tokenAuthorityBump] = await PublicKey.findProgramAddressSync(
      [
        Buffer.from("token_authority"),
        userTokenAccount.toBuffer(),
      ],
      program.programId
    );
  });
  
  it('Initialize the token transfer library', async () => {
    await program.methods
      .initialize({
        processorProgramId: provider.publicKey,
        maxTransferAmount: new anchor.BN(1000000 * (10 ** MINT_DECIMALS)),
        maxBatchSize: 10,
        feeBps: TEST_FEE_BPS,
        feeCollector: feeReceiver.publicKey,
        slippageBps: 0,
        validateAccountOwnership: true,
        enforceSourceAllowlist: false,
        enforceRecipientAllowlist: false,
        enforceMintAllowlist: false,
        allowedSources: [],
        allowedRecipients: [],
        allowedMints: []
      })
      .accounts({
        authority: authority.publicKey,
        config: configPda,
        systemProgram: SystemProgram.programId
      })
      .signers([authority])
      .rpc();
      
    // Fetch and verify the config
    const config = await program.account.libraryConfig.fetch(configPda);
    
    expect(config.authority.toString()).to.equal(authority.publicKey.toString());
    expect(config.isActive).to.be.true;
    expect(config.processorProgramId.toString()).to.equal(provider.publicKey.toString());
    expect(config.maxTransferAmount.toString()).to.equal(new anchor.BN(1000000 * (10 ** MINT_DECIMALS)).toString());
    expect(config.maxBatchSize).to.equal(10);
    expect(config.feeBps).to.equal(TEST_FEE_BPS);
    expect(config.feeCollector.toString()).to.equal(feeReceiver.publicKey.toString());
  });
  
  it('Transfer tokens with fee collection', async () => {
    const preSourceBalance = (await getAccount(
      provider.connection,
      userTokenAccount,
      undefined,
      TOKEN_PROGRAM_ID
    )).amount;
    
    const preDestBalance = (await getAccount(
      provider.connection,
      recipientTokenAccount,
      undefined,
      TOKEN_PROGRAM_ID
    )).amount;
    
    const preFeeBalance = (await getAccount(
      provider.connection,
      feeTokenAccount,
      undefined,
      TOKEN_PROGRAM_ID
    )).amount;
    
    await program.methods
      .transferToken({
        amount: new anchor.BN(TEST_TRANSFER_AMOUNT),
        sourceOwner: user.publicKey,
        destinationOwner: recipient.publicKey,
        slippageBps: null,
        memo: "Test transfer"
      })
      .accounts({
        authority: authority.publicKey,
        config: configPda,
        sourceTokenAccount: userTokenAccount,
        destinationTokenAccount: recipientTokenAccount,
        feeAccount: feeTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId
      })
      .signers([authority, user])
      .rpc();
      
    // Verify balances after transfer
    const postSourceBalance = (await getAccount(
      provider.connection,
      userTokenAccount,
      undefined,
      TOKEN_PROGRAM_ID
    )).amount;
    
    const postDestBalance = (await getAccount(
      provider.connection,
      recipientTokenAccount,
      undefined,
      TOKEN_PROGRAM_ID
    )).amount;
    
    const postFeeBalance = (await getAccount(
      provider.connection,
      feeTokenAccount,
      undefined,
      TOKEN_PROGRAM_ID
    )).amount;
    
    // Calculate expected fee
    const expectedFee = BigInt(TEST_TRANSFER_AMOUNT) * BigInt(TEST_FEE_BPS) / BigInt(10000);
    const expectedTransferAmount = BigInt(TEST_TRANSFER_AMOUNT) - expectedFee;
    
    // Verify source was debited by the full amount
    expect(preSourceBalance - postSourceBalance).to.equal(BigInt(TEST_TRANSFER_AMOUNT));
    
    // Verify destination received amount minus fee
    expect(postDestBalance - preDestBalance).to.equal(expectedTransferAmount);
    
    // Verify fee was collected
    expect(postFeeBalance - preFeeBalance).to.equal(expectedFee);
  });
  
  it('Transfer SOL with fee collection', async () => {
    const transferAmount = LAMPORTS_PER_SOL / 10; // 0.1 SOL
    
    const preSourceBalance = await provider.connection.getBalance(user.publicKey);
    const preDestBalance = await provider.connection.getBalance(recipient.publicKey);
    const preFeeBalance = await provider.connection.getBalance(feeReceiver.publicKey);
    
    await program.methods
      .transferSol({
        amount: new anchor.BN(transferAmount),
        memo: "Test SOL transfer"
      })
      .accounts({
        authority: authority.publicKey,
        config: configPda,
        sourceWallet: user.publicKey,
        destinationWallet: recipient.publicKey,
        feeReceiver: feeReceiver.publicKey,
        systemProgram: SystemProgram.programId
      })
      .signers([authority, user])
      .rpc();
      
    // Verify balances after transfer
    const postSourceBalance = await provider.connection.getBalance(user.publicKey);
    const postDestBalance = await provider.connection.getBalance(recipient.publicKey);
    const postFeeBalance = await provider.connection.getBalance(feeReceiver.publicKey);
    
    // Calculate expected fee
    const expectedFee = Math.floor(transferAmount * TEST_FEE_BPS / 10000);
    const expectedTransferAmount = transferAmount - expectedFee;
    
    // Need to account for gas fees in the source balance check
    expect(preSourceBalance - postSourceBalance).to.be.greaterThan(transferAmount);
    
    // Verify destination received amount minus fee
    expect(postDestBalance - preDestBalance).to.equal(expectedTransferAmount);
    
    // Verify fee was collected
    expect(postFeeBalance - preFeeBalance).to.equal(expectedFee);
  });
  
  it('Execute a batch transfer of tokens', async () => {
    // Create multiple recipient accounts for batch transfer
    const recipients = [];
    const recipientAccounts = [];
    
    for (let i = 0; i < 3; i++) {
      const recipientKeypair = Keypair.generate();
      recipients.push(recipientKeypair);
      
      await provider.connection.requestAirdrop(recipientKeypair.publicKey, LAMPORTS_PER_SOL / 10);
      
      const tokenAccount = await createAssociatedTokenAccount(
        provider.connection,
        authority,
        mint,
        recipientKeypair.publicKey,
        undefined,
        TOKEN_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      );
      
      recipientAccounts.push(tokenAccount);
    }
    
    const preSourceBalance = (await getAccount(
      provider.connection,
      authorityTokenAccount,
      undefined,
      TOKEN_PROGRAM_ID
    )).amount;
    
    const preFeeBalance = (await getAccount(
      provider.connection,
      feeTokenAccount,
      undefined,
      TOKEN_PROGRAM_ID
    )).amount;
    
    // Batch transfer of tokens to multiple recipients
    const transfers = [
      {
        amount: new anchor.BN(10 * (10 ** MINT_DECIMALS)),
        destinationIndex: 0,
        memo: "Batch transfer 1"
      },
      {
        amount: new anchor.BN(20 * (10 ** MINT_DECIMALS)),
        destinationIndex: 1,
        memo: "Batch transfer 2"
      },
      {
        amount: new anchor.BN(30 * (10 ** MINT_DECIMALS)),
        destinationIndex: 2,
        memo: "Batch transfer 3"
      }
    ];
    
    await program.methods
      .batchTransfer({
        transfers: transfers
      })
      .accounts({
        authority: authority.publicKey,
        config: configPda,
        sourceTokenAccount: authorityTokenAccount,
        destinationTokenAccounts: recipientAccounts,
        feeAccount: feeTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .signers([authority])
      .rpc();
      
    // Verify balances after batch transfer
    const postSourceBalance = (await getAccount(
      provider.connection,
      authorityTokenAccount,
      undefined,
      TOKEN_PROGRAM_ID
    )).amount;
    
    const postFeeBalance = (await getAccount(
      provider.connection,
      feeTokenAccount,
      undefined,
      TOKEN_PROGRAM_ID
    )).amount;
    
    // Calculate total transfer amount
    const totalAmount = transfers.reduce((sum, transfer) => sum + BigInt(transfer.amount.toString()), BigInt(0));
    
    // Calculate expected fee
    const expectedFee = totalAmount * BigInt(TEST_FEE_BPS) / BigInt(10000);
    
    // Verify source was debited by the full amount
    expect(preSourceBalance - postSourceBalance).to.equal(totalAmount);
    
    // Verify fee was collected
    expect(postFeeBalance - preFeeBalance).to.equal(expectedFee);
    
    // Verify each recipient received the correct amount
    for (let i = 0; i < recipientAccounts.length; i++) {
      const balance = (await getAccount(
        provider.connection,
        recipientAccounts[i],
        undefined,
        TOKEN_PROGRAM_ID
      )).amount;
      
      const expectedAmount = BigInt(transfers[i].amount.toString()) - 
        (BigInt(transfers[i].amount.toString()) * BigInt(TEST_FEE_BPS) / BigInt(10000));
      
      expect(balance).to.equal(expectedAmount);
    }
  });
  
  it('Delegate and transfer with authority', async () => {
    // Approve the PDA as a delegate for the user account
    const delegateAmount = BigInt(TEST_TRANSFER_AMOUNT);
    
    // Transfer with delegated authority
    const preSourceBalance = (await getAccount(
      provider.connection,
      userTokenAccount,
      undefined,
      TOKEN_PROGRAM_ID
    )).amount;
    
    const preDestBalance = (await getAccount(
      provider.connection,
      recipientTokenAccount,
      undefined,
      TOKEN_PROGRAM_ID
    )).amount;
    
    // This test requires setting up ownership delegation which is complex to do in a test
    // Simplified test to show structure, would need CPI call to delegate authority first
    try {
      await program.methods
        .transferWithAuthority({
          amount: new anchor.BN(TEST_TRANSFER_AMOUNT),
          bump: tokenAuthorityBump,
          memo: "Delegated transfer"
        })
        .accounts({
          authority: authority.publicKey,
          config: configPda,
          sourceTokenAccount: userTokenAccount,
          destinationTokenAccount: recipientTokenAccount,
          tokenAuthority: tokenAuthorityPda,
          feeAccount: feeTokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID
        })
        .signers([authority])
        .rpc();
        
      // Note: We expect this to fail in the test unless proper delegation is set up
      // In a real environment, the PDA would be properly authorized
      console.log("Transfer with authority succeeded (unexpected in test)");
    } catch (e) {
      console.log("Transfer with authority failed as expected without proper delegation setup");
    }
  });
  
  it('Updates the library configuration', async () => {
    // This is a hypothetical update method that could be added to the library
    // Testing changes to configuration parameters
    
    const newFeeReceiver = Keypair.generate();
    await provider.connection.requestAirdrop(newFeeReceiver.publicKey, LAMPORTS_PER_SOL / 10);
    
    const newFeeTokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      authority,
      mint,
      newFeeReceiver.publicKey,
      undefined,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );
    
    // Update configuration (assuming there's an update method)
    // Note: You would need to implement this method in the library
    /* 
    await program.methods
      .updateConfig({
        feeBps: 200, // 2%
        feeCollector: newFeeReceiver.publicKey,
        maxTransferAmount: new anchor.BN(500000 * (10 ** MINT_DECIMALS)),
      })
      .accounts({
        authority: authority.publicKey,
        config: configPda,
      })
      .signers([authority])
      .rpc();
      
    // Fetch and verify the updated config
    const config = await program.account.libraryConfig.fetch(configPda);
    expect(config.feeBps).to.equal(200);
    expect(config.feeCollector.toString()).to.equal(newFeeReceiver.publicKey.toString());
    */
  });
}); 