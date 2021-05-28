use std::{process::Command, sync::Mutex};

use anyhow::anyhow;
use console::Style;
use once_cell::sync::Lazy;
use similar::{ChangeTag, TextDiff};
use structopt::StructOpt;

mod targets;
mod utils;

use crate::utils::{load_expected_output, run_capturing_stdout, run_command, rustc_is_nightly};

static ALL_ERRORS: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(vec![]));

#[derive(Debug, StructOpt)]
struct Options {
    #[structopt(subcommand)]
    cmd: TestCommand,
    /// Treat compiler warnings as errors (`RUSTFLAGS="--deny warnings"`)
    #[structopt(long, short)]
    deny_warnings: bool,
    /// Keep target toolchains that were installed as dependency
    #[structopt(long, short)]
    keep_targets: bool,
}

#[derive(Debug, StructOpt)]
#[allow(clippy::enum_variant_names)]
enum TestCommand {
    TestAll,
    TestBook,
    TestCross,
    TestHost,
    TestLint,
    TestSnapshot {
        #[structopt(short, help = "Name of the snapshot")]
        name: Option<String>,
    },
}

fn main() -> anyhow::Result<()> {
    let opt: Options = Options::from_args();

    // TODO: one could argue that not all test scenarios require installation of targets
    let added_targets = targets::install().expect("Error while installing required targets");

    match opt.cmd {
        TestCommand::TestAll => {
            test_host(opt.deny_warnings);
            test_cross();
            test_snapshot();
            test_book();
            test_lint();
        }
        TestCommand::TestHost => test_host(opt.deny_warnings),
        TestCommand::TestCross => test_cross(),
        TestCommand::TestSnapshot { name: None } => test_snapshot(),
        TestCommand::TestSnapshot { name: Some(name) } => test_single_snapshot(&name, "", false)?,
        TestCommand::TestBook => test_book(),
        TestCommand::TestLint => test_lint(),
    }

    if !opt.keep_targets && !added_targets.is_empty() {
        targets::uninstall(added_targets);
    }

    let all_errors = ALL_ERRORS.lock().unwrap();
    if !all_errors.is_empty() {
        eprintln!();
        Err(anyhow!("😔 some tests failed: {:#?}", all_errors))
    } else {
        Ok(())
    }
}

fn do_test(test: impl FnOnce() -> anyhow::Result<()>, context: &str) {
    test().unwrap_or_else(|e| ALL_ERRORS.lock().unwrap().push(format!("{}: {}", context, e)));
}

fn test_host(deny_warnings: bool) {
    println!("🧪 host");

    let env = if deny_warnings {
        vec![("RUSTFLAGS", "--deny warnings")]
    } else {
        vec![]
    };

    do_test(
        || run_command("cargo", &["check", "--workspace"], None, &env),
        "host",
    );

    do_test(
        || {
            run_command(
                "cargo",
                &["check", "--workspace", "--features", "unstable-test"],
                None,
                &env,
            )
        },
        "host",
    );

    do_test(
        || {
            run_command(
                "cargo",
                &["check", "--workspace", "--features", "alloc"],
                None,
                &env,
            )
        },
        "host",
    );

    do_test(
        || {
            run_command(
                "cargo",
                &["test", "--workspace", "--features", "unstable-test"],
                None,
                &[],
            )
        },
        "host",
    );

    do_test(
        || {
            run_command(
                "cargo",
                &["test", "--workspace", "--features", "unstable-test"],
                None,
                &[],
            )
        },
        "host",
    );
}

