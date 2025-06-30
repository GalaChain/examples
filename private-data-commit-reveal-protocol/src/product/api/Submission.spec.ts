import {
  ChainUser,
  FeeVerificationDto,
  asValidUserRef,
  createValidDTO,
  randomUniqueKey
} from "@gala-chain/api";
import { sha256 } from "@noble/hashes/sha2";
import { bytesToHex, utf8ToBytes } from "@noble/hashes/utils";
import BigNumber from "bignumber.js";
import { plainToInstance } from "class-transformer";

import { Submission } from "./Submission";

describe("Submission chain entry", () => {
  const testHash = bytesToHex(sha256(utf8ToBytes("test")));

  let validEntry: Submission;

  const validCommitment = "test item/100/0/test salt";
  const validHash = "f43c3cb6c01662028dce48ec947b35ef9fd68349d06255b5b3c38f586ca0fbb4";

  it("should construct a Submission with constructor", async () => {
    // Given
    const entry = new Submission({
      collection: "test collection",
      item: "test item",
      owner: asValidUserRef("client|abc"),
      bid: new BigNumber("100"),
      commitmentNonce: "0",
      commitmentHash: testHash,
      salt: "test salt"
    });

    // When
    const validation = await entry.validate();

    // Then
    expect(validation).toEqual([]);
    validEntry = entry;
  });

  it("should construct a Submission with plainToInstance", async () => {
    // Given
    const entry = plainToInstance(Submission, {
      collection: "test collection",
      item: "test item",
      owner: asValidUserRef("client|abc"),
      bid: new BigNumber("100"),
      commitmentNonce: "0",
      commitmentHash: testHash,
      salt: "test salt"
    });

    // When
    const validation = await entry.validate();

    // Then
    expect(validation).toEqual([]);
    expect(entry).toEqual(validEntry);
  });

  it("should concatenate fields to represent the commitment as a single string", () => {
    // Given
    const expectedCommitment = validCommitment;

    // When
    const commitment = validEntry.concatenateCommitment();

    // Then
    expect(commitment).toBe(expectedCommitment);
  });

  it("should generate a hash based on the concatenated commitment properties", () => {
    // Given
    const expectedHash = validHash;

    // When
    const hash = validEntry.generateHash();

    // Then
    expect(hash).toEqual(expectedHash);
  });

  it("should verify a valid hash based on its defined commitment properties", () => {
    // Given
    const expectedResult = true;

    // When
    const result = validEntry.verifyHash(validHash);

    // Then
    expect(result).toBe(expectedResult);
  });

  test("valid commitment and hash pair in test setup", () => {
    // Given
    const calculatedHash = bytesToHex(sha256(utf8ToBytes(validCommitment)));

    // When
    const result = calculatedHash === validHash;

    // Then
    expect(result).toBe(true);
  });
});
