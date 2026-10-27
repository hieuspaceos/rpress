// Single module list: the binary consumes the `rpress` library crate so
// the two targets can't drift on module visibility (module hygiene L).
use rpress::{admin, auth, generate, import, render, site_build, site_deploy, site_init, site_serve, verify};

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rpress")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Import WP dump → rpress.db
    Import {
        #[command(subcommand)]
        step: ImportStep,
    },
    /// Render a slug through templates
    Render {
        #[command(subcommand)]
        step: RenderStep,
    },
    /// Write static HTML tree
    Generate {
        #[arg(long, default_value = "rpress.db")]
        db: PathBuf,
        #[arg(long, default_value = "templates")]
        templates: PathBuf,
        #[arg(long, default_value = "cache")]
        out: PathBuf,
        #[arg(long, default_value = "http://localhost:8400")]
        site_url: String,
        /// URL base for the tour plugin (e.g. "voyage").
        #[arg(long, default_value = "voyage")]
        tour_base: String,
    },
    /// WP → rpress one-liner: import dump, create admin, generate site
    Migrate {
        dump: PathBuf,
        #[arg(long, default_value = "rpress.db")]
        db: PathBuf,
        #[arg(long, default_value = "templates")]
        templates: PathBuf,
        #[arg(long, default_value = "cache")]
        out: PathBuf,
        #[arg(long, default_value = "http://localhost:8400")]
        site_url: String,
        #[arg(long)]
        www: Option<PathBuf>,
        /// URL base for the tour plugin (e.g. "voyage").
        #[arg(long, default_value = "voyage")]
        tour_base: String,
    },
    User {
        #[command(subcommand)]
        step: UserStep,
    },
    /// Site workflow: init → build → serve → deploy (one env per site)
    Site {
        #[command(subcommand)]
        step: SiteStep,
    },
    Serve {
        #[arg(long, default_value = "127.0.0.1:8400")]
        bind: String,
        #[arg(long, default_value = "rpress.db")]
        db: PathBuf,
        #[arg(long, default_value = "templates")]
        templates: PathBuf,
        #[arg(long, default_value = "cache")]
        out: PathBuf,
        /// Persistent data dir: user uploads + vendored assets (NOT cache).
        /// Defaults to `<out>/../www` — sibling of the disposable cache dir.
        #[arg(long)]
        www: Option<PathBuf>,
        /// Host this site answers on; '*' = catch-all (single-site).
        #[arg(long, default_value = "*")]
        host: String,
        /// Short site slug (session scoping).
        #[arg(long, default_value = "default")]
        site: String,
        /// Brand name shown in the admin chrome.
        #[arg(long)]
        name: Option<String>,
        /// URL base for the tour plugin (e.g. "voyage").
        #[arg(long, default_value = "voyage")]
        tour_base: String,
    },
}

#[derive(Subcommand)]
enum ImportStep {
    /// Parse dump, map to rpress.db, write import-report.txt
    Parse {
        #[arg(long, default_value = "dump.sql")]
        dump: PathBuf,
        #[arg(long, default_value = "rpress.db")]
        db: PathBuf,
        #[arg(long, default_value = "import-report.txt")]
        report: PathBuf,
    },
    /// Run audit-count checklist against rpress.db
    Verify {
        #[arg(long, default_value = "rpress.db")]
        db: PathBuf,
    },
}

