import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { SolchStakingContract } from "../target/types/solch_staking_contract";

describe("solch-staking-contract", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.SolchStakingContract as Program<SolchStakingContract>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.rpc.initialize({});
    console.log("Your transaction signature", tx);
  });
});
