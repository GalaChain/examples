import {
  ChainUser,
  GalaChainResponse,
  asValidUserRef,
  createValidDTO,
  randomUniqueKey
} from "@gala-chain/api";
import { GalaChainContext } from "@gala-chain/chaincode";
import { fixture, randomUser, writesMap } from "@gala-chain/test";
import { sha256 } from "@noble/hashes/sha2";
import { bytesToHex, utf8ToBytes } from "@noble/hashes/utils";
import BigNumber from "bignumber.js";
import { plainToInstance } from "class-transformer";

import { ProductContract } from "./ProductContract";
import { CommitSubmissionDto, CommitSubmissionResDto, SubmissionCommitment } from "./api";

describe("commitSubmission chaincode call", () => {
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

  test("commitSubmission", async () => {
    const dto = new CommitSubmissionDto({
      collection,
      hash,
      uniqueKey: nonce
    }).signed(user1.privateKey);

    const expectedResponse = await createValidDTO(CommitSubmissionResDto, { collection, owner, hash, nonce });

    const { ctx, contract } = fixture<GalaChainContext, ProductContract>(ProductContract).registeredUsers(
      user1
    );

    const response = await contract.CommitSubmission(ctx, dto);

    expect(response).toEqual(GalaChainResponse.Success(expectedResponse));
  });
});
