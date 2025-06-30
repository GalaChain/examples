import {
  BigNumberIsPositive,
  BigNumberProperty,
  ChainKey,
  ChainObject,
  IsUserRef,
  UserRef,
  asValidUserRef
} from "@gala-chain/api";
import { sha256 } from "@noble/hashes/sha2";
import { bytesToHex, utf8ToBytes } from "@noble/hashes/utils";
import BigNumber from "bignumber.js";
import { IsNotEmpty, IsString } from "class-validator";

import { SubmissionCommitment } from "./SubmissionCommitment";

export interface ISubmission {
  collection: string;
  item: string;
  owner: UserRef;
  bid: BigNumber;
  commitmentNonce: string;
  commitmentHash: string;
  salt: string;
}

export class Submission extends ChainObject {
  public static INDEX_KEY = "PDCRS"; // Private data commit reveal Submission

  constructor(data?: ISubmission) {
    super();
    this.collection = data?.collection ?? "";
    this.item = data?.item ?? "";
    this.owner = data?.owner ?? asValidUserRef("service|null");
    this.bid = data?.bid ?? new BigNumber("");
    this.commitmentNonce = data?.commitmentNonce ?? "";
    this.commitmentHash = data?.commitmentHash ?? "";
    this.salt = data?.salt ?? "";
  }

  @ChainKey({ position: 0 })
  @IsNotEmpty()
  @IsString()
  public collection: string;

  @ChainKey({ position: 1 })
  @IsNotEmpty()
  @IsString()
  public item: string;

  @ChainKey({ position: 2 })
  @IsUserRef()
  public owner: UserRef;

  @ChainKey({ position: 3 })
  @IsNotEmpty()
  @IsString()
  public commitmentNonce: string;

  @IsNotEmpty()
  @IsString()
  public commitmentHash: string;

  @IsNotEmpty()
  @IsString()
  public salt: string;

  @BigNumberIsPositive()
  @BigNumberProperty()
  public bid: BigNumber;

  public concatenateCommitment() {
    const { item, commitmentNonce, salt, bid } = this;
    const _ = SubmissionCommitment.SEPARATOR;

    const commitment = `${item}${_}${bid.toString()}${_}${commitmentNonce}${_}${salt}`;

    return commitment;
  }

  public generateHash(): string {
    const commitment = this.concatenateCommitment();
    const bytes = utf8ToBytes(commitment);
    const hashedBytes = sha256(bytes);

    const hashHex = bytesToHex(hashedBytes);

    return hashHex;
  }

  public verifyHash(hash: string): boolean {
    const expectedHash = this.generateHash();

    return expectedHash === hash;
  }
}
