mod auth;
mod db;
mod generate;
mod import;
mod intake;
mod forms_markup;
mod php_serialize;
mod render;
mod render_archive;
mod render_post;
mod render_tour;
mod schema;
mod widgets;
mod admin;
mod sitefx;
mod sqldump;
mod verify;

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
    },
    User {
        #[command(subcommand)]
        step: UserStep,
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
        /// URL segment for the tour CPT (e.g. "voyage").
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
            RenderStep::Slug { slug, db, templates, out, site_url } => {
                render::run(&db, &templates, &slug, &out, &site_url)
            }
            RenderStep::Coverage { db } => render::coverage(&db),
            RenderStep::VerifyRefs { db, templates, site_url } => {
                verify::verify_refs(&db, &templates, &site_url)
            }
        },
        Cmd::Generate { db, templates, out, site_url } => {
            generate::run(&db, &templates, &out, &site_url)
        },
        Cmd::User { step } => match step {
            UserStep::Add { email, password, role, db } => auth::add_user(&db, &email, &password, &role),
            UserStep::Passwd { email, password, db } => auth::set_password(&db, &email, &password),
        },
        Cmd::Migrate { dump, db, templates, out, site_url, www } => {
            let _ = www; // uploads land in www via admin; migrate only needs db+out
            println!("== import ==");
            import::run(&dump, &db, &db.with_extension("import-report.txt"))?;
            let password = rand::random::<u128>().to_string();
            let (password, _) = password.split_at(24.min(password.len()));
            auth::add_user(&db, "admin@localhost", password, "admin")?;
            println!("admin user: admin@localhost\npassword: {password}");
            println!("== generate ==");
            generate::run(&db, &templates, &out, &site_url)?;
            println!(
                "== done ==\nnext: rpress serve --bind 127.0.0.1:8400 --db {} --templates {} --out {}\nadmin:  http://127.0.0.1:8400/admin",
                db.display(),
                templates.display(),
                out.display(),
            );
            Ok(())
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
