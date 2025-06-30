import { GalaChainResponse, asValidUserRef, createValidDTO, randomUniqueKey } from "@gala-chain/api";
import { GalaChainContext } from "@gala-chain/chaincode";
import { fixture, randomUser, writesMap } from "@gala-chain/test";
import { sha256 } from "@noble/hashes/sha2";
import { bytesToHex, utf8ToBytes } from "@noble/hashes/utils";
import BigNumber from "bignumber.js";

import { ProductContract } from "./ProductContract";
import { SubmissionCommitment, WithdrawSubmissionDto, WithdrawSubmissionResDto } from "./api";

describe("withdrawSubmission chaincode call", () => {
  const commitmentParams = {
    collection: "test collection",
    item: "test item",
    nonce: "0",
    bid: new BigNumber("100"),
    salt: "test salt"
  };
  const { collection, nonce } = commitmentParams;
  const validCommitment = "test item/100/0/test salt";
  const hash = bytesToHex(sha256(utf8ToBytes(validCommitment)));
  const user1 = randomUser();
  const owner = asValidUserRef(user1.identityKey);

  test("withdrawSubmission", async () => {
    const dto = new WithdrawSubmissionDto({
      collection,
      owner,
      hash,
      nonce,
      uniqueKey: randomUniqueKey()
    }).signed(user1.privateKey);

    const expectedResponse = await createValidDTO(WithdrawSubmissionResDto, {
      collection,
      owner,
      hash,
      nonce
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

    const response = await contract.WithdrawSubmission(ctx, dto);

    expect(response).toEqual(GalaChainResponse.Success(expectedResponse));
  });
});
