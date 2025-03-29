import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { expect } from "chai";

describe("Registry Program Tests", () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // Get the program ID from the workspace
  const registry = anchor.workspace.Registry;
  
  // Generate placeholder for authorization program and account factory
  const authorizationProgramId = Keypair.generate().publicKey;
  const accountFactoryProgramId = Keypair.generate().publicKey;
  
  // Generate a test library program ID
  const libraryProgramId = Keypair.generate().publicKey;
  
  // Store state for registry tests
  let registryStatePda: PublicKey;
  let libraryInfoPda: PublicKey;
  
  // Helper function to derive PDAs
  async function findRegistryStatePda() {
    const [pda, bump] = await PublicKey.findProgramAddress(
      [Buffer.from("registry_state")],
      registry.programId
    );
    return { pda, bump };
  }
  
  async function findLibraryInfoPda(programId: PublicKey) {
    const [pda, bump] = await PublicKey.findProgramAddress(
      [Buffer.from("library_info"), programId.toBuffer()],
      registry.programId
    );
    return { pda, bump };
  }
  
  before(async () => {
    // Derive the registry state PDA
    const { pda } = await findRegistryStatePda();
    registryStatePda = pda;
    
    // Derive the library info PDA
    const libraryPda = await findLibraryInfoPda(libraryProgramId);
    libraryInfoPda = libraryPda.pda;
  });
  
  it("Initializes the registry program", async () => {
    // Initialize the program
    await registry.methods
      .initialize(authorizationProgramId, accountFactoryProgramId)
      .accounts({
        registryState: registryStatePda,
        owner: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
      
    // Fetch the state
    const state = await registry.account.registryState.fetch(
      registryStatePda
    );
    
    // Verify the state
    expect(state.owner.toString()).to.equal(provider.wallet.publicKey.toString());
    expect(state.authorizationProgramId.toString()).to.equal(authorizationProgramId.toString());
    expect(state.accountFactory.toString()).to.equal(accountFactoryProgramId.toString());
  });
  
  it("Registers a library", async () => {
    // Library details
    const libraryType = "token_transfer";
    const description = "A library for transferring tokens between accounts";
    const isApproved = true;
    
    // Register the library
    await registry.methods
      .registerLibrary(libraryType, description, isApproved)
      .accounts({
        registryState: registryStatePda,
        libraryInfo: libraryInfoPda,
        programId: libraryProgramId,
        owner: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
      
    // Fetch the library info
    const libraryInfo = await registry.account.libraryInfo.fetch(
      libraryInfoPda
    );
    
    // Verify the library info
    expect(libraryInfo.programId.toString()).to.equal(libraryProgramId.toString());
    expect(libraryInfo.libraryType).to.equal(libraryType);
    expect(libraryInfo.description).to.equal(description);
    expect(libraryInfo.isApproved).to.equal(isApproved);
    expect(libraryInfo.version).to.equal("1.0.0");
  });
  
  it("Updates a library's status", async () => {
    // Update the library status to not approved
    await registry.methods
      .updateLibraryStatus(false)
      .accounts({
        registryState: registryStatePda,
        libraryInfo: libraryInfoPda,
        owner: provider.wallet.publicKey,
      })
      .rpc();
      
    // Fetch the library info
    const libraryInfo = await registry.account.libraryInfo.fetch(
      libraryInfoPda
    );
    
    // Verify the status was updated
    expect(libraryInfo.isApproved).to.be.false;
  });
  
  it("Queries a library's information", async () => {
    // Query the library
    const libraryInfo = await registry.methods
      .queryLibrary()
      .accounts({
        registryState: registryStatePda,
        libraryInfo: libraryInfoPda,
        programId: libraryProgramId,
      })
      .view();
      
    // Verify the returned info
    expect(libraryInfo.programId.toString()).to.equal(libraryProgramId.toString());
    expect(libraryInfo.libraryType).to.equal("token_transfer");
    expect(libraryInfo.description).to.equal("A library for transferring tokens between accounts");
    expect(libraryInfo.isApproved).to.be.false;
    expect(libraryInfo.version).to.equal("1.0.0");
  });
}); 