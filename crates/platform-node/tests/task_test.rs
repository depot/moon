use moon_lang_node::package::PackageJson;
use moon_platform_node::task::{create_task, should_run_in_ci, TaskContext};
use moon_platform_node::{create_tasks_from_scripts, infer_tasks_from_scripts};
use moon_task::{PlatformType, Task, TaskOptions};
use moon_utils::string_vec;
use std::collections::{BTreeMap, HashMap};

mod should_run_in_ci {
    use super::*;

    #[test]
    fn handles_reserved_words() {
        assert!(!should_run_in_ci("dev", ""));
        assert!(!should_run_in_ci("serve", ""));
        assert!(!should_run_in_ci("start", ""));

        assert!(should_run_in_ci("dev:app", ""));
        assert!(!should_run_in_ci("serve:app", ""));
        assert!(!should_run_in_ci("start:app", ""));

        assert!(should_run_in_ci("app:dev", ""));
        assert!(!should_run_in_ci("app:serve", ""));
        assert!(!should_run_in_ci("app:start", ""));
    }

    #[test]
    fn handles_watch_mode() {
        assert!(!should_run_in_ci("name", "packemon build --watch"));
        assert!(!should_run_in_ci("name", "rollup --watch"));
        assert!(!should_run_in_ci("name", "tsc --watch"));
    }

    #[test]
    fn handles_gatsby() {
        // yes
        assert!(should_run_in_ci("name", "gatsby --version"));
        assert!(should_run_in_ci("name", "gatsby --help"));
        assert!(should_run_in_ci("name", "gatsby build"));
        assert!(should_run_in_ci("name", "gatsby info"));
        assert!(should_run_in_ci("name", "npx gatsby build"));

        // no
        assert!(!should_run_in_ci("name", "gatsby dev"));
        assert!(!should_run_in_ci("name", "gatsby develop"));
        assert!(!should_run_in_ci("name", "gatsby new"));
        assert!(!should_run_in_ci("name", "gatsby serve"));
        assert!(!should_run_in_ci("name", "gatsby repl"));
    }

    #[test]
    fn handles_nextjs() {
        // yes
        assert!(should_run_in_ci("name", "next --version"));
        assert!(should_run_in_ci("name", "next --help"));
        assert!(should_run_in_ci("name", "next build"));
        assert!(should_run_in_ci("name", "next export"));
        assert!(should_run_in_ci("name", "npx next build"));

        // no
        assert!(!should_run_in_ci("name", "next dev"));
        assert!(!should_run_in_ci("name", "next start"));
    }

    #[test]
    fn handles_parcel() {
        // yes
        assert!(should_run_in_ci("name", "parcel --version"));
        assert!(should_run_in_ci("name", "parcel --help"));
        assert!(should_run_in_ci("name", "parcel build"));
        assert!(should_run_in_ci("name", "npx parcel build"));

        // no
        assert!(!should_run_in_ci("name", "parcel ./src/index.ts"));
        assert!(!should_run_in_ci("name", "parcel serve index.js"));
        assert!(!should_run_in_ci("name", "parcel watch"));
        assert!(!should_run_in_ci("name", "npx parcel"));
    }

    #[test]
    fn handles_react_scripts() {
        // yes
        assert!(should_run_in_ci("name", "react-scripts --version"));
        assert!(should_run_in_ci("name", "react-scripts --help"));
        assert!(should_run_in_ci("name", "react-scripts build"));
        assert!(should_run_in_ci("name", "react-scripts eject"));
        assert!(should_run_in_ci("name", "npx react-scripts build"));

        // no
        assert!(!should_run_in_ci("name", "react-scripts start"));
        assert!(!should_run_in_ci("name", "react-scripts test --watch"));
    }

    #[test]
    fn handles_snowpack() {
        // yes
        assert!(should_run_in_ci("name", "snowpack --version"));
        assert!(should_run_in_ci("name", "snowpack --help"));
        assert!(should_run_in_ci("name", "snowpack build"));
        assert!(should_run_in_ci("name", "npx snowpack build"));

        // no
        assert!(!should_run_in_ci("name", "snowpack dev"));
    }

    #[test]
    fn handles_vite() {
        // yes
        assert!(should_run_in_ci("name", "vite --version"));
        assert!(should_run_in_ci("name", "vite --help"));
        assert!(should_run_in_ci("name", "vite build"));
        assert!(should_run_in_ci("name", "vite optimize"));
        assert!(should_run_in_ci("name", "npx vite build"));

        // no
        assert!(!should_run_in_ci("name", "vite --watch"));
        assert!(!should_run_in_ci("name", "vite"));
        assert!(!should_run_in_ci("name", "vite dev"));
        assert!(!should_run_in_ci("name", "vite serve"));
        assert!(!should_run_in_ci("name", "vite preview"));
        assert!(!should_run_in_ci("name", "npx vite"));
        assert!(!should_run_in_ci("name", "npx vite dev"));
    }

    #[test]
    fn handles_webpack() {
        // yes
        assert!(should_run_in_ci("name", "webpack --version"));
        assert!(should_run_in_ci("name", "webpack --help"));
        assert!(should_run_in_ci("name", "webpack build"));
        assert!(should_run_in_ci("name", "webpack bundle"));
        assert!(should_run_in_ci("name", "webpack info"));
        assert!(should_run_in_ci("name", "npx webpack build"));

        // no
        assert!(!should_run_in_ci("name", "webpack --entry"));
        assert!(!should_run_in_ci("name", "webpack --watch"));
        assert!(!should_run_in_ci("name", "webpack"));
        assert!(!should_run_in_ci("name", "webpack s"));
        assert!(!should_run_in_ci("name", "webpack serve"));
        assert!(!should_run_in_ci("name", "webpack server"));
        assert!(!should_run_in_ci("name", "webpack w"));
        assert!(!should_run_in_ci("name", "webpack watch"));
        assert!(!should_run_in_ci("name", "npx webpack serve"));
    }
}

mod create_task {
    use super::*;

    mod script_files {
        use super::*;

        #[test]
        fn handles_bash() {
            let task = create_task(
                "project:task",
                "script",
                "bash scripts/setup.sh",
                TaskContext::ConvertToTask,
            )
            .unwrap();

            assert_eq!(
                task,
                Task {
                    command: "bash".to_owned(),
                    args: string_vec!["scripts/setup.sh"],
                    platform: PlatformType::System,
                    ..Task::new("project:task")
                }
            )
        }

