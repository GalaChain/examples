import { GalaChainResponse, asValidUserRef, createValidDTO, randomUniqueKey } from "@gala-chain/api";
import { GalaChainContext } from "@gala-chain/chaincode";
import { fixture, randomUser, writesMap } from "@gala-chain/test";
import { sha256 } from "@noble/hashes/sha2";
import { bytesToHex, utf8ToBytes } from "@noble/hashes/utils";
import BigNumber from "bignumber.js";

import { ProductContract } from "./ProductContract";
import { RevealSubmissionDto, RevealSubmissionResDto, SubmissionCommitment } from "./api";

describe("revealSubmission chaincode call", () => {
  const commitmentParams = {
    collection: "test collection",
    item: "test item",
    nonce: "0",
    bid: new BigNumber("100"),
    salt: "test salt"
  };
  const { collection, item, nonce, bid, salt } = commitmentParams;
  const validCommitment = "test item/100/0/test salt";
  const hash = bytesToHex(sha256(utf8ToBytes(validCommitment)));
  const user1 = randomUser();
  const owner = asValidUserRef(user1.identityKey);

  test("revealSubmission", async () => {
    const dto = new RevealSubmissionDto({
      collection,
      item,
      commitmentHash: hash,
      commitmentNonce: nonce,
      salt,
      bid,
      uniqueKey: randomUniqueKey()
    }).signed(user1.privateKey);

    const expectedResponse = await createValidDTO(RevealSubmissionResDto, {
      collection,
      item,
      owner,
      commitmentHash: hash,
      commitmentNonce: nonce,
      salt,
      bid
    });

    const savedCommitment = new SubmissionCommitment({
      collection,
      owner,
      hash,
      nonce
    });

    const { ctx, contract } = fixture<GalaChainContext, ProductContract>(ProductContract)
      .registeredUsers(user1)
      .savedState(savedCommitment);

    const response = await contract.RevealSubmission(ctx, dto);

    expect(response).toEqual(GalaChainResponse.Success(expectedResponse));
  });
});
