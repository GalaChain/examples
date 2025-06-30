import { asValidUserRef } from "@gala-chain/api";
import { sha256 } from "@noble/hashes/sha2";
import { bytesToHex, utf8ToBytes } from "@noble/hashes/utils";
import BigNumber from "bignumber.js";
import { plainToInstance } from "class-transformer";

import { ISubmissionCommitment, SubmissionCommitment } from "./SubmissionCommitment";

describe("SubmissionCommitment chain entry", () => {
  const testHash = bytesToHex(sha256(utf8ToBytes("test")));

  let validEntry: SubmissionCommitment;

  const validProperties: ISubmissionCommitment = {
    collection: "test collection",
    owner: asValidUserRef("client|abc"),
    hash: testHash,
    nonce: "0"
  };

  it("should construct a valid SubmissionCommitment with constructor", async () => {
    const entry = new SubmissionCommitment(validProperties);

    const validation = await entry.validate();

    expect(validation).toEqual([]);
    validEntry = entry;
  });

  it("should construct a valid SubmissionCommitment with plainToInstance", async () => {
    const entry = plainToInstance(SubmissionCommitment, validProperties);

    const validation = await entry.validate();

    expect(validation).toEqual([]);
  });

  it("should not include commitment properties if included with constructor args", async () => {
    const senstiveProperties = {
      ...validProperties,
      item: "test item",
      bid: new BigNumber("100"),
      commitmentNonce: "0",
      commitmentHash: testHash,
      salt: "test salt"
    };

    // When
    const constructedEntry = new SubmissionCommitment(senstiveProperties);

    // Then
    expect(constructedEntry).toEqual(validEntry);

    expect(senstiveProperties.item).toBe("test item");
    expect(senstiveProperties.bid).toEqual(new BigNumber("100"));
    expect(senstiveProperties.commitmentNonce).toBe("0");
    expect(senstiveProperties.commitmentHash).toBe(testHash);
    expect(senstiveProperties.salt).toBe("test salt");
  });
});
