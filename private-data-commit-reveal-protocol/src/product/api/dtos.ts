import {
  BigNumberIsPositive,
  BigNumberProperty,
  ChainCallDTO,
  IsUserRef,
  SubmitCallDTO,
  UserRef,
  asValidUserRef
} from "@gala-chain/api";
import BigNumber from "bignumber.js";
import { IsHash, IsNotEmpty, IsString } from "class-validator";
import { JSONSchema } from "class-validator-jsonschema";

export interface ICommitSubmissionDto {
  collection: string;
  hash: string;
  uniqueKey: string;
}

export class CommitSubmissionDto extends SubmitCallDTO {
  @IsNotEmpty()
  @IsString()
  collection: string;

  @IsNotEmpty()
  @IsHash("sha256")
  hash: string;

  constructor(args: unknown) {
    super();
    const data: ICommitSubmissionDto = args as ICommitSubmissionDto;
    this.collection = data?.collection ?? "";
    this.hash = data?.hash ?? "";
    this.uniqueKey = data?.uniqueKey ?? "";
  }
}

export class CommitSubmissionResDto extends ChainCallDTO {
  @IsNotEmpty()
  @IsString()
  collection: string;

  @IsNotEmpty()
  @IsString()
  owner: UserRef;

  @IsNotEmpty()
  @IsString()
  hash: string;

  @IsNotEmpty()
  @IsString()
  nonce: string;
}

export interface IWithdrawSubmissionDto {
  collection: string;
  owner: UserRef;
  hash: string;
  nonce: string;
  uniqueKey: string;
}

export class WithdrawSubmissionDto extends SubmitCallDTO {
  constructor(data: IWithdrawSubmissionDto) {
    super();
    this.collection = data?.collection ?? "";
    this.owner = data?.owner ?? asValidUserRef("service|null");
    this.hash = data?.hash ?? "";
    this.nonce = data?.nonce ?? "";
    this.uniqueKey = data?.uniqueKey ?? "";
  }

  @IsNotEmpty()
  @IsString()
  collection: string;

  @IsNotEmpty()
  @IsString()
  owner: UserRef;

  @IsNotEmpty()
  @IsHash("sha256")
  hash: string;

  @IsNotEmpty()
  @IsString()
  nonce: string;
}

export class WithdrawSubmissionResDto extends ChainCallDTO {
  @IsNotEmpty()
  @IsString()
  collection: string;

  @IsNotEmpty()
  @IsString()
  owner: UserRef;

  @IsNotEmpty()
  @IsString()
  hash: string;

  @IsNotEmpty()
  @IsString()
  nonce: string;
}

export interface IRevealSubmissionDto {
  collection: string;
  item: string;
  commitmentNonce: string;
  commitmentHash: string;
  salt: string;
  bid: BigNumber;
  uniqueKey: string;
}

export class RevealSubmissionDto extends SubmitCallDTO {
  constructor(args: unknown) {
    super();
    const data: IRevealSubmissionDto = args as IRevealSubmissionDto;
    this.collection = data?.collection ?? "";
    this.item = data?.item ?? "";
    this.commitmentNonce = data?.commitmentNonce ?? "";
    this.commitmentHash = data?.commitmentHash ?? "";
    this.salt = data?.salt ?? "";
    this.bid = data?.bid ?? new BigNumber("");
    this.uniqueKey = data?.uniqueKey ?? "";
  }

  @IsNotEmpty()
  @IsString()
  public collection: string;

  @IsNotEmpty()
  @IsString()
  public item: string;

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
}

export class RevealSubmissionResDto extends ChainCallDTO {
  @IsNotEmpty()
  @IsString()
  public collection: string;

  @IsNotEmpty()
  @IsString()
  public item: string;

  @IsNotEmpty()
  @IsString()
  public owner: UserRef;

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
}