        #[test]
        fn handles_bash_without_command() {
            let task = create_task(
                "project:task",
                "script",
                "scripts/setup.sh",
                TaskContext::ConvertToTask,
            )
            .unwrap();

            assert_eq!(
                task,
                Task {
                    command: "bash".to_owned(),
                    args: string_vec!["scripts/setup.sh"],
                    platform: PlatformType::System,
                    ..Task::new("project:task")
                }
            )
        }

        #[test]
        fn handles_node() {
            let task = create_task(
                "project:task",
                "script",
                "node scripts/test.js",
                TaskContext::ConvertToTask,
            )
            .unwrap();

            assert_eq!(
                task,
                Task {
                    command: "node".to_owned(),
                    args: string_vec!["scripts/test.js"],
                    platform: PlatformType::Node,
                    ..Task::new("project:task")
                }
            )
        }

        #[test]
        fn handles_node_without_command() {
            let candidates = ["scripts/test.js", "scripts/test.cjs", "scripts/test.mjs"];

            for candidate in candidates {
                let task = create_task(
                    "project:task",
                    "script",
                    candidate,
                    TaskContext::ConvertToTask,
                )
                .unwrap();

                assert_eq!(
                    task,
                    Task {
                        command: "node".to_owned(),
                        args: string_vec![candidate],
                        platform: PlatformType::Node,
                        ..Task::new("project:task")
                    }
                )
            }
        }
    }

    mod env_vars {
        use super::*;

        #[test]
        fn extracts_single_var() {
            let task = create_task(
                "project:task",
                "script",
                "KEY=VALUE yarn install",
                TaskContext::ConvertToTask,
            )
            .unwrap();

            assert_eq!(
                task,
                Task {
                    command: "yarn".to_owned(),
                    args: string_vec!["install"],
                    env: HashMap::from([("KEY".to_owned(), "VALUE".to_owned())]),
                    platform: PlatformType::Node,
                    ..Task::new("project:task")
                }
            )
        }

        #[test]
        fn extracts_multiple_vars() {
            let task = create_task(
                "project:task",
                "script",
                "KEY1=VAL1 KEY2=VAL2 yarn install",
                TaskContext::ConvertToTask,
            )
            .unwrap();

            assert_eq!(
                task,
                Task {
                    command: "yarn".to_owned(),
                    args: string_vec!["install"],
                    env: HashMap::from([
                        ("KEY1".to_owned(), "VAL1".to_owned()),
                        ("KEY2".to_owned(), "VAL2".to_owned())
                    ]),
                    platform: PlatformType::Node,
                    ..Task::new("project:task")
                }
            )
        }

        #[test]
        fn handles_semicolons() {
            let task = create_task(
                "project:task",
                "script",
                "KEY1=VAL1; KEY2=VAL2; yarn install",
                TaskContext::ConvertToTask,
            )
            .unwrap();

            assert_eq!(
                task,
                Task {
                    command: "yarn".to_owned(),
                    args: string_vec!["install"],
                    env: HashMap::from([
                        ("KEY1".to_owned(), "VAL1".to_owned()),
                        ("KEY2".to_owned(), "VAL2".to_owned())
                    ]),
                    platform: PlatformType::Node,
                    ..Task::new("project:task")
                }
            )
        }

        #[test]
        fn handles_quoted_values() {
            let task = create_task(
                "project:task",
                "script",
                "NODE_OPTIONS='-f -b' yarn",
                TaskContext::ConvertToTask,
            )
            .unwrap();

            assert_eq!(
                task,
                Task {
                    command: "yarn".to_owned(),
                    env: HashMap::from([("NODE_OPTIONS".to_owned(), "-f -b".to_owned())]),
                    platform: PlatformType::Node,
                    ..Task::new("project:task")
                }
            )
        }
    }

    mod outputs {
        use super::*;

        #[test]
        fn detects_outputs_from_args() {
            let candidates = vec![
                ("-o", "dir", "dir"),
                ("-o", "./file.js", "file.js"),
                ("--out", "./lib", "lib"),
                ("--out-dir", "build", "build"),
                ("--out-file", "./build/min.js", "build/min.js"),
                ("--outdir", "build", "build"),
                ("--outfile", "./build/min.js", "build/min.js"),
                ("--outDir", "build", "build"),
                ("--outFile", "./build/min.js", "build/min.js"),
                ("--dist", "dist", "dist"),
                ("--dist-dir", "./dist", "dist"),
                ("--dist-file", "./dist/bundle.js", "dist/bundle.js"),
                ("--distDir", "dist", "dist"),
                ("--distFile", "dist/bundle.js", "dist/bundle.js"),
            ];

            for candidate in candidates {
                let task = create_task(
                    "project:task",
                    "script",
                    &format!("tool build {} {}", candidate.0, candidate.1),
                    TaskContext::ConvertToTask,
                )
                .unwrap();

                assert_eq!(
                    task,
                    Task {
                        command: "tool".to_owned(),
                        args: string_vec!["build", candidate.0, candidate.1],
                        outputs: string_vec![candidate.2],
                        platform: PlatformType::Node,
                        ..Task::new("project:task")
                    }
                )
            }
        }

        #[should_panic(expected = "NoParentOutput(\"../parent/dir\", \"project:task\")")]
        #[test]
        fn fails_on_parent_relative() {
            create_task(
                "project:task",
                "script",
                "build --out ../parent/dir",
                TaskContext::ConvertToTask,
            )
            .unwrap();
        }

        #[should_panic(expected = "NoAbsoluteOutput(\"/abs/dir\", \"project:task\")")]
        #[test]
        fn fails_on_absolute() {
            create_task(
                "project:task",
                "script",
                "build --out /abs/dir",
                TaskContext::ConvertToTask,
            )
            .unwrap();
        }

        #[should_panic(expected = "NoAbsoluteOutput(\"C:\\\\abs\\\\dir\", \"project:task\")")]
        #[test]
        fn fails_on_absolute_windows() {
            create_task(
                "project:task",
                "script",
                "build --out C:\\\\abs\\\\dir",
                TaskContext::ConvertToTask,
            )
            .unwrap();
        }
    }
}

mod infer_tasks_from_scripts {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn wraps_scripts() {
        let pkg = PackageJson {
            scripts: Some(BTreeMap::from([
                ("postinstall".into(), "./setup.sh".into()),
                ("build:app".into(), "webpack build --output ./dist".into()),
                ("dev".into(), "webpack dev".into()),
                ("test".into(), "jest .".into()),
                ("posttest".into(), "run-coverage".into()),
                ("lint".into(), "eslint src/**/* .".into()),
                ("typecheck".into(), "tsc --build".into()),
            ])),
            ..PackageJson::default()
        };

