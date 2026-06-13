# Changelog

## [0.4.2](https://github.com/git-ents/git-metadata/compare/git-metadata-v0.4.1...git-metadata-v0.4.2) (2026-06-13)


### Bug Fixes

* Remove tests from Cargo package metadata ([0e5c118](https://github.com/git-ents/git-metadata/commit/0e5c1184ce3bf8a8cb1920069f7bda9dd288c41c))

## [0.4.1](https://github.com/git-ents/git-metadata/compare/git-metadata-v0.4.0...git-metadata-v0.4.1) (2026-05-17)


### chore

* Pin next release ([f523db8](https://github.com/git-ents/git-metadata/commit/f523db8650d66d868b87ee112a6073d5402c2d59))


### Documentation

* Add code of conduct ([62cdabd](https://github.com/git-ents/git-metadata/commit/62cdabdaf0ee526256a5477835be2b4c4e61ee90))
* Add contribution guide ([cb240be](https://github.com/git-ents/git-metadata/commit/cb240becb39f6a87e486f5757cd4e27e79f138cc))
* Add note for testers in README ([f219c20](https://github.com/git-ents/git-metadata/commit/f219c205461985c6b3b1eaf2d15eb1a9ab7f1db1))
* Add reference to code of conduct in README ([f71c62d](https://github.com/git-ents/git-metadata/commit/f71c62d61d77b2fdd45eaa52195832409eba39f3))
* Add reference to contribution guide in README ([34502d5](https://github.com/git-ents/git-metadata/commit/34502d506c2bd9204c4f0e20dd7b467033f5a522))

## [0.4.0](https://github.com/git-ents/git-metadata/compare/git-metadata-v0.3.0...git-metadata-v0.4.0) (2026-05-17)


### Features

* Add -a/--all flag to list subcommand ([8fed577](https://github.com/git-ents/git-metadata/commit/8fed577dfcb41f9fbd7c25bac59f17408efb310f))
* Add `Edit` and `Merge` CLI subcommands ([eb21231](https://github.com/git-ents/git-metadata/commit/eb212313658af7ed761aeb003f98877290c5fb80))
* Add `Executor::merge` for 3-way merge of metadata refs ([eb21231](https://github.com/git-ents/git-metadata/commit/eb212313658af7ed761aeb003f98877290c5fb80))
* Add `Executor::read_blob_at` for reading blob entries ([eb21231](https://github.com/git-ents/git-metadata/commit/eb212313658af7ed761aeb003f98877290c5fb80))
* Add edit and merge subcommands ([eb21231](https://github.com/git-ents/git-metadata/commit/eb212313658af7ed761aeb003f98877290c5fb80))


### Bug Fixes

* Make object a keyword argument in add, remove, and edit subcommands ([2b88890](https://github.com/git-ents/git-metadata/commit/2b88890e32ef86f94a77b185ac016ac1f89ff264))

## [0.3.0-rc.2](https://github.com/git-ents/git-metadata/compare/git-metadata-v0.3.0-rc.1...git-metadata-v0.3.0-rc.2) (2026-05-16)


### Features

* `metadata` validates input and ref-target shape ([dbeb94c](https://github.com/git-ents/git-metadata/commit/dbeb94c95de99f0008b5fa606912b00ccefa4370))
* Accept optional author signature in Ledger::create ([cb4d735](https://github.com/git-ents/git-metadata/commit/cb4d735638eebdb907cf69bd0bf5d839268513c7)), closes [#10](https://github.com/git-ents/git-metadata/issues/10)
* Add --author-name and --author-email to create subcommand ([cb4d735](https://github.com/git-ents/git-metadata/commit/cb4d735638eebdb907cf69bd0bf5d839268513c7))
* Add --link &lt;rev&gt; to add (resolved permalink, peels tags) ([8499883](https://github.com/git-ents/git-metadata/commit/84998838f9decc1d7f23d5b36779096e2ad90f21))
* Add --link-ref &lt;name&gt; to add (unresolved ref name as blob) ([8499883](https://github.com/git-ents/git-metadata/commit/84998838f9decc1d7f23d5b36779096e2ad90f21))
* Add `AlreadyExists`, `InvalidRootType`, `FanoutPathConflict` errors ([dbeb94c](https://github.com/git-ents/git-metadata/commit/dbeb94c95de99f0008b5fa606912b00ccefa4370))
* Add `ContentAddressable` and `Pointer` traits ([677936a](https://github.com/git-ents/git-metadata/commit/677936a4579a0c27c97a02e17c48e275c3be5eab))
* Add `DEFAULT_FANOUT` constant ([82f6a4f](https://github.com/git-ents/git-metadata/commit/82f6a4f672120508454a1635e6779b0a9c1744b7))
* Add `Error::UnexpectedKind` variant ([d57c2ba](https://github.com/git-ents/git-metadata/commit/d57c2ba0a09d06fe31581563b87c7fa3929c10cf))
* Add `Executor::repo()` accessor ([8ac3697](https://github.com/git-ents/git-metadata/commit/8ac369788e7541a64bc749943754d23a77462c96))
* Add `Executor::stale()` read-only counterpart to `prune` ([8ac3697](https://github.com/git-ents/git-metadata/commit/8ac369788e7541a64bc749943754d23a77462c96))
* Add `InvalidFanoutDepth` and `FanoutFind` error variants ([82f6a4f](https://github.com/git-ents/git-metadata/commit/82f6a4f672120508454a1635e6779b0a9c1744b7))
* Add `list <object>` subcommand (replaces old `show` flat output) ([ce0fc2a](https://github.com/git-ents/git-metadata/commit/ce0fc2a01fdabbc55f88bca24b108653c5fd8f70))
* Add `message: Option<&str>` to `MetadataRepository::metadata_delete` ([8ac3697](https://github.com/git-ents/git-metadata/commit/8ac369788e7541a64bc749943754d23a77462c96))
* Add `message: Option<&str>` to `MetadataRepository::metadata` ([8ac3697](https://github.com/git-ents/git-metadata/commit/8ac369788e7541a64bc749943754d23a77462c96))
* Add `message`/`author` params to `Executor::upsert`, `import`, `remove` ([8ac3697](https://github.com/git-ents/git-metadata/commit/8ac369788e7541a64bc749943754d23a77462c96))
* Add `Metadata::id` and `Metadata::data` accessors ([0c8fb44](https://github.com/git-ents/git-metadata/commit/0c8fb44e1f40b2fc8d5917f8ae59d2ee9e715d06))
* Add `Metadata::id` and `Metadata::data` accessors ([9757be5](https://github.com/git-ents/git-metadata/commit/9757be5810fab143070ece4ab43496f9bb4ba7fe))
* Add `tree::remove_leaf` helper that prunes empty intermediates ([8764905](https://github.com/git-ents/git-metadata/commit/876490578e4f1f43fc9ef726a2fb1f4013a89b8c))
* Add `TreeId` newtype with `From<gix::Tree>` and `From<TreeId> for gix::ObjectId` ([f5d5c88](https://github.com/git-ents/git-metadata/commit/f5d5c88912bff2b4fe1abc49e6a11a5d66cac49d))
* Add `Tx::with_message()` builder; default commit message is `"store: commit transaction"` ([d57c2ba](https://github.com/git-ents/git-metadata/commit/d57c2ba0a09d06fe31581563b87c7fa3929c10cf))
* Add CLI ([ff68888](https://github.com/git-ents/git-metadata/commit/ff68888d90286dcfbaaf5aa554ab7aa7c530abe9))
* Add Error::FanoutDepthConflict variant ([a77bc5d](https://github.com/git-ents/git-metadata/commit/a77bc5da9d2e55ab36531cef6156d5030f1cfe4c))
* Add FileMode enum (Blob, Executable, Tree, Commit) ([c72b96c](https://github.com/git-ents/git-metadata/commit/c72b96ca0ca85cb91daa585f3db2be653120a6e8))
* Add git-db crate to workspace ([e2ed0fc](https://github.com/git-ents/git-metadata/commit/e2ed0fc74fa1cd71bbb5b863a8dfd3f247ade4fd))
* Add global --force flag used by add, copy, and man-page generation ([676d45a](https://github.com/git-ents/git-metadata/commit/676d45a72e2111a4f43181f4111a848939d5cbf0))
* Add IdStrategy::CommitOid for commit-OID-keyed entity refs ([31c8dc3](https://github.com/git-ents/git-metadata/commit/31c8dc3ff376f0a88a78fdf444053c95ae0336da)), closes [#6](https://github.com/git-ents/git-metadata/issues/6)
* Add Mutation::Pin for inserting entries that reference existing objects ([c72b96c](https://github.com/git-ents/git-metadata/commit/c72b96ca0ca85cb91daa585f3db2be653120a6e8))
* Add optional author override to Ledger::create ([cb4d735](https://github.com/git-ents/git-metadata/commit/cb4d735638eebdb907cf69bd0bf5d839268513c7))
* Add print_tree_entry helper in main.rs ([4f09287](https://github.com/git-ents/git-metadata/commit/4f092877208d65bbf7bda2f3269b263e7ed67d12))
* Add store abstractions ([260820f](https://github.com/git-ents/git-metadata/commit/260820f4e0b60b3132664bb53338daed49da4bea))
* Add validate_metadata_tree to MetadataRepository trait ([a77bc5d](https://github.com/git-ents/git-metadata/commit/a77bc5da9d2e55ab36531cef6156d5030f1cfe4c))
* Enable gix tree-editor feature for structural sharing ([bf946a9](https://github.com/git-ents/git-metadata/commit/bf946a9d8048d07a23b0bee0da6fac422ef8c5ad))
* Implement `ContentAddressable`, `Ref`, and `Transaction` over gix ([7949943](https://github.com/git-ents/git-metadata/commit/79499433db139dfbc4b6ef10850aeeb9560a0cc0))
* Implement `Executor::upsert` ([0c8fb44](https://github.com/git-ents/git-metadata/commit/0c8fb44e1f40b2fc8d5917f8ae59d2ee9e715d06))
* Implement `Executor::upsert` ([333a29f](https://github.com/git-ents/git-metadata/commit/333a29ff1694a62ed44297ddbd935bb2bb7e1826))
* Implement `find_metadata` ([0c8fb44](https://github.com/git-ents/git-metadata/commit/0c8fb44e1f40b2fc8d5917f8ae59d2ee9e715d06))
* Implement `find_metadata` ([119791d](https://github.com/git-ents/git-metadata/commit/119791d6ee48b359468e0ef60f7c43d147bc6395))
* Implement `metadata_default_ref` returning `refs/metadata/commits` ([82f6a4f](https://github.com/git-ents/git-metadata/commit/82f6a4f672120508454a1635e6779b0a9c1744b7))
* Implement `metadata_delete` and add `tree::remove_leaf` ([0c8fb44](https://github.com/git-ents/git-metadata/commit/0c8fb44e1f40b2fc8d5917f8ae59d2ee9e715d06))
* Implement `metadata_delete` and add `tree::remove_leaf` ([8764905](https://github.com/git-ents/git-metadata/commit/876490578e4f1f43fc9ef726a2fb1f4013a89b8c))
* Implement `metadata`, mirror libgit2 notes shape ([0c8fb44](https://github.com/git-ents/git-metadata/commit/0c8fb44e1f40b2fc8d5917f8ae59d2ee9e715d06))
* Implement `metadata`, mirror libgit2 notes shape ([dbeb94c](https://github.com/git-ents/git-metadata/commit/dbeb94c95de99f0008b5fa606912b00ccefa4370))
* Implement compile_patterns and tree_is_empty helpers ([7269b52](https://github.com/git-ents/git-metadata/commit/7269b528b18c8ce841abb32cec860ab022c4c924))
* Implement Executor::copy ([7269b52](https://github.com/git-ents/git-metadata/commit/7269b528b18c8ce841abb32cec860ab022c4c924))
* Implement Executor::remove with glob-pattern matching ([7269b52](https://github.com/git-ents/git-metadata/commit/7269b528b18c8ce841abb32cec860ab022c4c924))
* Implement Executor::remove, stale, copy, prune ([7269b52](https://github.com/git-ents/git-metadata/commit/7269b528b18c8ce841abb32cec860ab022c4c924))
* Implement Executor::stale and Executor::prune ([7269b52](https://github.com/git-ents/git-metadata/commit/7269b528b18c8ce841abb32cec860ab022c4c924))
* Implement Executor::upsert using tree::insert_leaf directly ([7302605](https://github.com/git-ents/git-metadata/commit/73026056fa67fceab5d5ffa3923eb4f47c5777d7))
* Implement Executor::upsert; parameterize insert_leaf over leaf_kind ([0c8fb44](https://github.com/git-ents/git-metadata/commit/0c8fb44e1f40b2fc8d5917f8ae59d2ee9e715d06))
* Implement Executor::upsert; parameterize insert_leaf over leaf_kind ([7302605](https://github.com/git-ents/git-metadata/commit/73026056fa67fceab5d5ffa3923eb4f47c5777d7))
* Implement Store and Tx for phase 1 transaction ([bf946a9](https://github.com/git-ents/git-metadata/commit/bf946a9d8048d07a23b0bee0da6fac422ef8c5ad))
* Install nextest via taiki-e/install-action@nextest ([8b9181f](https://github.com/git-ents/git-metadata/commit/8b9181f06b9fd55169db2eb2c700a2b5123ac971))
* Make `list` show flat tree entries and `show` render a tree ([ce0fc2a](https://github.com/git-ents/git-metadata/commit/ce0fc2a01fdabbc55f88bca24b108653c5fd8f70))
* Open editor for `add` when no content is provided interactively ([2a7cd53](https://github.com/git-ents/git-metadata/commit/2a7cd530c9aff7984219977b76ce5c5b193d63aa))
* Print man page path on write and require --force to overwrite ([676d45a](https://github.com/git-ents/git-metadata/commit/676d45a72e2111a4f43181f4111a848939d5cbf0))
* Read `.fanout` depth blob in `metadatas` ([82f6a4f](https://github.com/git-ents/git-metadata/commit/82f6a4f672120508454a1635e6779b0a9c1744b7))
* Read committer identity from git config via `repo.committer()`, falling back to generic defaults ([d57c2ba](https://github.com/git-ents/git-metadata/commit/d57c2ba0a09d06fe31581563b87c7fa3929c10cf))
* Render `show` output as a tree via `termtree` ([ce0fc2a](https://github.com/git-ents/git-metadata/commit/ce0fc2a01fdabbc55f88bca24b108653c5fd8f70))
* Require `.fanout` depth in metadata trees and document `MetadataRepository` ([0c8fb44](https://github.com/git-ents/git-metadata/commit/0c8fb44e1f40b2fc8d5917f8ae59d2ee9e715d06))
* Require `.fanout` depth in metadata trees and document `MetadataRepository` ([82f6a4f](https://github.com/git-ents/git-metadata/commit/82f6a4f672120508454a1635e6779b0a9c1744b7))
* Rework `add` to support --link/--link-ref; make --path optional ([8499883](https://github.com/git-ents/git-metadata/commit/84998838f9decc1d7f23d5b36779096e2ad90f21))
* Run doctests with cargo test --doc ([8b9181f](https://github.com/git-ents/git-metadata/commit/8b9181f06b9fd55169db2eb2c700a2b5123ac971))
* Run tests with cargo nextest run ([8b9181f](https://github.com/git-ents/git-metadata/commit/8b9181f06b9fd55169db2eb2c700a2b5123ac971))
* Scaffolding for `git-store` ([4480796](https://github.com/git-ents/git-metadata/commit/4480796879f5295864c90cf1c4463e1df065140d))
* Split --generate-man-page into bool flag plus --man-dir ([676d45a](https://github.com/git-ents/git-metadata/commit/676d45a72e2111a4f43181f4111a848939d5cbf0))
* Split `Pointer` into `Ref` and `Transaction` traits ([7ff0b5f](https://github.com/git-ents/git-metadata/commit/7ff0b5f49aa7e1d2f4b6fe661aafc35d23de4b81))
* Store::open/init bind to refs/db/&lt;n&gt; ([bf946a9](https://github.com/git-ents/git-metadata/commit/bf946a9d8048d07a23b0bee0da6fac422ef8c5ad))
* Tx::commit with configurable CAS retry loop ([bf946a9](https://github.com/git-ents/git-metadata/commit/bf946a9d8048d07a23b0bee0da6fac422ef8c5ad))
* Tx::get/put/delete/list over nested tree paths ([bf946a9](https://github.com/git-ents/git-metadata/commit/bf946a9d8048d07a23b0bee0da6fac422ef8c5ad))


### Bug Fixes

* --force is no longer global; add and copy each declare their own --force flag ([c06cd88](https://github.com/git-ents/git-metadata/commit/c06cd88847b9f2a3813279fc8face130cd930756))
* Add initial_depth: Option&lt;u8&gt; to MetadataRepository::metadata(); wire --shard-level through upsert ([c06cd88](https://github.com/git-ents/git-metadata/commit/c06cd88847b9f2a3813279fc8face130cd930756))
* Bind `metadata_default_ref()` result so the borrow outlives the match ([6afe663](https://github.com/git-ents/git-metadata/commit/6afe66336bbc0131c277996f849338247deb1bb4))
* Change `remove` return type to `Result<Option<gix::ObjectId>>` ([707709d](https://github.com/git-ents/git-metadata/commit/707709de871c611592dd6e4f9ba59acc1b80b8cd))
* Correct fanout traversal and error propagation in `metadatas` ([0c8fb44](https://github.com/git-ents/git-metadata/commit/0c8fb44e1f40b2fc8d5917f8ae59d2ee9e715d06))
* Correct fanout traversal and error propagation in `metadatas` ([6afe663](https://github.com/git-ents/git-metadata/commit/6afe66336bbc0131c277996f849338247deb1bb4))
* Correct generate-man arguments ([5e879a1](https://github.com/git-ents/git-metadata/commit/5e879a126b6b2718ce8d4498ee6974aa458ad34d))
* Correct stale `[`Repo`]` link in module doc to `[`Executor`]` ([707709d](https://github.com/git-ents/git-metadata/commit/707709de871c611592dd6e4f9ba59acc1b80b8cd))
* Create man dir if necessary ([21d2a15](https://github.com/git-ents/git-metadata/commit/21d2a15314b5a7de665c52d26a79e08cbbe20322))
* Default man page output dir to XDG_DATA_HOME ([f713512](https://github.com/git-ents/git-metadata/commit/f7135128c2ea1d90387cdbb7714d15cc760c8f0e))
* Document `force` behavior in `upsert`, `copy`, and `import` ([707709d](https://github.com/git-ents/git-metadata/commit/707709de871c611592dd6e4f9ba59acc1b80b8cd))
* Document copy --force as a destructive wholesale replace ([c06cd88](https://github.com/git-ents/git-metadata/commit/c06cd88847b9f2a3813279fc8face130cd930756))
* Ensure_fanout_blob now errors on type/depth conflict, no-ops on match ([a77bc5d](https://github.com/git-ents/git-metadata/commit/a77bc5da9d2e55ab36531cef6156d5030f1cfe4c))
* Fail fast on non-hex fanout leaves ([6afe663](https://github.com/git-ents/git-metadata/commit/6afe66336bbc0131c277996f849338247deb1bb4))
* Fix editor template line-continuation formatting ([c06cd88](https://github.com/git-ents/git-metadata/commit/c06cd88847b9f2a3813279fc8face130cd930756))
* List and show return empty output instead of erroring when no metadata ref exists ([c06cd88](https://github.com/git-ents/git-metadata/commit/c06cd88847b9f2a3813279fc8face130cd930756))
* Make object positional in remove (was -o/--object, inconsistent with add/show) ([c06cd88](https://github.com/git-ents/git-metadata/commit/c06cd88847b9f2a3813279fc8face130cd930756))
* Metadata::new no longer requires id to be a blob; any object type is valid ([c06cd88](https://github.com/git-ents/git-metadata/commit/c06cd88847b9f2a3813279fc8face130cd930756))
* Peel metadata ref to a tree before traversal ([6afe663](https://github.com/git-ents/git-metadata/commit/6afe66336bbc0131c277996f849338247deb1bb4))
* Propagate ref-find, peel, and traverse errors via new variants ([6afe663](https://github.com/git-ents/git-metadata/commit/6afe66336bbc0131c277996f849338247deb1bb4))
* Publish only the crate matching the release tag ([912b8ce](https://github.com/git-ents/git-metadata/commit/912b8cecb7d9257a303697be9589edd88960af21))
* Publish pre-release crates to crates.io ([6f1d588](https://github.com/git-ents/git-metadata/commit/6f1d588f98cd5b8639dad12ded683f00e08fd402))
* Reject empty paths in `Tx` operations ([f541852](https://github.com/git-ents/git-metadata/commit/f541852b7d85fcfa3b710fb5213c7e1d6ea9d05f))
* Reject non-blob `.fanout` entries (symlink, gitlink) with `InvalidFanoutDepth` ([6e20304](https://github.com/git-ents/git-metadata/commit/6e20304335805ed05a171a65ee7adbbee23e2bbf))
* Remove --shard-level from copy (concept does not apply) ([c06cd88](https://github.com/git-ents/git-metadata/commit/c06cd88847b9f2a3813279fc8face130cd930756))
* Remove manual version pins ([ed13199](https://github.com/git-ents/git-metadata/commit/ed13199df77777a0f234730264ec3f4891610c11))
* Remove redundant `get_ref` method ([707709d](https://github.com/git-ents/git-metadata/commit/707709de871c611592dd6e4f9ba59acc1b80b8cd))
* Replace panicking `into_commit()`/`into_tree()` with `try_into_commit()`/`try_into_tree()`, returning `Error::UnexpectedKind` on mismatch ([d57c2ba](https://github.com/git-ents/git-metadata/commit/d57c2ba0a09d06fe31581563b87c7fa3929c10cf))
* Require positional argument to be last ([4827a71](https://github.com/git-ents/git-metadata/commit/4827a7115738690385c6082b6d7035c8d3eb8810))
* Resolve clippy -D warnings diagnostics in stub impls ([2793dde](https://github.com/git-ents/git-metadata/commit/2793dde6e9537bcf7014f60c64280ee352e3eff8))
* Resolve diagnostics in `metadatas` impl ([0818601](https://github.com/git-ents/git-metadata/commit/0818601fa578f15d2c1767a5acd089644469c843))
* Resolve nine CLI issues and rename default metadata ref ([c06cd88](https://github.com/git-ents/git-metadata/commit/c06cd88847b9f2a3813279fc8face130cd930756))
* Revert `upsert` to `todo!()` pending rewrite ([707709d](https://github.com/git-ents/git-metadata/commit/707709de871c611592dd6e4f9ba59acc1b80b8cd))
* Update --path help to mention --link and --link-ref as alternatives ([c06cd88](https://github.com/git-ents/git-metadata/commit/c06cd88847b9f2a3813279fc8face130cd930756))


### Performance Improvements

* Hoist hex buffer allocation out of the entry loop ([6afe663](https://github.com/git-ents/git-metadata/commit/6afe66336bbc0131c277996f849338247deb1bb4))

## [0.3.0-rc.1](https://github.com/git-ents/git-metadata/compare/git-metadata-v0.2.1...git-metadata-v0.3.0-rc.1) (2026-03-26)


### Features

* Add git-chain ([2a2cfea](https://github.com/git-ents/git-metadata/commit/2a2cfeaa9a78ee3d4a764008c14c9acb90672594))
* Add git-ledger ([2a2cfea](https://github.com/git-ents/git-metadata/commit/2a2cfeaa9a78ee3d4a764008c14c9acb90672594))
* Implement relation operations in git-metadata ([7725be8](https://github.com/git-ents/git-metadata/commit/7725be82c332da618ab0bfc6d6d39d9f46ee064b))


### Bug Fixes

* Apply clippy suggestions (search_is_some in tests, fmt in main.rs) ([6f83928](https://github.com/git-ents/git-metadata/commit/6f83928928c114a0cfd54a2b9d98732c64a461a9))
* Handle slash in link keys and batch prune commits in git-metadata ([a37fb3e](https://github.com/git-ents/git-metadata/commit/a37fb3eb1430d869e77be663681f8223876c004d))

## [0.2.1](https://github.com/git-ents/git-metadata/compare/git-metadata-v0.2.0...git-metadata-v0.2.1) (2026-03-20)

### Miscellaneous Chores

* Move project to new repository ([d961a8c](https://github.com/git-ents/git-metadata/commit/d961a8cc0cf8459b790b4d614bd27c0e4d24cd15))
* Pin release ([5c1728a](https://github.com/git-ents/git-metadata/commit/5c1728a684724ed6507b9f9f06bf563f21db796e))

## 0.1.0 (2026-03-11)

### ⚠ BREAKING CHANGES

* metadata_remove no longer takes MetadataOptions

### Features

* Add executor (exe) and CLI implementation ([10d9c07](https://github.com/git-ents/git-metadata/commit/10d9c0749bdfa7abfe0c4f14af528e4c91a43e51))
* Add initial trait method and accompanying implementation ([c4ed9f8](https://github.com/git-ents/git-metadata/commit/c4ed9f85753443685fbaa63b13ffb1501f4d92d1))
* Add man generation to CLI ([275207f](https://github.com/git-ents/git-metadata/commit/275207f1b6c7b7aad9bdda1ccfb3b18f2d0fc2ac))
* Add metadata trait method definitions ([30ef2eb](https://github.com/git-ents/git-metadata/commit/30ef2eb67582dcceae1a88f6819802370dd00947))
* Add metadata trait method implementations ([30ef2eb](https://github.com/git-ents/git-metadata/commit/30ef2eb67582dcceae1a88f6819802370dd00947))
* Auto-detect fanout depth on read and remove ([bd86065](https://github.com/git-ents/git-metadata/commit/bd860650b8471215ad5f131f28666d613648178c))
* Glob matching for remove patterns (*, **, prefix match) ([4cf0d90](https://github.com/git-ents/git-metadata/commit/4cf0d90c56bed7188db4511eb0132d475286435c))
* Path-based metadata API following git-notes semantics ([4cf0d90](https://github.com/git-ents/git-metadata/commit/4cf0d90c56bed7188db4511eb0132d475286435c))
* Resolve revisions (HEAD, refs, short OIDs) in all object arguments ([4cf0d90](https://github.com/git-ents/git-metadata/commit/4cf0d90c56bed7188db4511eb0132d475286435c))

### Bug Fixes

* Auto-detect fanout depth and correct shard level semantics ([bd86065](https://github.com/git-ents/git-metadata/commit/bd860650b8471215ad5f131f28666d613648178c))
* Correct release-please manifest package key ([88f4ec1](https://github.com/git-ents/git-metadata/commit/88f4ec128b8c7a6544a612ae5a673e9462be7b8b))
* Remove cargo-workspace plugin for single-crate repo ([1d19a7d](https://github.com/git-ents/git-metadata/commit/1d19a7de740e6d637f7bc7751604d3deab244643))
* Shard-level default changed from 2 to 1 (git-notes compatible) ([bd86065](https://github.com/git-ents/git-metadata/commit/bd860650b8471215ad5f131f28666d613648178c))