#[derive(Subcommand)]
enum RenderStep {
    Slug {
        slug: String,
        #[arg(long, default_value = "rpress.db")]
        db: PathBuf,
        #[arg(long, default_value = "templates")]
        templates: PathBuf,
        #[arg(long, default_value = "out.html")]
        out: PathBuf,
        #[arg(long, default_value = "http://localhost:8400")]
        site_url: String,
        /// URL base for the tour plugin (e.g. "voyage").
        #[arg(long, default_value = "voyage")]
        tour_base: String,
    },
    Coverage {
        #[arg(long, default_value = "rpress.db")]
        db: PathBuf,
    },
    /// Tag-diff parity check vs templates/reference/
    VerifyRefs {
        #[arg(long, default_value = "rpress.db")]
        db: PathBuf,
        #[arg(long, default_value = "templates")]
        templates: PathBuf,
        #[arg(long, default_value = "http://localhost:8400")]
        site_url: String,
        /// URL base for the tour plugin (e.g. "voyage").
        #[arg(long, default_value = "voyage")]
        tour_base: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Import { step } => match step {
            ImportStep::Parse { dump, db, report } => import::run(&dump, &db, &report),
            ImportStep::Verify { db } => verify::run(&db),
        },
        Cmd::Render { step } => match step {
            RenderStep::Slug { slug, db, templates, out, site_url, tour_base } => {
                render::run(&db, &templates, &slug, &out, &site_url, &tour_base)
            }
            RenderStep::Coverage { db } => render::coverage(&db),
            RenderStep::VerifyRefs { db, templates, site_url, tour_base } => {
                verify::verify_refs(&db, &templates, &site_url, &tour_base)
            }
        },
        Cmd::Generate { db, templates, out, site_url, tour_base } => {
            generate::run(&db, &templates, &out, &site_url, &tour_base)
        },
        Cmd::User { step } => match step {
            UserStep::Add { email, password, role, db } => auth::add_user(&db, &email, &password, &role),
            UserStep::Passwd { email, password, db } => auth::set_password(&db, &email, &password),
        },
        Cmd::Migrate { dump, db, templates, out, site_url, www, tour_base } => {
            let _ = www; // uploads land in www via admin; migrate only needs db+out
            println!("== import ==");
            import::run(&dump, &db, &db.with_extension("import-report.txt"))?;
            let password = rand::random::<u128>().to_string();
            let (password, _) = password.split_at(24.min(password.len()));
            auth::add_user(&db, "admin@localhost", password, "admin")?;
            println!("admin user: admin@localhost\npassword: {password}");
            println!("== generate ==");
            generate::run(&db, &templates, &out, &site_url, &tour_base)?;
            println!(
                "== done ==\nnext: rpress serve --bind 127.0.0.1:8400 --db {} --templates {} --out {}\nadmin:  http://127.0.0.1:8400/admin",
                db.display(),
                templates.display(),
                out.display(),
            );
            Ok(())
        },
        Cmd::Site { step } => match step {
            SiteStep::Init(args) => site_init::run(site_init::InitArgs {
                site: args.site,
                source: args.source,
                ssh: args.ssh,
                wp_root: args.wp_root,
                dump_remote: args.dump_remote,
                fresh_dump: args.fresh_dump,
                dump: args.dump,
                wp_content: args.wp_content,
                backup: args.backup,
                site_url: args.site_url,
                bind: args.bind,
                name: args.name,
                tour_base: args.tour_base,
                clone_ok: args.clone_ok,
                force: args.force,
            }),
            SiteStep::Build { site, only, from, json, templates } => {
                site_build::run(site_build::BuildArgs { site, only, from, json, templates })
            }
            SiteStep::Serve { site, bind, smoke } => {
                site_serve::run(site_serve::ServeArgs { site, bind, smoke })
            }
            SiteStep::Deploy { site, target, domain, dest, project, app_dir, yes, dry_run } => {
                site_deploy::run(site_deploy::DeployArgs { site, target, domain, dest, project, app_dir, yes, dry_run })
            }
        },
        Cmd::Serve { bind, db, templates, out, www, host, site, name, tour_base } => {
            let addr: std::net::SocketAddr = bind.parse()?;
            let name = name.unwrap_or_else(|| host.clone());
            let www = www.unwrap_or_else(|| {
                out.parent().unwrap_or(std::path::Path::new(".")).join("www")
            });
            tokio::runtime::Runtime::new()?.block_on(admin::serve(db, templates, out, www, addr, site, host, name, tour_base))
        }
    }
}