        let tasks = infer_tasks_from_scripts("project", &pkg).unwrap();

        assert_eq!(
            tasks,
            BTreeMap::from([
                (
                    "build-app".to_owned(),
                    Task {
                        command: "moon".to_owned(),
                        args: string_vec!["node", "run-script", "build:app"],
                        outputs: string_vec!["dist"],
                        platform: PlatformType::Node,
                        ..Task::new("project:build-app")
                    }
                ),
                (
                    "dev".to_owned(),
                    Task {
                        command: "moon".to_owned(),
                        args: string_vec!["node", "run-script", "dev"],
                        options: TaskOptions {
                            run_in_ci: false,
                            ..TaskOptions::default()
                        },
                        platform: PlatformType::Node,
                        ..Task::new("project:dev")
                    }
                ),
                (
                    "test".to_owned(),
                    Task {
                        command: "moon".to_owned(),
                        args: string_vec!["node", "run-script", "test"],
                        platform: PlatformType::Node,
                        ..Task::new("project:test")
                    }
                ),
                (
                    "lint".to_owned(),
                    Task {
                        command: "moon".to_owned(),
                        args: string_vec!["node", "run-script", "lint"],
                        platform: PlatformType::Node,
                        ..Task::new("project:lint")
                    }
                ),
                (
                    "typecheck".to_owned(),
                    Task {
                        command: "moon".to_owned(),
                        args: string_vec!["node", "run-script", "typecheck"],
                        platform: PlatformType::Node,
                        ..Task::new("project:typecheck")
                    }
                ),
            ])
        )
    }
}

mod create_tasks_from_scripts {
    use super::*;

    #[test]
    fn ignores_unsupported_syntax() {
        let mut pkg = PackageJson {
            scripts: Some(BTreeMap::from([
                ("cd".into(), "cd website && yarn build".into()),
                ("out".into(), "some-bin > output.log".into()),
                ("in".into(), "output.log < some-bin".into()),
                ("pipe".into(), "ls | grep foo".into()),
                ("or".into(), "foo || bar".into()),
                ("semi".into(), "foo ;; bar".into()),
            ])),
            ..PackageJson::default()
        };

        let tasks = create_tasks_from_scripts("project", &mut pkg).unwrap();

        assert!(tasks.is_empty());
    }

    #[test]
    fn renames_to_ids() {
        let mut pkg = PackageJson {
            scripts: Some(BTreeMap::from([
                ("base".into(), "script".into()),
                ("foo-bar".into(), "script".into()),
                ("foo_bar".into(), "script".into()),
                ("foo:bar".into(), "script".into()),
                ("foo-bar:baz".into(), "script".into()),
                ("foo_bar:baz".into(), "script".into()),
                ("foo:bar:baz".into(), "script".into()),
                ("foo_bar:baz-qux".into(), "script".into()),
                ("fooBar".into(), "script".into()),
            ])),
            ..PackageJson::default()
        };

        let tasks = create_tasks_from_scripts("project", &mut pkg).unwrap();

        assert_eq!(
            tasks.keys().cloned().collect::<Vec<String>>(),
            string_vec![
                "base",
                "foo-bar",
                "foo-bar-baz",
                "fooBar",
                "foo_bar",
                "foo_bar-baz",
                "foo_bar-baz-qux",
            ]
        );
    }

    #[test]
    fn converts_stand_alone() {
        let mut pkg = PackageJson {
            scripts: Some(BTreeMap::from([
                ("test".into(), "jest .".into()),
                ("lint".into(), "eslint src/**/* .".into()),
                ("typecheck".into(), "tsc --build".into()),
            ])),
            ..PackageJson::default()
        };

        let tasks = create_tasks_from_scripts("project", &mut pkg).unwrap();

        assert_eq!(pkg.scripts, None);

        assert_eq!(
            tasks,
            BTreeMap::from([
                (
                    "test".to_owned(),
                    Task {
                        command: "jest".to_owned(),
                        args: string_vec!["."],
                        platform: PlatformType::Node,
                        ..Task::new("project:test")
                    }
                ),
                (
                    "lint".to_owned(),
                    Task {
                        command: "eslint".to_owned(),
                        args: string_vec!["src/**/*", "."],
                        platform: PlatformType::Node,
                        ..Task::new("project:lint")
                    }
                ),
                (
                    "typecheck".to_owned(),
                    Task {
                        command: "tsc".to_owned(),
                        args: string_vec!["--build"],
                        platform: PlatformType::Node,
                        ..Task::new("project:typecheck")
                    }
                ),
            ])
        )
    }

    mod pre_post {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn creates_pre_and_post() {
            let mut pkg = PackageJson {
                scripts: Some(BTreeMap::from([
                    ("test".into(), "jest .".into()),
                    ("pretest".into(), "do something".into()),
                    ("posttest".into(), "do another".into()),
                ])),
                ..PackageJson::default()
            };

            let tasks = create_tasks_from_scripts("project", &mut pkg).unwrap();

            assert_eq!(pkg.scripts, None);

            assert_eq!(
                tasks,
                BTreeMap::from([
                    (
                        "pretest".to_owned(),
                        Task {
                            command: "do".to_owned(),
                            args: string_vec!["something"],
                            platform: PlatformType::Node,
                            ..Task::new("project:pretest")
                        }
                    ),
                    (
                        "posttest".to_owned(),
                        Task {
                            command: "do".to_owned(),
                            args: string_vec!["another"],
                            deps: string_vec!["~:test"],
                            platform: PlatformType::Node,
                            ..Task::new("project:posttest")
                        }
                    ),
                    (
                        "test".to_owned(),
                        Task {
                            command: "jest".to_owned(),
                            args: string_vec!["."],
                            deps: string_vec!["~:pretest"],
                            platform: PlatformType::Node,
                            ..Task::new("project:test")
                        }
                    ),
                ])
            )
        }

