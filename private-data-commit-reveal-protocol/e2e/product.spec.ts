import {
  BatchDto,
  ChainClient,
  ChainUser,
  CommonContractAPI,
  GalaChainResponse,
  UserRef,
  asValidUserRef,
  commonContractAPI,
  createValidDTO,
  randomUniqueKey
} from "@gala-chain/api";
import { AdminChainClients, TestClients, randomize, transactionSuccess } from "@gala-chain/test";
import { sha256 } from "@noble/hashes/sha2";
import { bytesToHex, utf8ToBytes } from "@noble/hashes/utils";
import BigNumber from "bignumber.js";
import { plainToInstance } from "class-transformer";

import {
  CommitSubmissionDto,
  CommitSubmissionResDto,
  RevealSubmissionDto,
  RevealSubmissionResDto,
  Submission,
  SubmissionCommitment,
  WithdrawSubmissionDto,
  WithdrawSubmissionResDto
} from "../src/product";

describe("Product contract", () => {
  const productContractConfig = {
    product: {
      channel: "product-channel",
      chaincode: "basic-product",
      contract: "Product",
      api: productContractAPI
    }
  };

  let client: AdminChainClients<typeof productContractConfig>;
  let user: ChainUser;
  let collection: string, item: string, nonce: string, commitmentNonce: string, bid: BigNumber, salt: string;
  let validCommitment: string;
  let hash: string;
  const expectedHash = "";
  let owner: UserRef;

  let commitment1: SubmissionCommitment;
  let submission1: Submission;

  beforeAll(async () => {
    client = await TestClients.createForAdmin(productContractConfig);
    user = await client.createRegisteredUser();

    owner = asValidUserRef(user.identityKey);
    collection = randomize("test collection");
    item = randomize("test item");
    nonce = randomUniqueKey();
    commitmentNonce = nonce;
    bid = new BigNumber("100");
    salt = randomize("random salt");

    const _ = SubmissionCommitment.SEPARATOR;
    const commitment = `${item}${_}${bid.toString()}${_}${commitmentNonce}${_}${salt}`;
    hash = bytesToHex(sha256(utf8ToBytes(commitment)));

    submission1 = new Submission({
      collection,
      item,
      owner,
      bid,
      commitmentNonce: nonce,
      commitmentHash: hash,
      salt
    });
  });

  afterAll(async () => {
    await client.disconnect();
  });

  test("CommitSubmission", async () => {
    const dto = await createValidDTO(CommitSubmissionDto, {
      collection,
      hash,
      uniqueKey: nonce
    }).signed(user.privateKey);

    const expectedResponse = await createValidDTO(CommitSubmissionResDto, {
      collection,
      owner,
      hash,
      nonce
    });

    const response = await client.product.CommitSubmission(dto);

    expect(response).toEqual(transactionSuccess(expectedResponse));
  });

  test("RevealSubmission", async () => {
    const dto = await createValidDTO(RevealSubmissionDto, {
      collection,
      item,
      commitmentNonce,
      commitmentHash: hash,
      salt,
      bid,
      uniqueKey: randomUniqueKey()
    }).signed(user.privateKey);

    const expectedResponse = plainToInstance(RevealSubmissionResDto, {
      collection,
      item,
      owner,
      commitmentNonce: nonce,
      commitmentHash: hash,
      salt,
      bid,
      uniqueKey: ""
    });

    const response = await client.product.RevealSubmission(dto);

    expect(response).toEqual(transactionSuccess(expectedResponse));
  });
});

interface IProductContractAPI {
  CommitSubmission(dto: CommitSubmissionDto): Promise<GalaChainResponse<CommitSubmissionResDto>>;
  WithdrawSubmission(dto: WithdrawSubmissionDto): Promise<GalaChainResponse<WithdrawSubmissionResDto>>;
  RevealSubmission(dto: RevealSubmissionDto): Promise<GalaChainResponse<RevealSubmissionResDto>>;
}

function productContractAPI(client: ChainClient): IProductContractAPI & CommonContractAPI {
  return {
    ...commonContractAPI(client),
    CommitSubmission(dto: CommitSubmissionDto) {
      return client.submitTransaction("CommitSubmission", dto, CommitSubmissionResDto) as Promise<
        GalaChainResponse<CommitSubmissionResDto>
      >;
    },
    RevealSubmission(dto: RevealSubmissionDto) {
      return client.submitTransaction("RevealSubmission", dto, RevealSubmissionDto) as Promise<
        GalaChainResponse<RevealSubmissionResDto>
      >;
    },
    WithdrawSubmission(dto: WithdrawSubmissionDto) {
      return client.submitTransaction("WithdrawSubmission", dto, WithdrawSubmissionResDto) as Promise<
        GalaChainResponse<WithdrawSubmissionResDto>
      >;
    }
  };
}