#[derive(Subcommand)]
enum SiteStep {
    /// Onboard a WP source → writes sites/<name>.env
    Init(InitArgs),
    /// pull → import → configure → generate → check, per-phase reporting
    Build {
        /// Site slug (sites/<slug>.env)
        site: String,
        /// Run only these phases, comma-separated
        #[arg(long)]
        only: Option<String>,
        /// Resume from this phase onward
        #[arg(long)]
        from: Option<String>,
        /// Machine-readable report on stdout
        #[arg(long)]
        json: bool,
        /// Templates dir (default: ./templates)
        #[arg(long)]
        templates: Option<PathBuf>,
    },
    /// Preflight + test checklist + serve the sandbox site
    Serve {
        /// Site slug (sites/<slug>.env)
        site: String,
        /// Override the env's BIND
        #[arg(long)]
        bind: Option<String>,
        /// Spawn, probe the key routes, report and exit — no browser needed
        #[arg(long)]
        smoke: bool,
    },
    /// Deploy a tested site: --target static|pages|vps, always with --domain
    Deploy {
        /// Site slug (sites/<slug>.env)
        site: String,
        /// static | pages | vps
        #[arg(long)]
        target: String,
        /// Public origin, e.g. https://example.com (never guessed)
        #[arg(long)]
        domain: Option<String>,
        /// static/vps: <dir> | <user@host>:<path> | vps:<alias>:<path>
        #[arg(long)]
        dest: Option<String>,
        /// pages: Cloudflare Pages project name (default: site slug)
        #[arg(long)]
        project: Option<String>,
        /// vps: remote app dir label for the runbook (default: dest path)
        #[arg(long)]
        app_dir: Option<String>,
        /// Skip the confirmation prompt (agents)
        #[arg(long)]
        yes: bool,
        /// Print the plan, change nothing
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(clap::Args)]
struct InitArgs {
    /// Site slug (sites/<slug>.env); prompted when interactive
    site: Option<String>,
    /// ssh | local | archive
    #[arg(long)]
    source: Option<String>,
    /// user@host — probe wp-config, DB creds and dumps over ssh
    #[arg(long)]
    ssh: Option<String>,
    /// Remote WordPress root (skips wp-config discovery)
    #[arg(long)]
    wp_root: Option<String>,
    /// Remote .sql path (skips dump discovery)
    #[arg(long)]
    dump_remote: Option<String>,
    /// Export a fresh dump on the host via wp-cli/mysqldump
    #[arg(long)]
    fresh_dump: bool,
    /// Local .sql[.gz] file, or a dir to scan for one
    #[arg(long)]
    dump: Option<PathBuf>,
    /// Local wp-content dir (or a parent dir to scan)
    #[arg(long)]
    wp_content: Option<PathBuf>,
    /// Backup archive: .zip / .tar(.gz) / .sql.gz / .sql
    #[arg(long)]
    backup: Option<PathBuf>,
    /// Origin URL the dump came from (probed when possible)
    #[arg(long)]
    site_url: Option<String>,
    /// Local bind for serve, e.g. 127.0.0.1:8410 (auto-picks a free port)
    #[arg(long)]
    bind: Option<String>,
    /// Display name in admin chrome
    #[arg(long)]
    name: Option<String>,
    /// URL base for the tour plugin (e.g. "voyage")
    #[arg(long)]
    tour_base: Option<String>,
    /// Confirm the pull source is a clone/staging, never production
    #[arg(long)]
    clone_ok: bool,
    /// Overwrite an existing sites/<name>.env
    #[arg(long)]
    force: bool,
}

#[derive(Subcommand)]
enum UserStep {
    Add {
        email: String,
        password: String,
        #[arg(long, default_value = "editor")]
        role: String,
        #[arg(long, default_value = "rpress.db")]
        db: PathBuf,
    },
    Passwd {
        email: String,
        password: String,
        #[arg(long, default_value = "rpress.db")]
        db: PathBuf,
    },
}