        #[test]
        fn supports_multiple_pre_via_andand() {
            let mut pkg = PackageJson {
                scripts: Some(BTreeMap::from([
                    ("test".into(), "jest .".into()),
                    ("pretest".into(), "do something && do another".into()),
                ])),
                ..PackageJson::default()
            };

            let tasks = create_tasks_from_scripts("project", &mut pkg).unwrap();

            assert_eq!(pkg.scripts, None);

            assert_eq!(
                tasks,
                BTreeMap::from([
                    (
                        "pretest-dep1".to_owned(),
                        Task {
                            command: "do".to_owned(),
                            args: string_vec!["something"],
                            platform: PlatformType::Node,
                            ..Task::new("project:pretest-dep1")
                        }
                    ),
                    (
                        "pretest".to_owned(),
                        Task {
                            command: "do".to_owned(),
                            args: string_vec!["another"],
                            deps: string_vec!["~:pretest-dep1"],
                            platform: PlatformType::Node,
                            ..Task::new("project:pretest")
                        }
                    ),
                    (
                        "test".to_owned(),
                        Task {
                            command: "jest".to_owned(),
                            args: string_vec!["."],
                            deps: string_vec!["~:pretest"],
                            platform: PlatformType::Node,
                            ..Task::new("project:test")
                        }
                    )
                ])
            )
        }

        #[test]
        fn supports_multiple_post_via_andand() {
            let mut pkg = PackageJson {
                scripts: Some(BTreeMap::from([
                    ("test".into(), "jest .".into()),
                    ("posttest".into(), "do something && do another".into()),
                ])),
                ..PackageJson::default()
            };

            let tasks = create_tasks_from_scripts("project", &mut pkg).unwrap();

            assert_eq!(pkg.scripts, None);

            assert_eq!(
                tasks,
                BTreeMap::from([
                    (
                        "posttest-dep1".to_owned(),
                        Task {
                            command: "do".to_owned(),
                            args: string_vec!["something"],
                            platform: PlatformType::Node,
                            ..Task::new("project:posttest-dep1")
                        }
                    ),
                    (
                        "posttest".to_owned(),
                        Task {
                            command: "do".to_owned(),
                            args: string_vec!["another"],
                            deps: string_vec!["~:posttest-dep1", "~:test"],
                            platform: PlatformType::Node,
                            ..Task::new("project:posttest")
                        }
                    ),
                    (
                        "test".to_owned(),
                        Task {
                            command: "jest".to_owned(),
                            args: string_vec!["."],
                            platform: PlatformType::Node,
                            ..Task::new("project:test")
                        }
                    ),
                ])
            )
        }

        #[test]
        fn handles_pre_within_script() {
            let mut pkg = PackageJson {
                scripts: Some(BTreeMap::from([
                    ("release".into(), "npm run prerelease && npm publish".into()),
                    ("prerelease".into(), "webpack build".into()),
                ])),
                ..PackageJson::default()
            };

            let tasks = create_tasks_from_scripts("project", &mut pkg).unwrap();

            assert_eq!(pkg.scripts, None);

            assert_eq!(
                tasks,
                BTreeMap::from([
                    (
                        "prerelease".to_owned(),
                        Task {
                            command: "webpack".to_owned(),
                            args: string_vec!["build"],
                            platform: PlatformType::Node,
                            ..Task::new("project:prerelease")
                        }
                    ),
                    (
                        "release".to_owned(),
                        Task {
                            command: "npm".to_owned(),
                            args: string_vec!["publish"],
                            deps: string_vec!["~:prerelease"],
                            platform: PlatformType::Node,
                            ..Task::new("project:release")
                        }
                    ),
                ])
            )
        }
    }

    mod pm_run {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn skips_when_pointing_to_an_unknown() {
            let mut pkg = PackageJson {
                scripts: Some(BTreeMap::from([
                    ("lint".into(), "eslint .".into()),
                    ("lint:fix".into(), "npm run invalid -- --fix".into()),
                ])),
                ..PackageJson::default()
            };

            let tasks = create_tasks_from_scripts("project", &mut pkg).unwrap();

            assert_eq!(pkg.scripts, None);

            assert_eq!(
                tasks,
                BTreeMap::from([(
                    "lint".to_owned(),
                    Task {
                        command: "eslint".to_owned(),
                        args: string_vec!["."],
                        platform: PlatformType::Node,
                        ..Task::new("project:lint")
                    }
                )])
            )
        }

        #[test]
        fn converts_without_args() {
            let candidates = [
                "npm run lint",
                "npm run lint --",
                "pnpm run lint",
                "pnpm run lint --",
                "yarn run lint",
                "yarn run lint --",
            ];

            for candidate in candidates {
                let mut pkg = PackageJson {
                    scripts: Some(BTreeMap::from([
                        ("lint".into(), "eslint .".into()),
                        ("lint:fix".into(), candidate.to_owned()),
                    ])),
                    ..PackageJson::default()
                };

                let tasks = create_tasks_from_scripts("project", &mut pkg).unwrap();

                assert_eq!(pkg.scripts, None);

                assert_eq!(
                    tasks,
                    BTreeMap::from([
                        (
                            "lint".to_owned(),
                            Task {
                                command: "eslint".to_owned(),
                                args: string_vec!["."],
                                platform: PlatformType::Node,
                                ..Task::new("project:lint")
                            }
                        ),
                        (
                            "lint-fix".to_owned(),
                            Task {
                                command: "moon".to_owned(),
                                args: string_vec!["run", "project:lint"],
                                platform: PlatformType::Node,
                                ..Task::new("project:lint-fix")
                            }
                        ),
                    ])
                )
            }
        }

        #[test]
        fn converts_with_args() {
            let candidates = [
                "npm run lint -- --fix",
                "pnpm run lint -- --fix",
                "pnpm run lint --fix",
                "yarn run lint -- --fix",
                "yarn run lint --fix",
            ];

            for candidate in candidates {
                let mut pkg = PackageJson {
                    scripts: Some(BTreeMap::from([
                        ("lint:fix".into(), candidate.to_owned()),
                        ("lint".into(), "eslint .".into()),
                    ])),
                    ..PackageJson::default()
                };

                let tasks = create_tasks_from_scripts("project", &mut pkg).unwrap();

                assert_eq!(pkg.scripts, None);

                assert_eq!(
                    tasks,
                    BTreeMap::from([
                        (
                            "lint".to_owned(),
                            Task {
                                command: "eslint".to_owned(),
                                args: string_vec!["."],
                                platform: PlatformType::Node,
                                ..Task::new("project:lint")
                            }
                        ),
                        (
                            "lint-fix".to_owned(),
                            Task {
                                command: "moon".to_owned(),
                                args: string_vec!["run", "project:lint", "--", "--fix"],
                                platform: PlatformType::Node,
                                ..Task::new("project:lint-fix")
                            }
                        ),
                    ])
                )
            }
        }

