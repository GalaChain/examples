import { asValidUserRef, createValidDTO } from "@gala-chain/api";
import { GalaChainContext, putChainObject } from "@gala-chain/chaincode";

import { CommitSubmissionDto, CommitSubmissionResDto, SubmissionCommitment } from "./api";

export async function commitSubmission(
  ctx: GalaChainContext,
  dto: CommitSubmissionDto
): Promise<CommitSubmissionResDto> {
  const { collection, hash } = dto;
  const nonce = dto.uniqueKey;
  const owner = asValidUserRef(ctx.callingUser);

  const entry = new SubmissionCommitment({
    collection,
    hash,
    owner,
    nonce
  });

  await putChainObject(ctx, entry);

  const response = await createValidDTO(CommitSubmissionResDto, {
    collection,
    hash,
    owner,
    nonce
  });

  return response;
}
