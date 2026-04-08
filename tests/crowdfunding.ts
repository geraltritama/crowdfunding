// @ts-nocheck
import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
} from "@solana/web3.js";
import { expect } from "chai";
import { Crowdfunding } from "../target/types/crowdfunding";

/**
 * Runtime (localnet) integration tests.
 *
 * Run with:
 * - `anchor test`  (recommended; starts a local validator and sets env vars)
 */

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

const getVaultPda = (programId: PublicKey, campaign: PublicKey) =>
  PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), campaign.toBuffer()],
    programId
  )[0];

const getContributionPda = (
  programId: PublicKey,
  campaign: PublicKey,
  contributor: PublicKey
) =>
  PublicKey.findProgramAddressSync(
    [
      Buffer.from("contribution"),
      campaign.toBuffer(),
      contributor.toBuffer(),
    ],
    programId
  )[0];

describe("crowdfunding runtime flow (localnet)", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Crowdfunding as Program<Crowdfunding>;
  const connection = provider.connection;

  const airdrop = async (pk: PublicKey, sol = 10) => {
    const sig = await connection.requestAirdrop(pk, sol * LAMPORTS_PER_SOL);
    await connection.confirmTransaction(sig, "confirmed");
  };

  it("success path: contribute -> withdraw before deadline fails -> withdraw after deadline succeeds -> double withdraw fails", async function () {
    this.timeout(120000);

    const creator = Keypair.generate();
    const contributor = Keypair.generate();
    const campaign = Keypair.generate();

    await airdrop(creator.publicKey, 10);
    await airdrop(contributor.publicKey, 10);

    const vault = getVaultPda(program.programId, campaign.publicKey);
    const contribution = getContributionPda(
      program.programId,
      campaign.publicKey,
      contributor.publicKey
    );

    const goal = new BN(1 * LAMPORTS_PER_SOL);
    const deadline = new BN(Math.floor(Date.now() / 1000) + 2);

    const createSig = await program.methods
      .createCampaign(goal, deadline)
      .accounts({
        creator: creator.publicKey,
        campaign: campaign.publicKey,
        vault,
        systemProgram: SystemProgram.programId,
      })
      .signers([creator, campaign])
      .rpc();

    const contributeSig = await program.methods
      .contribute(new BN(2 * LAMPORTS_PER_SOL))
      .accounts({
        contributor: contributor.publicKey,
        campaign: campaign.publicKey,
        vault,
        contribution,
        systemProgram: SystemProgram.programId,
      })
      .signers([contributor])
      .rpc();

    // withdraw BEFORE deadline should fail
    try {
      await program.methods
        .withdraw()
        .accounts({
          creator: creator.publicKey,
          campaign: campaign.publicKey,
          vault,
          systemProgram: SystemProgram.programId,
        })
        .signers([creator])
        .rpc();
      expect.fail("withdraw before deadline should throw");
    } catch (err: any) {
      expect(err.message).to.match(/tooEarly|deadline/i);
    }

    await sleep(2500);

    const withdrawSig = await program.methods
      .withdraw()
      .accounts({
        creator: creator.publicKey,
        campaign: campaign.publicKey,
        vault,
        systemProgram: SystemProgram.programId,
      })
      .signers([creator])
      .rpc();

    // double withdraw should fail
    try {
      await program.methods
        .withdraw()
        .accounts({
          creator: creator.publicKey,
          campaign: campaign.publicKey,
          vault,
          systemProgram: SystemProgram.programId,
        })
        .signers([creator])
        .rpc();
      expect.fail("double withdraw should throw");
    } catch (err: any) {
      expect(err.message).to.match(/alreadyClaimed/i);
    }

    console.log(
      JSON.stringify(
        { createSig, contributeSig, withdrawSig },
        null,
        2
      )
    );
  });

  it("refund path: contribute -> refund after deadline succeeds -> double refund fails", async function () {
    this.timeout(120000);

    const creator = Keypair.generate();
    const contributor = Keypair.generate();
    const campaign = Keypair.generate();

    await airdrop(creator.publicKey, 10);
    await airdrop(contributor.publicKey, 10);

    const vault = getVaultPda(program.programId, campaign.publicKey);
    const contribution = getContributionPda(
      program.programId,
      campaign.publicKey,
      contributor.publicKey
    );

    const goal = new BN(100 * LAMPORTS_PER_SOL); // won't be met
    const deadline = new BN(Math.floor(Date.now() / 1000) + 2);

    const createSig = await program.methods
      .createCampaign(goal, deadline)
      .accounts({
        creator: creator.publicKey,
        campaign: campaign.publicKey,
        vault,
        systemProgram: SystemProgram.programId,
      })
      .signers([creator, campaign])
      .rpc();

    const contributeSig = await program.methods
      .contribute(new BN(1 * LAMPORTS_PER_SOL))
      .accounts({
        contributor: contributor.publicKey,
        campaign: campaign.publicKey,
        vault,
        contribution,
        systemProgram: SystemProgram.programId,
      })
      .signers([contributor])
      .rpc();

    await sleep(2500);

    const refundSig = await program.methods
      .refund()
      .accounts({
        contributor: contributor.publicKey,
        campaign: campaign.publicKey,
        vault,
        contribution,
        systemProgram: SystemProgram.programId,
      })
      .signers([contributor])
      .rpc();

    // double refund should fail
    try {
      await program.methods
        .refund()
        .accounts({
          contributor: contributor.publicKey,
          campaign: campaign.publicKey,
          vault,
          contribution,
          systemProgram: SystemProgram.programId,
        })
        .signers([contributor])
        .rpc();
      expect.fail("double refund should throw");
    } catch (err: any) {
      // Depending on whether the Contribution account was closed, Anchor may throw an
      // account-related error before reaching the program error code.
      expect(err.message).to.match(/nothingToRefund|invalid|claimed|contribut|account/i);
    }

    console.log(
      JSON.stringify(
        { createSig, contributeSig, refundSig },
        null,
        2
      )
    );
  });
});

