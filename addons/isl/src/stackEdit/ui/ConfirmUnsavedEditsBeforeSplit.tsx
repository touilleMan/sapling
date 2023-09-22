/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

import type {FieldsBeingEdited} from '../../CommitInfoView/types';
import type {CommitInfo} from '../../types';

import {
  commitFieldsBeingEdited,
  editedCommitMessages,
  unsavedFieldsBeingEdited,
} from '../../CommitInfoView/CommitInfoState';
import {commitMessageFieldsSchema} from '../../CommitInfoView/CommitMessageFields';
import {FlexSpacer} from '../../ComponentUtils';
import {Tooltip} from '../../Tooltip';
import {T, t} from '../../i18n';
import {useModal} from '../../useModal';
import {VSCodeButton, VSCodeDivider} from '@vscode/webview-ui-toolkit/react';
import {useRecoilCallback, useRecoilValue} from 'recoil';
import {Icon} from 'shared/Icon';

import './ConfirmUnsavedEditsBeforeSplit.css';

export function useConfirmUnsavedEditsBeforeSplit(): (
  commits: Array<CommitInfo>,
) => Promise<boolean> {
  const showModal = useModal();
  const showConfirmation = useRecoilCallback(
    ({snapshot}) =>
      async (commits: Array<CommitInfo>): Promise<boolean> => {
        const editedCommits = commits
          .map(commit => [
            commit,
            snapshot.getLoadable(unsavedFieldsBeingEdited(commit.hash)).valueMaybe(),
          ])
          .filter(([_, f]) => f != null) as Array<[CommitInfo, FieldsBeingEdited]>;
        if (editedCommits.some(([_, f]) => Object.values(f).some(Boolean))) {
          const continueWithSplit = await showModal<boolean>({
            type: 'custom',
            component: ({returnResultAndDismiss}) => (
              <PreSplitUnsavedEditsConfirmationModal
                editedCommits={editedCommits}
                returnResultAndDismiss={returnResultAndDismiss}
              />
            ),
            title: t('Save edits before splitting?'),
          });
          return continueWithSplit === true;
        }
        return true;
      },
  );

  return (commits: Array<CommitInfo>) => {
    return showConfirmation(commits);
  };
}

function PreSplitUnsavedEditsConfirmationModal({
  editedCommits,
  returnResultAndDismiss,
}: {
  editedCommits: Array<[CommitInfo, FieldsBeingEdited]>;
  returnResultAndDismiss: (continueWithSplit: boolean) => unknown;
}) {
  const schema = useRecoilValue(commitMessageFieldsSchema);

  const resetEditedCommitMessage = useRecoilCallback(({reset}) => (commit: CommitInfo) => {
    reset(editedCommitMessages(commit.hash));
    reset(commitFieldsBeingEdited);
  });

  return (
    <div className="confirm-unsaved-edits-pre-split" data-testid="confirm-unsaved-edits-pre-split">
      <>
        <div>
          <T count={editedCommits.length}>confirmUnsavedEditsBeforeSplit</T>
        </div>
        <div className="commits-with-unsaved-changes">
          {editedCommits.map(([commit, fields]) => (
            <>
              <span key={`${commit.hash}`} className="commit">
                <span className="unsaved-message-indicator">
                  <Tooltip title={t('This commit has unsaved changes to its message')}>
                    <Icon icon="circle-large-filled" />
                  </Tooltip>
                </span>
                {commit.title}
              </span>
              <span key={`${commit.hash}-fields`} className="byline">
                <T
                  replace={{
                    $commitTitle: commit.title,
                    $fields: (
                      <>
                        {Object.entries(fields)
                          .filter(([, value]) => value)
                          .map(([field]) => {
                            const icon = schema.find(f => f.key === field)?.icon;
                            return (
                              <span key={field} className="field-name">
                                {icon && <Icon icon={icon} />}
                                {field}
                              </span>
                            );
                          })}
                      </>
                    ),
                  }}>
                  unsaved changes to $fields
                </T>
              </span>
            </>
          ))}
        </div>
        <VSCodeDivider />
        <div className="use-modal-buttons">
          <FlexSpacer />
          <VSCodeButton appearance="secondary" onClick={() => returnResultAndDismiss(false)}>
            <T>Cancel</T>
          </VSCodeButton>
          <VSCodeButton
            appearance="secondary"
            onClick={() => {
              for (const [commit] of editedCommits) {
                resetEditedCommitMessage(commit);
              }
              returnResultAndDismiss(true); // continue with split
            }}>
            <T>Discard Edits</T>
          </VSCodeButton>
          <VSCodeButton
            appearance="primary"
            onClick={() => {
              // Unsaved edits will be automatically loaded by the split as the commits' text
              returnResultAndDismiss(true); // continue with split
            }}>
            <T>Save Edits</T>
          </VSCodeButton>
        </div>
      </>
    </div>
  );
}