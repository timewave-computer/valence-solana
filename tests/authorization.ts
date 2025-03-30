import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { expect } from "chai";

describe("Authorization Program Tests", () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // Get the program ID from the workspace
  const authorization = anchor.workspace.Authorization;
  
  // Generate placeholder for processor and registry programs
  const processorProgramId = Keypair.generate().publicKey;
  const registryProgramId = Keypair.generate().publicKey;
  
  // Store state for authorization tests
  let authorizationStatePda: PublicKey;
  let authorizationPda: PublicKey;
  let authorizationBump: number;
  const authLabel = "test_auth";
  
  // Helper function to derive PDAs
  async function findAuthorizationStatePda() {
    const [pda, bump] = await PublicKey.findProgramAddress(
      [Buffer.from("authorization_state")],
      authorization.programId
    );
    return { pda, bump };
  }
  
  async function findAuthorizationPda(label: string) {
    const [pda, bump] = await PublicKey.findProgramAddress(
      [Buffer.from("authorization"), Buffer.from(label)],
      authorization.programId
    );
    return { pda, bump };
  }
  
  before(async () => {
    // Derive the authorization state PDA
    const { pda } = await findAuthorizationStatePda();
    authorizationStatePda = pda;
    
    // Derive the authorization PDA
    const authPda = await findAuthorizationPda(authLabel);
    authorizationPda = authPda.pda;
    authorizationBump = authPda.bump;
  });
  
  it("Initializes the authorization program", async () => {
    // Initialize the program
    await authorization.methods
      .initialize(registryProgramId, processorProgramId)
      .accounts({
        authorizationState: authorizationStatePda,
        owner: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
      
    // Fetch the state
    const state = await authorization.account.authorizationState.fetch(
      authorizationStatePda
    );
    
    // Verify the state
    expect(state.owner.toString()).to.equal(provider.wallet.publicKey.toString());
    expect(state.processorProgramId.toString()).to.equal(processorProgramId.toString());
    expect(state.valenceRegistry.toString()).to.equal(registryProgramId.toString());
    expect(state.executionCounter.toString()).to.equal("0");
  });
  
  it("Creates an authorization", async () => {
    // Create an authorization
    const now = Math.floor(Date.now() / 1000);
    
    await authorization.methods
      .createAuthorization(
        authLabel,
        { public: {} },
        null,
        now,
        null,
        10,
        { medium: {} },
        { atomic: {} }
      )
      .accounts({
        authorizationState: authorizationStatePda,
        authorization: authorizationPda,
        owner: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
      
    // Fetch the authorization
    const auth = await authorization.account.authorization.fetch(
      authorizationPda
    );
    
    // Verify the authorization
    expect(auth.label).to.equal(authLabel);
    expect(auth.owner.toString()).to.equal(provider.wallet.publicKey.toString());
    expect(auth.isActive).to.be.true;
    expect(auth.maxConcurrentExecutions).to.equal(10);
  });
  
  it("Looks up an authorization (tests cache)", async () => {
    // Look up the authorization by label
    const lookupResult = await authorization.methods
      .lookupAuthorization(authLabel)
      .accounts({
        authorizationState: authorizationStatePda,
        authorization: authorizationPda,
      })
      .view();
      
    // Verify the result
    expect(lookupResult.toString()).to.equal(authorizationPda.toString());
    
    // Look up again (should use cache)
    const cachedResult = await authorization.methods
      .lookupAuthorization(authLabel)
      .accounts({
        authorizationState: authorizationStatePda,
        authorization: null, // Don't provide the authorization, forcing cache use
      })
      .view();
      
    // Verify the cached result
    expect(cachedResult.toString()).to.equal(authorizationPda.toString());
  });
  
  it("Disables an authorization", async () => {
    // Disable the authorization
    await authorization.methods
      .disableAuthorization()
      .accounts({
        authorizationState: authorizationStatePda,
        authorization: authorizationPda,
        owner: provider.wallet.publicKey,
      })
      .rpc();
      
    // Fetch the authorization
    const auth = await authorization.account.authorization.fetch(
      authorizationPda
    );
    
    // Verify it's disabled
    expect(auth.isActive).to.be.false;
  });
  
  it("Enables an authorization", async () => {
    // Enable the authorization
    await authorization.methods
      .enableAuthorization()
      .accounts({
        authorizationState: authorizationStatePda,
        authorization: authorizationPda,
        owner: provider.wallet.publicKey,
      })
      .rpc();
      
    // Fetch the authorization
    const auth = await authorization.account.authorization.fetch(
      authorizationPda
    );
    
    // Verify it's enabled
    expect(auth.isActive).to.be.true;
  });
  
  it("Modifies an authorization", async () => {
    // Modify the authorization
    await authorization.methods
      .modifyAuthorization(
        { ownerOnly: {} },
        null,
        null,
        null,
        5,
        { high: {} },
        null
      )
      .accounts({
        authorizationState: authorizationStatePda,
        authorization: authorizationPda,
        owner: provider.wallet.publicKey,
      })
      .rpc();
      
    // Fetch the authorization
    const auth = await authorization.account.authorization.fetch(
      authorizationPda
    );
    
    // Verify the changes
    expect(auth.permissionType).to.deep.include.keys("ownerOnly");
    expect(auth.maxConcurrentExecutions).to.equal(5);
    expect(auth.priority).to.deep.include.keys("high");
  });
}); 