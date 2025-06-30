import { ValidationFailedError, asValidUserRef, createValidDTO } from "@gala-chain/api";
import { GalaChainContext, deleteChainObject, getObjectByKey, putChainObject } from "@gala-chain/chaincode";

import { RevealSubmissionDto, RevealSubmissionResDto, Submission, SubmissionCommitment } from "./api";

export async function revealSubmission(
  ctx: GalaChainContext,
  dto: RevealSubmissionDto
): Promise<RevealSubmissionResDto> {
  const { collection, item, commitmentNonce, commitmentHash, salt, bid } = dto;

  const owner = asValidUserRef(ctx.callingUser);

  const commitmentKey = SubmissionCommitment.getCompositeKeyFromParts(SubmissionCommitment.INDEX_KEY, [
    collection,
    owner,
    commitmentHash,
    commitmentNonce
  ]);

  const commitment = await getObjectByKey(ctx, SubmissionCommitment, commitmentKey);

  const submission = new Submission({
    collection,
    item,
    owner,
    bid,
    commitmentNonce,
    commitmentHash,
    salt
  });

  const validSubmissionHash = submission.verifyHash(commitmentHash);

  if (!validSubmissionHash) {
    throw new ValidationFailedError(
      `Failed to validate commitmentHash against provided submission properties: ` +
        `${submission.concatenateCommitment()}, generated hash: ${submission.generateHash()}, ` +
        `commitmentHash: ${commitmentHash}`
    );
  }

  await deleteChainObject(ctx, commitment);
  await putChainObject(ctx, submission);

  const response: RevealSubmissionResDto = await createValidDTO(RevealSubmissionResDto, {
    collection,
    item,
    owner,
    commitmentNonce,
    commitmentHash,
    salt,
    bid
  });

  return response;
}
