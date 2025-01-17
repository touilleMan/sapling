/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This software may be used and distributed according to the terms of the
 * GNU General Public License version 2.
 */

use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::anyhow;
use anyhow::Error;
use anyhow::Result;
use bookmarks_types::BookmarkKey;
use fbinit::FacebookInit;
use gitexport_tools::build_partial_commit_graph_for_export;
use gitexport_tools::rewrite_partial_changesets;
use mononoke_api::BookmarkFreshness;
use mononoke_api::ChangesetId;
use mononoke_api::RepoContext;
use mononoke_app::fb303::AliveService;
use mononoke_app::fb303::Fb303AppExtension;
use mononoke_app::MononokeApp;
use mononoke_app::MononokeAppBuilder;
use mononoke_types::NonRootMPath;
use print_graph::print_graph;
use print_graph::PrintGraphOptions;
use repo_authorization::AuthorizationContext;
use slog::debug;

use crate::types::GitExportArgs;

pub mod types {
    use std::path::PathBuf;

    use clap::Args;
    use clap::Parser;
    use mononoke_app::args::RepoArgs;

    #[derive(Debug, Args)]
    pub struct PrintGraphArgs {
        // Graph printing args for debugging and tests
        #[clap(long)]
        /// Print the commit graph of the source repo to the provided file.
        /// Used for integration tests.
        pub source_graph_output: Option<PathBuf>,

        #[clap(long)]
        /// Print the commit graph of the partial repo to the provided file
        /// Used for integration tests.
        pub partial_graph_output: Option<PathBuf>,

        /// Maximum distance from the initial changeset to any displayed
        /// changeset when printing a commit graph.
        #[clap(long, short, default_value_t = 10)]
        pub distance_limit: usize,
    }

    /// Mononoke Git Exporter
    #[derive(Debug, Parser)]
    pub struct GitExportArgs {
        /// Name of the hg repo being exported
        #[clap(flatten)]
        pub hg_repo_args: RepoArgs,

        /// Path to the git repo being created
        #[clap(long)]
        pub output: Option<PathBuf>, // TODO(T160787114): Make this required

        /// List of directories in `hg_repo` to be exported to a git repo
        #[clap(long, short('p'))]
        /// Paths in the source hg repo that should be exported to a git repo.
        pub export_paths: Vec<PathBuf>,
        // TODO(T160600443): support last revision argument
        // TODO(T160600443): support until_timestamp argument

        // -----------------------------------------------------------------
        // Graph printing args for debugging and tests
        #[clap(flatten)]
        pub print_graph_args: PrintGraphArgs,
    }
}

#[fbinit::main]
fn main(fb: FacebookInit) -> Result<(), Error> {
    let app: MononokeApp = MononokeAppBuilder::new(fb)
        .with_app_extension(Fb303AppExtension {})
        .build::<GitExportArgs>()?;

    app.run_with_monitoring_and_logging(async_main, "gitexport", AliveService)
}

async fn async_main(app: MononokeApp) -> Result<(), Error> {
    let args: GitExportArgs = app.args()?;
    let logger = app.logger();
    let ctx = app.new_basic_context();

    let repo = app.open_repo(&args.hg_repo_args).await?;

    let auth_ctx = AuthorizationContext::new_bypass_access_control();
    let repo_ctx: RepoContext = RepoContext::new(ctx, auth_ctx.into(), repo, None, None).await?;

    // TODO(T160600443): support using a specific changeset as starting commit,
    // instead of a bookmark.
    let bookmark_key = BookmarkKey::from_str("master")?;

    let cs_ctx = repo_ctx
        .resolve_bookmark(&bookmark_key, BookmarkFreshness::MostRecent)
        .await?
        .unwrap();

    if let Some(source_graph_output) = args.print_graph_args.source_graph_output.clone() {
        print_commit_graph(
            &repo_ctx,
            cs_ctx.id(),
            source_graph_output,
            args.print_graph_args.distance_limit,
        )
        .await?;
    };

    let export_paths = args
        .export_paths
        .into_iter()
        .map(|p| TryFrom::try_from(p.as_os_str()))
        .collect::<Result<Vec<NonRootMPath>>>()?;

    let (changesets, cs_parents) =
        build_partial_commit_graph_for_export(logger, export_paths.clone(), cs_ctx).await?;

    debug!(logger, "changesets: {:#?}", changesets);
    debug!(logger, "changeset parents: {:?}", cs_parents);

    let temp_repo_ctx =
        rewrite_partial_changesets(app.fb, repo_ctx, changesets, &cs_parents, export_paths).await?;

    let temp_master_csc = temp_repo_ctx
        .resolve_bookmark(
            &BookmarkKey::from_str("master")?,
            BookmarkFreshness::MostRecent,
        )
        .await?
        .ok_or(anyhow!("Couldn't find master bookmark in temp repo."))?;

    if let Some(partial_graph_output) = args.print_graph_args.partial_graph_output.clone() {
        print_commit_graph(
            &temp_repo_ctx,
            temp_master_csc.id(),
            partial_graph_output,
            args.print_graph_args.distance_limit,
        )
        .await?;
    };

    // TODO(T160787114): export temporary repo to git repo

    Ok(())
}

async fn print_commit_graph(
    repo_ctx: &RepoContext,
    cs_id: ChangesetId,
    output: PathBuf,
    limit: usize,
) -> Result<()> {
    let print_graph_args = PrintGraphOptions {
        limit,
        display_message: true,
        display_id: true,
        display_file_changes: true,
        display_author: false,
        display_author_date: false,
    };
    let changesets = vec![cs_id];

    let output_file = Box::new(File::create(output).unwrap());

    print_graph(
        repo_ctx.ctx(),
        repo_ctx.repo(),
        changesets,
        print_graph_args,
        output_file,
    )
    .await
}