fn test_cross() {
    println!("🧪 cross");
    let targets = [
        "thumbv6m-none-eabi",
        "thumbv8m.base-none-eabi",
        "riscv32i-unknown-none-elf",
    ];

    for target in &targets {
        do_test(
            || run_command("cargo", &["check", "--target", target, "-p", "defmt"], None, &[]),
            "cross",
        );
        do_test(
            || {
                run_command(
                    "cargo",
                    &["check", "--target", target, "-p", "defmt", "--features", "alloc"],
                    None,
                    &[],
                )
            },
            "cross",
        );
    }

    do_test(
        || {
            run_command(
                "cargo",
                &[
                    "check",
                    "--target",
                    "thumbv6m-none-eabi",
                    "--workspace",
                    "--exclude",
                    "defmt-itm",
                    "--exclude",
                    "firmware",
                ],
                Some("firmware"),
                &[],
            )
        },
        "cross",
    );

    do_test(
        || {
            run_command(
                "cargo",
                &["check", "--target", "thumbv7em-none-eabi", "--workspace"],
                Some("firmware"),
                &[],
            )
        },
        "cross",
    );

    do_test(
        || {
            run_command(
                "cargo",
                &[
                    "check",
                    "--target",
                    "thumbv6m-none-eabi",
                    "--features",
                    "print-defmt",
                ],
                Some("firmware/panic-probe"),
                &[],
            )
        },
        "cross",
    );

    do_test(
        || {
            run_command(
                "cargo",
                &[
                    "check",
                    "--target",
                    "thumbv6m-none-eabi",
                    "--features",
                    "print-rtt",
                ],
                Some("firmware/panic-probe"),
                &[],
            )
        },
        "cross",
    )
}

fn test_snapshot() {
    println!("🧪 qemu/snapshot");
    let mut tests = vec![
        "log",
        "timestamp",
        "panic",
        "assert",
        "assert-eq",
        "assert-ne",
        "unwrap",
        "defmt-test",
        "hints",
        "dbg",
    ];

    if rustc_is_nightly() {
        tests.push("alloc");
    }

    for test in tests {
        let features = if test == "alloc" { "alloc" } else { "" };

        do_test(|| test_single_snapshot(test, features, false), "qemu/snapshot");
        do_test(|| test_single_snapshot(test, features, true), "qemu/snapshot");
    }
}

fn test_single_snapshot(name: &str, features: &str, release_mode: bool) -> anyhow::Result<()> {
    let display_name = format!("{} ({})", name, if release_mode { "release" } else { "dev" });
    println!("{}", display_name);

    let mut args = if release_mode {
        vec!["-q", "rrb", name]
    } else {
        vec!["-q", "rb", name]
    };

    if !features.is_empty() {
        args.extend_from_slice(&["--features", features]);
    }

    const CWD: &str = "firmware/qemu";
    let actual = run_capturing_stdout(Command::new("cargo").args(&args).current_dir(CWD))?;
    let expected = load_expected_output(name, release_mode)?;
    let diff = TextDiff::from_lines(&expected, &actual);

    // if anything isn't ChangeTag::Equal, print it and turn on error flag
    let mut actual_matches_expected = true;
    for op in diff.ops() {
        for change in diff.iter_changes(op) {
            let styled_change = match change.tag() {
                ChangeTag::Delete => Some(("-", Style::new().red())),
                ChangeTag::Insert => Some(("+", Style::new().green())),
                ChangeTag::Equal => None,
            };
            if let Some((sign, style)) = styled_change {
                actual_matches_expected = false;
                eprint!("{}{}", style.apply_to(sign).bold(), style.apply_to(change),);
            }
        }
    }

    if actual_matches_expected {
        Ok(())
    } else {
        Err(anyhow!("{}", display_name))
    }
}

fn test_book() {
    println!("🧪 book");
    do_test(|| run_command("cargo", &["clean"], None, &[]), "book");

    do_test(
        || run_command("cargo", &["build", "--features", "unstable-test"], None, &[]),
        "book",
    );

    do_test(
        || {
            run_command(
                "mdbook",
                &["test", "-L", "../target/debug", "-L", "../target/debug/deps"],
                Some("book"),
                &[],
            )
        },
        "book",
    );
}

fn test_lint() {
    println!("🧪 lint");
    do_test(|| run_command("cargo", &["clean"], None, &[]), "lint");
    do_test(
        || run_command("cargo", &["fmt", "--all", "--", "--check"], None, &[]),
        "lint",
    );

    do_test(
        || run_command("cargo", &["clippy", "--workspace"], None, &[]),
        "lint",
    );
}