        #[test]
        fn handles_env_vars() {
            let mut pkg = PackageJson {
                scripts: Some(BTreeMap::from([
                    ("build".into(), "webpack build".into()),
                    (
                        "build:dev".into(),
                        "NODE_ENV=development npm run build -- --stats".into(),
                    ),
                    (
                        "build:prod".into(),
                        "NODE_ENV=production yarn run build".into(),
                    ),
                    (
                        "build:staging".into(),
                        "NODE_ENV=staging pnpm run build --mode production".into(),
                    ),
                ])),
                ..PackageJson::default()
            };

            let tasks = create_tasks_from_scripts("project", &mut pkg).unwrap();

            assert_eq!(pkg.scripts, None);

            assert_eq!(
                tasks,
                BTreeMap::from([
                    (
                        "build".to_owned(),
                        Task {
                            command: "webpack".to_owned(),
                            args: string_vec!["build"],
                            platform: PlatformType::Node,
                            ..Task::new("project:build")
                        }
                    ),
                    (
                        "build-dev".to_owned(),
                        Task {
                            command: "moon".to_owned(),
                            args: string_vec!["run", "project:build", "--", "--stats"],
                            env: HashMap::from([("NODE_ENV".to_owned(), "development".to_owned())]),
                            platform: PlatformType::Node,
                            ..Task::new("project:build-dev")
                        }
                    ),
                    (
                        "build-prod".to_owned(),
                        Task {
                            command: "moon".to_owned(),
                            args: string_vec!["run", "project:build"],
                            env: HashMap::from([("NODE_ENV".to_owned(), "production".to_owned())]),
                            platform: PlatformType::Node,
                            ..Task::new("project:build-prod")
                        }
                    ),
                    (
                        "build-staging".to_owned(),
                        Task {
                            command: "moon".to_owned(),
                            args: string_vec!["run", "project:build", "--", "--mode", "production"],
                            env: HashMap::from([("NODE_ENV".to_owned(), "staging".to_owned())]),
                            platform: PlatformType::Node,
                            ..Task::new("project:build-staging")
                        }
                    ),
                ])
            )
        }
    }

    mod life_cycle {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn rewrites_run_commands() {
            let mut pkg = PackageJson {
                scripts: Some(BTreeMap::from([
                    ("build".into(), "babel .".into()),
                    ("lint".into(), "eslint .".into()),
                    ("test".into(), "jest .".into()),
                    ("preversion".into(), "npm run lint && npm run test".into()),
                    ("version".into(), "npm run build".into()),
                    (
                        "postversion".into(),
                        "npm ci && git add package-lock.json".into(),
                    ),
                ])),
                ..PackageJson::default()
            };

            let tasks = create_tasks_from_scripts("project", &mut pkg).unwrap();

            assert_eq!(
                pkg.scripts,
                Some(BTreeMap::from([
                    (
                        "preversion".to_owned(),
                        "moon run project:lint && moon run project:test".to_owned()
                    ),
                    ("version".to_owned(), "moon run project:build".to_owned()),
                    (
                        "postversion".to_owned(),
                        "npm ci && git add package-lock.json".to_owned()
                    ),
                ]))
            );

            assert_eq!(
                tasks,
                BTreeMap::from([
                    (
                        "build".to_owned(),
                        Task {
                            command: "babel".to_owned(),
                            args: string_vec!["."],
                            platform: PlatformType::Node,
                            ..Task::new("project:build")
                        }
                    ),
                    (
                        "lint".to_owned(),
                        Task {
                            command: "eslint".to_owned(),
                            args: string_vec!["."],
                            platform: PlatformType::Node,
                            ..Task::new("project:lint")
                        }
                    ),
                    (
                        "test".to_owned(),
                        Task {
                            command: "jest".to_owned(),
                            args: string_vec!["."],
                            platform: PlatformType::Node,
                            ..Task::new("project:test")
                        }
                    )
                ])
            )
        }
    }
}

mod complex_examples {
    use super::*;
    use pretty_assertions::assert_eq;

    // https://github.com/babel/babel/blob/main/package.json
    #[test]
    fn babel() {
        let mut pkg = PackageJson {
            scripts: Some(BTreeMap::from([
                ("postinstall".into(), "husky install".into()),
                ("bootstrap".into(), "make bootstrap".into()),
                ("codesandbox:build".into(), "make build-no-bundle".into()),
                ("build".into(), "make build".into()),
                ("fix".into(), "make fix".into()),
                ("lint".into(), "make lint".into()),
                ("test".into(), "make test".into()),
                (
                    "version".into(),
                    "yarn --immutable-cache && git add yarn.lock".into(),
                ),
                ("test:esm".into(), "node test/esm/index.js".into()),
                (
                    "test:runtime:generate-absolute-runtime".into(),
                    "node test/runtime-integration/generate-absolute-runtime.cjs".into(),
                ),
                (
                    "test:runtime:bundlers".into(),
                    "node test/runtime-integration/bundlers.cjs".into(),
                ),
                (
                    "test:runtime:node".into(),
                    "node test/runtime-integration/node.cjs".into(),
                ),
            ])),
            ..PackageJson::default()
        };

        let tasks = create_tasks_from_scripts("project", &mut pkg).unwrap();

        assert_eq!(
            pkg.scripts,
            Some(BTreeMap::from([
                ("postinstall".to_owned(), "husky install".to_owned()),
                (
                    "version".to_owned(),
                    "yarn --immutable-cache && git add yarn.lock".to_owned()
                )
            ]))
        );

        assert_eq!(
            tasks,
            BTreeMap::from([
                (
                    "bootstrap".to_owned(),
                    Task {
                        command: "make".to_owned(),
                        args: string_vec!["bootstrap"],
                        platform: PlatformType::System,
                        ..Task::new("project:bootstrap")
                    }
                ),
                (
                    "build".to_owned(),
                    Task {
                        command: "make".to_owned(),
                        args: string_vec!["build"],
                        platform: PlatformType::System,
                        ..Task::new("project:build")
                    }
                ),
                (
                    "codesandbox-build".to_owned(),
                    Task {
                        command: "make".to_owned(),
                        args: string_vec!["build-no-bundle"],
                        platform: PlatformType::System,
                        ..Task::new("project:codesandbox-build")
                    }
                ),
                (
                    "fix".to_owned(),
                    Task {
                        command: "make".to_owned(),
                        args: string_vec!["fix"],
                        platform: PlatformType::System,
                        ..Task::new("project:fix")
                    }
                ),
                (
                    "lint".to_owned(),
                    Task {
                        command: "make".to_owned(),
                        args: string_vec!["lint"],
                        platform: PlatformType::System,
                        ..Task::new("project:lint")
                    }
                ),
                (
                    "test".to_owned(),
                    Task {
                        command: "make".to_owned(),
                        args: string_vec!["test"],
                        platform: PlatformType::System,
                        ..Task::new("project:test")
                    }
                ),
                (
                    "test-esm".to_owned(),
                    Task {
                        command: "node".to_owned(),
                        args: string_vec!["test/esm/index.js"],
                        platform: PlatformType::Node,
                        ..Task::new("project:test-esm")
                    }
                ),
                (
                    "test-runtime-bundlers".to_owned(),
                    Task {
                        command: "node".to_owned(),
                        args: string_vec!["test/runtime-integration/bundlers.cjs"],
                        platform: PlatformType::Node,
                        ..Task::new("project:test-runtime-bundlers")
                    }
                ),
                (
                    "test-runtime-generate-absolute-runtime".to_owned(),
                    Task {
                        command: "node".to_owned(),
                        args: string_vec!["test/runtime-integration/generate-absolute-runtime.cjs"],
                        platform: PlatformType::Node,
                        ..Task::new("project:test-runtime-generate-absolute-runtime")
                    }
                ),
                (
                    "test-runtime-node".to_owned(),
                    Task {
                        command: "node".to_owned(),
                        args: string_vec!["test/runtime-integration/node.cjs"],
                        platform: PlatformType::Node,
                        ..Task::new("project:test-runtime-node")
                    }
                ),
            ])
        );
    }

