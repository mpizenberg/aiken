use aiken_project::{
    config::{self, Config},
    package_name::{self, PackageName},
};
use indoc::{formatdoc, indoc};
use miette::IntoDiagnostic;
use owo_colors::{OwoColorize, Stream::Stderr};
use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

#[derive(clap::Args)]
/// Create a new Aiken project
pub struct Args {
    /// Project name
    name: String,
    /// Library only
    #[clap(long, short)]
    lib: bool,
}

pub fn exec(args: Args) -> miette::Result<()> {
    let package_name = PackageName::from_str(&args.name).into_diagnostic()?;
    create_project(args, &package_name)?;
    print_success_message(&package_name);
    Ok(())
}

fn create_project(args: Args, package_name: &PackageName) -> miette::Result<()> {
    let root = PathBuf::from(&package_name.repo);

    if root.exists() {
        Err(package_name::Error::ProjectExists {
            name: package_name.repo.clone(),
        })?;
    }

    create_lib(&root)?;

    if !args.lib {
        create_env(&root)?;
        create_validators(&root)?;
        create_validator(&root)?;
    }

    readme(&root, &package_name.repo)?;

    Config::default(package_name)
        .save(&root)
        .into_diagnostic()?;

    create_github_action(&root)?;

    gitignore(&root)?;

    Ok(())
}

fn print_success_message(package_name: &PackageName) {
    eprintln!(
        "\n{}",
        formatdoc! {
            r#"Your Aiken project {package} has been {s} created.
               The project can be compiled and tested by running these commands:

                   {cd} {folder}
                   {aiken} check
                   {aiken} build

               {hint} You may want to update the {stdlib} version in {toml}.
            "#,
            s = "successfully"
                .if_supports_color(Stderr, |s| s.bright_green())
                .if_supports_color(Stderr, |s| s.bold()),
            cd = "cd"
                .if_supports_color(Stderr, |s| s.purple())
                .if_supports_color(Stderr, |s| s.bold()),
            package = package_name.to_string()
                .if_supports_color(Stderr, |s| s.bright_blue()),
            folder = package_name.repo
                .if_supports_color(Stderr, |s| s.bright_blue()),
            aiken = "aiken"
                .if_supports_color(Stderr, |s| s.purple())
                .if_supports_color(Stderr, |s| s.bold()),
            hint = "hint:"
                .if_supports_color(Stderr, |s| s.cyan())
                .if_supports_color(Stderr, |s| s.bold()),
            stdlib = "stdlib"
                .if_supports_color(Stderr, |s| s.purple())
                .if_supports_color(Stderr, |s| s.bold()),
            toml = "aiken.toml"
                .if_supports_color(Stderr, |s| s.bold()),
        }
    )
}

fn create_env(root: &Path) -> miette::Result<()> {
    let env = root.join("env");
    fs::create_dir_all(env).into_diagnostic()
}

fn create_lib(root: &Path) -> miette::Result<()> {
    let lib = root.join("lib");
    fs::create_dir_all(lib).into_diagnostic()
}

fn create_validators(root: &Path) -> miette::Result<()> {
    let validators = root.join("validators");
    fs::create_dir_all(validators).into_diagnostic()
}

fn create_validator(root: &Path) -> miette::Result<()> {
    let validators = root.join("validators");
    fs::write(
        validators.join("hello.ak"),
        indoc! {
            r#"
            use cardano/address.{{Credential}}
            use cardano/assets.{{PolicyId}}
            use cardano/certificate.{{Certificate}}
            use cardano/governance.{{ProposalProcedure, Voter}}
            use cardano/transaction.{{Transaction, OutputReference}}
            
            validator my_script {{
                mint(redeemer: MyMintRedeemer, policy_id: PolicyId, self: Transaction) {{
                    todo @"mint logic goes here"
                }}
                
                spend(datum: Option<MyDatum>, redeemer: MySpendRedeemer, utxo: OutputReference, self: Transaction) {{
                    todo @"spend logic goes here"
                }}
                
                withdraw(redeemer: MyWithdrawRedeemer, account: Credential, self: Transaction) {{
                    todo @"withdraw logic goes here"
                }}
                
                publish(redeemer: MyPublishRedeemer, certificate: Certificate, self: Transaction) {{
                    todo @"publish logic goes here"
                }}
                
                vote(redeemer: MyVoteRedeemer, voter: Voter, self: Transaction) {{
                    todo @"vote logic goes here"
                }}
                
                propose(redeemer: MyProposeRedeemer, proposal: ProposalProcedure, self: Transaction) {{
                    todo @"propose logic goes here"
                }}
            }}
            "#,
        },
    ).into_diagnostic()
}

fn readme(root: &Path, project_name: &str) -> miette::Result<()> {
    fs::write(
        root.join("README.md"),
        formatdoc! {
            r#"
                # {name}

                Write validators in the `validators` folder, and supporting functions in the `lib` folder using `.ak` as a file extension.

                ```aiken
                validator my_first_validator {{
                  spend(_datum: Option<Data>, _redeemer: Data, _output_reference: Data, _context: Data) {{
                    True
                  }}
                }}
                ```

                ## Building

                ```sh
                aiken build
                ```

                ## Configuring

                **aiken.toml**
                ```toml
                [config.default]
                network_id = 41
                ```

                Or, alternatively, write conditional environment modules under `env`.

                ## Testing

                You can write tests in any module using the `test` keyword. For example:

                ```aiken
                use config

                test foo() {{
                  config.network_id + 1 == 42
                }}
                ```

                To run all tests, simply do:

                ```sh
                aiken check
                ```

                To run only tests matching the string `foo`, do:

                ```sh
                aiken check -m foo
                ```

                ## Documentation

                If you're writing a library, you might want to generate an HTML documentation for it.

                Use:

                ```sh
                aiken docs
                ```

                ## Resources

                Find more on the [Aiken's user manual](https://aiken-lang.org).
            "#,
            name = project_name
        },
    ).into_diagnostic()
}

fn create_github_action(root: &Path) -> miette::Result<()> {
    let workflows = root.join(".github").join("workflows");

    fs::create_dir_all(&workflows).into_diagnostic()?;

    fs::write(
        workflows.join("continuous-integration.yml"),
        formatdoc! {
            r#"
            name: Continuous Integration

            on:
              push:
                branches: ["main"]
              pull_request:

            jobs:
              build:
                runs-on: ubuntu-latest
                steps:
                  - uses: actions/checkout@v3
                  - uses: aiken-lang/setup-aiken@v1
                    with:
                      version: {version}
                  - run: aiken fmt --check
                  - run: aiken check -D
                  - run: aiken build
            "#,
            version = config::compiler_version(false),
        },
    )
    .into_diagnostic()?;

    Ok(())
}

fn gitignore(root: &Path) -> miette::Result<()> {
    let gitignore_path = root.join(".gitignore");

    fs::write(
        gitignore_path,
        indoc! {
            r#"
                # Aiken compilation artifacts
                artifacts/
                # Aiken's project working directory
                build/
                # Aiken's default documentation export
                docs/
            "#
        },
    )
    .into_diagnostic()?;

    Ok(())
}
