/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This software may be used and distributed according to the terms of the
 * GNU General Public License version 2.
 */

mod count_underived;
mod exists;

use anyhow::Context;
use anyhow::Result;
use bonsai_git_mapping::BonsaiGitMapping;
use bonsai_hg_mapping::BonsaiHgMapping;
use bookmarks::Bookmarks;
use changesets::Changesets;
use clap::Parser;
use clap::Subcommand;
use commit_graph::CommitGraph;
use filenodes::Filenodes;
use mononoke_app::args::RepoArgs;
use mononoke_app::MononokeApp;
use repo_blobstore::RepoBlobstore;
use repo_derived_data::RepoDerivedData;
use repo_identity::RepoIdentity;

use self::count_underived::count_underived;
use self::count_underived::CountUnderivedArgs;
use self::exists::exists;
use self::exists::ExistsArgs;

#[facet::container]
struct Repo {
    #[facet]
    repo_identity: RepoIdentity,
    #[facet]
    repo_derived_data: RepoDerivedData,
    #[facet]
    bonsai_hg_mapping: dyn BonsaiHgMapping,
    #[facet]
    bonsai_git_mapping: dyn BonsaiGitMapping,
    #[facet]
    changesets: dyn Changesets,
    #[facet]
    repo_blobstore: RepoBlobstore,
    #[facet]
    bookmarks: dyn Bookmarks,
    #[facet]
    commit_graph: CommitGraph,
    #[facet]
    filenodes: dyn Filenodes,
}

/// Request information about derived data
#[derive(Parser)]
pub struct CommandArgs {
    #[clap(flatten)]
    repo: RepoArgs,

    #[clap(subcommand)]
    subcommand: DerivedDataSubcommand,
}

#[derive(Subcommand)]
enum DerivedDataSubcommand {
    /// Get the changeset of a bookmark
    Exists(ExistsArgs),
    /// Count how many ancestors of a given commit weren't derived
    CountUnderived(CountUnderivedArgs),
}

pub async fn run(app: MononokeApp, args: CommandArgs) -> Result<()> {
    let ctx = app.new_basic_context();

    let repo: Repo = app
        .open_repo(&args.repo)
        .await
        .context("Failed to open repo")?;

    match args.subcommand {
        DerivedDataSubcommand::Exists(args) => exists(&ctx, &repo, args).await?,
        DerivedDataSubcommand::CountUnderived(args) => count_underived(&ctx, &repo, args).await?,
    }

    Ok(())
}

mod args {
    use std::sync::Arc;

    use anyhow::Result;
    use clap::builder::PossibleValuesParser;
    use clap::Args;
    use context::CoreContext;
    use derived_data_utils::derived_data_utils;
    use derived_data_utils::derived_data_utils_for_config;
    use derived_data_utils::DerivedUtils;
    use derived_data_utils::DEFAULT_BACKFILLING_CONFIG_NAME;
    use derived_data_utils::POSSIBLE_DERIVED_TYPES;

    use super::Repo;

    #[derive(Args)]
    pub(super) struct DerivedUtilsArgs {
        /// Use backfilling config rather than enabled config
        #[clap(long)]
        pub(super) backfill: bool,
        /// Sets the name for backfilling derived data types config
        #[clap(long, default_value = DEFAULT_BACKFILLING_CONFIG_NAME)]
        pub(super) backfill_config_name: String,
        /// Type of derived data
        #[clap(long, short = 'T', value_parser = PossibleValuesParser::new(POSSIBLE_DERIVED_TYPES))]
        pub(super) derived_data_type: String,
    }

    impl DerivedUtilsArgs {
        pub(super) fn derived_utils(
            self,
            ctx: &CoreContext,
            repo: &Repo,
        ) -> Result<Arc<dyn DerivedUtils>> {
            if self.backfill {
                derived_data_utils_for_config(
                    ctx.fb,
                    repo,
                    self.derived_data_type,
                    self.backfill_config_name,
                )
            } else {
                derived_data_utils(ctx.fb, &repo, self.derived_data_type)
            }
        }
    }
}