    // https://github.com/milesj/packemon/blob/master/package.json
    #[test]
    fn packemon() {
        let mut pkg = PackageJson {
            scripts: Some(BTreeMap::from([
                ("build".into(), "yarn run packemon build".into()),
                ("check".into(), "yarn run type && yarn run test && yarn run lint".into()),
                ("clean".into(), "yarn run packemon clean".into()),
                ("commit".into(), "yarn install && git add yarn.lock".into()),
                ("coverage".into(), "yarn run test --coverage".into()),
                ("create-config".into(), "beemo create-config".into()),
                ("docs".into(), "cd website && yarn run start".into()),
                ("format".into(), "beemo prettier".into()),
                ("lint".into(), "beemo eslint".into()),
                ("packup".into(), "NODE_ENV=production yarn run packemon build --addEngines --addExports --declaration".into()),
                ("packemon".into(), "node ./packages/packemon/cjs/bin.cjs".into()),
                ("prerelease".into(), "yarn run clean && yarn run setup && yarn run packup && yarn run check".into()),
                ("release".into(), "yarn run prerelease && beemo run-script lerna-release".into()),
                ("setup".into(), "yarn dlx --package packemon@latest --package typescript --quiet packemon build".into()),
                ("test".into(), "beemo jest".into()),
                ("type".into(), "beemo typescript --build".into()),
                ("validate".into(), "yarn run packemon validate".into()),
            ])),
            ..PackageJson::default()
        };

        let tasks = create_tasks_from_scripts("project", &mut pkg).unwrap();

        assert_eq!(pkg.scripts, None);

        assert_eq!(
            tasks,
            BTreeMap::from([
                (
                    "build".to_owned(),
                    Task {
                        command: "moon".to_owned(),
                        args: string_vec!["run", "project:packemon", "--", "build"],
                        platform: PlatformType::Node,
                        ..Task::new("project:build")
                    }
                ),
                (
                    "check-dep1".to_owned(),
                    Task {
                        command: "moon".to_owned(),
                        args: string_vec!["run", "project:type"],
                        platform: PlatformType::Node,
                        ..Task::new("project:check-dep1")
                    }
                ),
                (
                    "check-dep2".to_owned(),
                    Task {
                        command: "moon".to_owned(),
                        args: string_vec!["run", "project:test"],
                        deps: string_vec!["~:check-dep1"],
                        platform: PlatformType::Node,
                        ..Task::new("project:check-dep2")
                    }
                ),
                (
                    "check".to_owned(),
                    Task {
                        command: "moon".to_owned(),
                        args: string_vec!["run", "project:lint"],
                        deps: string_vec!["~:check-dep2"],
                        platform: PlatformType::Node,
                        ..Task::new("project:check")
                    }
                ),
                (
                    "clean".to_owned(),
                    Task {
                        command: "moon".to_owned(),
                        args: string_vec!["run", "project:packemon", "--", "clean"],
                        platform: PlatformType::Node,
                        ..Task::new("project:clean")
                    }
                ),
                (
                    "commit-dep1".to_owned(),
                    Task {
                        command: "yarn".to_owned(),
                        args: string_vec!["install"],
                        platform: PlatformType::Node,
                        ..Task::new("project:commit-dep1")
                    }
                ),
                (
                    "commit".to_owned(),
                    Task {
                        command: "git".to_owned(),
                        args: string_vec!["add", "yarn.lock"],
                        deps: string_vec!["~:commit-dep1"],
                        platform: PlatformType::System,
                        ..Task::new("project:commit")
                    }
                ),
                (
                    "coverage".to_owned(),
                    Task {
                        command: "moon".to_owned(),
                        args: string_vec!["run", "project:test", "--", "--coverage"],
                        platform: PlatformType::Node,
                        ..Task::new("project:coverage")
                    }
                ),
                (
                    "create-config".to_owned(),
                    Task {
                        command: "beemo".to_owned(),
                        args: string_vec!["create-config"],
                        platform: PlatformType::Node,
                        ..Task::new("project:create-config")
                    }
                ),
                (
                    "format".to_owned(),
                    Task {
                        command: "beemo".to_owned(),
                        args: string_vec!["prettier"],
                        platform: PlatformType::Node,
                        ..Task::new("project:format")
                    }
                ),
                (
                    "lint".to_owned(),
                    Task {
                        command: "beemo".to_owned(),
                        args: string_vec!["eslint"],
                        platform: PlatformType::Node,
                        ..Task::new("project:lint")
                    }
                ),
                (
                    "packup".to_owned(),
                    Task {
                        command: "moon".to_owned(),
                        args: string_vec![
                            "run",
                            "project:packemon",
                            "--",
                            "build",
                            "--addEngines",
                            "--addExports",
                            "--declaration"
                        ],
                        env: HashMap::from([("NODE_ENV".to_owned(), "production".to_owned())]),
                        platform: PlatformType::Node,
                        ..Task::new("project:packup")
                    }
                ),
                (
                    "packemon".to_owned(),
                    Task {
                        command: "node".to_owned(),
                        args: string_vec!["./packages/packemon/cjs/bin.cjs"],
                        platform: PlatformType::Node,
                        ..Task::new("project:packemon")
                    }
                ),
                (
                    "prerelease-dep1".to_owned(),
                    Task {
                        command: "moon".to_owned(),
                        args: string_vec!["run", "project:clean"],
                        platform: PlatformType::Node,
                        ..Task::new("project:prerelease-dep1")
                    }
                ),
                (
                    "prerelease-dep2".to_owned(),
                    Task {
                        command: "moon".to_owned(),
                        args: string_vec!["run", "project:setup"],
                        deps: string_vec!["~:prerelease-dep1"],
                        platform: PlatformType::Node,
                        ..Task::new("project:prerelease-dep2")
                    }
                ),
                (
                    "prerelease-dep3".to_owned(),
                    Task {
                        command: "moon".to_owned(),
                        args: string_vec!["run", "project:packup"],
                        deps: string_vec!["~:prerelease-dep2"],
                        platform: PlatformType::Node,
                        ..Task::new("project:prerelease-dep3")
                    }
                ),
                (
                    "prerelease".to_owned(),
                    Task {
                        command: "moon".to_owned(),
                        args: string_vec!["run", "project:check"],
                        deps: string_vec!["~:prerelease-dep3"],
                        platform: PlatformType::Node,
                        ..Task::new("project:prerelease")
                    }
                ),
                (
                    "release".to_owned(),
                    Task {
                        command: "beemo".to_owned(),
                        args: string_vec!["run-script", "lerna-release"],
                        deps: string_vec!["~:prerelease"],
                        platform: PlatformType::Node,
                        ..Task::new("project:release")
                    }
                ),
                (
                    "setup".to_owned(),
                    Task {
                        command: "yarn".to_owned(),
                        args: string_vec![
                            "dlx",
                            "--package",
                            "packemon@latest",
                            "--package",
                            "typescript",
                            "--quiet",
                            "packemon",
                            "build"
                        ],
                        platform: PlatformType::Node,
                        ..Task::new("project:setup")
                    }
                ),
                (
                    "test".to_owned(),
                    Task {
                        command: "beemo".to_owned(),
                        args: string_vec!["jest"],
                        platform: PlatformType::Node,
                        ..Task::new("project:test")
                    }
                ),
                (
                    "type".to_owned(),
                    Task {
                        command: "beemo".to_owned(),
                        args: string_vec!["typescript", "--build"],
                        platform: PlatformType::Node,
                        ..Task::new("project:type")
                    }
                ),
                (
                    "validate".to_owned(),
                    Task {
                        command: "moon".to_owned(),
                        args: string_vec!["run", "project:packemon", "--", "validate"],
                        platform: PlatformType::Node,
                        ..Task::new("project:validate")
                    }
                ),
            ])
        )
    }

