// @ts-nocheck
import { expect } from "chai";
import * as fs from "fs";

type AnchorIdl = {
  instructions: Array<{
    name: string;
    accounts: Array<{
      name: string;
      writable?: boolean;
      signer?: boolean;
      pda?: {
        seeds: Array<
          | { kind: "const"; value: number[] }
          | { kind: "account"; path: string }
        >;
      };
    }>;
  }>;
};

function constSeedToUtf8(seed: { kind: "const"; value: number[] }) {
  return Buffer.from(seed.value).toString("utf8");
}

function findInstruction(idl: AnchorIdl, name: string) {
  const instr = idl.instructions.find((x) => x.name === name);
  if (!instr) throw new Error(`Missing instruction: ${name}`);
  return instr;
}

function findPdaSeeds(
  instr: ReturnType<typeof findInstruction>,
  accountName: string
) {
  const acct = instr.accounts.find((a) => a.name === accountName);
  expect(acct, `Account ${accountName} missing`).to.not.equal(undefined);
  if (!acct?.pda) throw new Error(`Account ${accountName} has no PDA in IDL`);
  return acct.pda.seeds;
}

describe("Crowdfunding IDL structure (security-critical checks)", () => {
  it("has all 4 required instructions", () => {
    const idl: AnchorIdl = JSON.parse(
      fs.readFileSync("target/idl/crowdfunding.json", "utf8")
    );

    const names = idl.instructions.map((i) => i.name);
    expect(names).to.include("create_campaign");
    expect(names).to.include("contribute");
    expect(names).to.include("withdraw");
    expect(names).to.include("refund");
  });

  it("vault PDA uses seeds: ['vault', campaign]", () => {
    const idl: AnchorIdl = JSON.parse(
      fs.readFileSync("target/idl/crowdfunding.json", "utf8")
    );
    const create = findInstruction(idl, "create_campaign");
    const vaultSeeds = findPdaSeeds(create, "vault");

    const [s0, s1] = vaultSeeds;
    expect("kind" in s0 && s0.kind === "const").to.equal(true);
    expect(constSeedToUtf8(s0 as any)).to.equal("vault");

    expect("kind" in s1 && s1.kind === "account").to.equal(true);
    expect((s1 as any).path).to.equal("campaign");
  });

  it("refund requires per-user contribution PDA (refund safety)", () => {
    const idl: AnchorIdl = JSON.parse(
      fs.readFileSync("target/idl/crowdfunding.json", "utf8")
    );
    const refund = findInstruction(idl, "refund");

    const contribAcct = refund.accounts.find((a) => a.name === "contribution");
    expect(
      contribAcct,
      "refund must include contribution account"
    ).to.not.equal(undefined);
    expect(contribAcct?.writable, "contribution must be writable").to.equal(
      true
    );
    expect(contribAcct?.pda, "contribution must be a PDA").to.not.equal(
      undefined
    );

    const seeds = findPdaSeeds(refund, "contribution");
    // expected: ["contribution", campaign, contributor]
    const constSeed = seeds[0];
    const campaignSeed = seeds[1];
    const contributorSeed = seeds[2];

    expect(constSeed.kind).to.equal("const");
    expect(constSeedToUtf8(constSeed as any)).to.equal("contribution");

    expect(campaignSeed.kind).to.equal("account");
    expect((campaignSeed as any).path).to.equal("campaign");

    expect(contributorSeed.kind).to.equal("account");
    expect((contributorSeed as any).path).to.equal("contributor");
  });
});