    // https://github.com/prettier/prettier/blob/main/package.json
    #[test]
    fn prettier() {
        let mut pkg = PackageJson {
            scripts: Some(BTreeMap::from([
                ("prepublishOnly".into(), "echo \"Error: must publish from dist/\" && exit 1".into()),
                ("test".into(), "jest".into()),
                ("test:dev-package".into(), "cross-env INSTALL_PACKAGE=1 jest".into()),
                ("test:dist".into(), "cross-env NODE_ENV=production jest".into()),
                ("test:dist-standalone".into(), "cross-env NODE_ENV=production TEST_STANDALONE=1 jest".into()),
                ("test:integration".into(), "jest tests/integration".into()),
                ("test:dist-lint".into(), "eslint --no-eslintrc --no-ignore --no-inline-config --config=./scripts/bundle-eslint-config.cjs \"dist/**/*.{js,mjs}\"".into()),
                ("perf".into(), "yarn run build && cross-env NODE_ENV=production node ./dist/bin-prettier.js".into()),
                ("perf:inspect".into(), "yarn run build && cross-env NODE_ENV=production node --inspect-brk ./dist/bin-prettier.js".into()),
                ("perf:benchmark".into(), "yarn run perf --debug-benchmark".into()),
                ("lint".into(), "run-p lint:*".into()),
                ("lint:typecheck".into(), "tsc".into()),
                ("lint:eslint".into(), "cross-env EFF_NO_LINK_RULES=true eslint . --format friendly".into()),
                ("lint:changelog".into(), "node ./scripts/lint-changelog.mjs".into()),
                ("lint:prettier".into(), "prettier . \"!test*\" --check".into()),
                ("lint:spellcheck".into(), "cspell --no-progress --relative --dot --gitignore".into()),
                ("lint:deps".into(), "node ./scripts/check-deps.mjs".into()),
                ("lint:actionlint".into(), "node-actionlint".into()),
                ("fix:eslint".into(), "yarn run lint:eslint --fix".into()),
                ("fix:prettier".into(), "yarn run lint:prettier --write".into()),
                ("build".into(), "node ./scripts/build/build.mjs".into()),
                ("build:website".into(), "node ./scripts/build-website.mjs".into()),
                ("vendors:bundle".into(), "node ./scripts/vendors/bundle-vendors.mjs".into()),
            ])),
            ..PackageJson::default()
        };

        let tasks = create_tasks_from_scripts("project", &mut pkg).unwrap();

        assert_eq!(
            pkg.scripts,
            Some(BTreeMap::from([(
                "prepublishOnly".to_owned(),
                "echo \"Error: must publish from dist/\" && exit 1".to_owned()
            )]))
        );

        assert_eq!(
            tasks,
            BTreeMap::from([
                (
                    "lint".to_owned(),
                    Task {
                        command: "run-p".to_owned(),
                        args: string_vec!["lint:*"],
                        platform: PlatformType::Node,
                        ..Task::new("project:lint")
                    }
                ),
                (
                    "lint-actionlint".to_owned(),
                    Task {
                        command: "node-actionlint".to_owned(),
                        platform: PlatformType::Node,
                        ..Task::new("project:lint-actionlint")
                    }
                ),
                (
                    "lint-changelog".to_owned(),
                    Task {
                        command: "node".to_owned(),
                        args: string_vec!["./scripts/lint-changelog.mjs"],
                        platform: PlatformType::Node,
                        ..Task::new("project:lint-changelog")
                    }
                ),
                (
                    "lint-deps".to_owned(),
                    Task {
                        command: "node".to_owned(),
                        args: string_vec!["./scripts/check-deps.mjs"],
                        platform: PlatformType::Node,
                        ..Task::new("project:lint-deps")
                    }
                ),
                (
                    "lint-eslint".to_owned(),
                    Task {
                        command: "cross-env".to_owned(),
                        args: string_vec!["eslint", ".", "--format", "friendly"],
                        env: HashMap::from([("EFF_NO_LINK_RULES".to_owned(), "true".to_owned())]),
                        platform: PlatformType::Node,
                        ..Task::new("project:lint-eslint")
                    }
                ),
                (
                    "lint-prettier".to_owned(),
                    Task {
                        command: "prettier".to_owned(),
                        args: string_vec![".", "!test*", "--check"],
                        platform: PlatformType::Node,
                        ..Task::new("project:lint-prettier")
                    }
                ),
                (
                    "lint-spellcheck".to_owned(),
                    Task {
                        command: "cspell".to_owned(),
                        args: string_vec!["--no-progress", "--relative", "--dot", "--gitignore"],
                        platform: PlatformType::Node,
                        ..Task::new("project:lint-spellcheck")
                    }
                ),
                (
                    "lint-typecheck".to_owned(),
                    Task {
                        command: "tsc".to_owned(),
                        platform: PlatformType::Node,
                        ..Task::new("project:lint-typecheck")
                    }
                ),
                (
                    "fix-eslint".to_owned(),
                    Task {
                        command: "moon".to_owned(),
                        args: string_vec!["run", "project:lint-eslint", "--", "--fix"],
                        platform: PlatformType::Node,
                        ..Task::new("project:fix-eslint")
                    }
                ),
                (
                    "fix-prettier".to_owned(),
                    Task {
                        command: "moon".to_owned(),
                        args: string_vec!["run", "project:lint-prettier", "--", "--write"],
                        platform: PlatformType::Node,
                        ..Task::new("project:fix-prettier")
                    }
                ),
                (
                    "build".to_owned(),
                    Task {
                        command: "node".to_owned(),
                        args: string_vec!["./scripts/build/build.mjs"],
                        platform: PlatformType::Node,
                        ..Task::new("project:build")
                    }
                ),
                (
                    "build-website".to_owned(),
                    Task {
                        command: "node".to_owned(),
                        args: string_vec!["./scripts/build-website.mjs"],
                        platform: PlatformType::Node,
                        ..Task::new("project:build-website")
                    }
                ),
                (
                    "perf".to_owned(),
                    Task {
                        command: "cross-env".to_owned(),
                        args: string_vec!["node", "./dist/bin-prettier.js"],
                        deps: string_vec!["~:perf-dep1"],
                        env: HashMap::from([("NODE_ENV".to_owned(), "production".to_owned())]),
                        platform: PlatformType::Node,
                        ..Task::new("project:perf")
                    }
                ),
                (
                    "perf-benchmark".to_owned(),
                    Task {
                        command: "moon".to_owned(),
                        args: string_vec!["run", "project:perf", "--", "--debug-benchmark"],
                        platform: PlatformType::Node,
                        ..Task::new("project:perf-benchmark")
                    }
                ),
                (
                    "perf-inspect".to_owned(),
                    Task {
                        command: "cross-env".to_owned(),
                        args: string_vec!["node", "--inspect-brk", "./dist/bin-prettier.js"],
                        deps: string_vec!["~:perf-inspect-dep1"],
                        env: HashMap::from([("NODE_ENV".to_owned(), "production".to_owned())]),
                        platform: PlatformType::Node,
                        ..Task::new("project:perf-inspect")
                    }
                ),
                (
                    "perf-inspect-dep1".to_owned(),
                    Task {
                        command: "moon".to_owned(),
                        args: string_vec!["run", "project:build"],
                        platform: PlatformType::Node,
                        ..Task::new("project:perf-inspect-dep1")
                    }
                ),
                (
                    "perf-dep1".to_owned(),
                    Task {
                        command: "moon".to_owned(),
                        args: string_vec!["run", "project:build"],
                        platform: PlatformType::Node,
                        ..Task::new("project:perf-dep1")
                    }
                ),
                (
                    "test".to_owned(),
                    Task {
                        command: "jest".to_owned(),
                        platform: PlatformType::Node,
                        ..Task::new("project:test")
                    }
                ),
                (
                    "test-dev-package".to_owned(),
                    Task {
                        command: "cross-env".to_owned(),
                        args: string_vec!["jest"],
                        env: HashMap::from([("INSTALL_PACKAGE".to_owned(), "1".to_owned())]),
                        platform: PlatformType::Node,
                        ..Task::new("project:test-dev-package")
                    }
                ),
                (
                    "test-dist".to_owned(),
                    Task {
                        command: "cross-env".to_owned(),
                        args: string_vec!["jest"],
                        env: HashMap::from([("NODE_ENV".to_owned(), "production".to_owned())]),
                        platform: PlatformType::Node,
                        ..Task::new("project:test-dist")
                    }
                ),
                (
                    "test-dist-lint".to_owned(),
                    Task {
                        command: "eslint".to_owned(),
                        args: string_vec![
                            "--no-eslintrc",
                            "--no-ignore",
                            "--no-inline-config",
                            "--config=./scripts/bundle-eslint-config.cjs",
                            "dist/**/*.{js,mjs}"
                        ],
                        platform: PlatformType::Node,
                        ..Task::new("project:test-dist-lint")
                    }
                ),
                (
                    "test-dist-standalone".to_owned(),
                    Task {
                        command: "cross-env".to_owned(),
                        args: string_vec!["jest"],
                        env: HashMap::from([
                            ("TEST_STANDALONE".to_owned(), "1".to_owned()),
                            ("NODE_ENV".to_owned(), "production".to_owned())
                        ]),
                        platform: PlatformType::Node,
                        ..Task::new("project:test-dist-standalone")
                    }
                ),
                (
                    "test-integration".to_owned(),
                    Task {
                        command: "jest".to_owned(),
                        args: string_vec!["tests/integration"],
                        platform: PlatformType::Node,
                        ..Task::new("project:test-integration")
                    }
                ),
                (
                    "vendors-bundle".to_owned(),
                    Task {
                        command: "node".to_owned(),
                        args: string_vec!["./scripts/vendors/bundle-vendors.mjs"],
                        platform: PlatformType::Node,
                        ..Task::new("project:vendors-bundle")
                    }
                ),
            ])
        );
    }
}
